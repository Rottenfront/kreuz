// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use std::{num::NonZeroUsize, sync::Arc};
use vello::{
    kurbo::{Affine, Circle, Ellipse, Line, RoundedRect, Stroke},
    peniko::{Blob, Brush, BrushRef, Color, Font, StyleRef},
    skrifa::{raw::FontRef, MetadataProvider},
    util::{RenderContext, RenderSurface},
    wgpu, AaConfig, Glyph, Renderer, RendererOptions, Scene,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

// Simple struct to hold the state of the renderer
pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

enum RenderState<'s> {
    Active(ActiveRenderState<'s>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<Window>>),
}

struct SimpleVelloApp<'s> {
    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState<'s>,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
}

impl<'s> ApplicationHandler for SimpleVelloApp<'s> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        // Get the winit window cached in a previous Suspended event or else create a new window
        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));
        window.request_redraw();

        // Create a vello Surface
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        // Create a vello Renderer for the surface (using its device id)
        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.state = RenderState::Active(ActiveRenderState { window, surface });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let RenderState::Active(state) = &self.state {
            self.state = RenderState::Suspended(Some(state.window.clone()));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // Ignore the event (return from the function) if
        //   - we have no render_state
        //   - OR the window id of the event doesn't match the window id of our render_state
        //
        // Else extract a mutable reference to the render state from its containing option for use below
        let render_state = match &mut self.state {
            RenderState::Active(state) if state.window.id() == window_id => state,
            _ => return,
        };

        match event {
            // Exit the event loop when a close is requested (e.g. window's close button is pressed)
            WindowEvent::CloseRequested => event_loop.exit(),

            // Resize the surface when the window is resized
            WindowEvent::Resized(size) => {
                self.context
                    .resize_surface(&mut render_state.surface, size.width, size.height);
                if let RenderState::Active(state) = &self.state {
                    state.window.request_redraw();
                }
            }

            // This is where all the rendering happens
            WindowEvent::RedrawRequested => {
                // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
                // the same Scene is reused so that the underlying memory allocation can also be reused.
                self.scene.reset();

                // Re-add the objects to draw to the scene.
                add_shapes_to_scene(&mut self.scene);

                // Get the RenderSurface (surface + config)
                let surface = &render_state.surface;

                // Get the window size
                let width = surface.config.width;
                let height = surface.config.height;

                // Get a handle to the device
                let device_handle = &self.context.devices[surface.dev_id];

                // Get the surface's texture
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Render to the surface's texture
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_surface(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface_texture,
                        &vello::RenderParams {
                            base_color: Color::BLACK, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("failed to render to surface");

                // Queue the texture to be presented on the surface
                surface_texture.present();

                device_handle.device.poll(wgpu::Maintain::Poll);
            }
            _ => {}
        }
    }
}

pub fn run() -> Result<()> {
    // Setup a bunch of state:
    let mut app = SimpleVelloApp {
        context: RenderContext::new(),
        renderers: vec![],
        state: RenderState::Suspended(None),
        scene: Scene::new(),
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(1044, 800))
        .with_resizable(true)
        .with_title("Vello Shapes");
    Arc::new(event_loop.create_window(attr).unwrap())
}

/// Helper function that creates a vello `Renderer` for a given `RenderContext` and `RenderSurface`
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Couldn't create renderer")
}

/// Add shapes to a vello scene. This does not actually render the shapes, but adds them
/// to the Scene data structure which represents a set of objects to draw.
fn add_shapes_to_scene(scene: &mut Scene) {
    // Draw an outlined rectangle
    let stroke = Stroke::new(6.0);
    let rect = RoundedRect::new(10.0, 10.0, 240.0, 240.0, 20.0);
    let rect_stroke_color = Color::rgb(0.9804, 0.702, 0.5294);
    scene.stroke(&stroke, Affine::IDENTITY, rect_stroke_color, None, &rect);

    // Draw a filled circle
    let circle = Circle::new((420.0, 200.0), 120.0);
    let circle_fill_color = Color::rgb(0.9529, 0.5451, 0.6588);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        circle_fill_color,
        None,
        &circle,
    );

    // Draw a filled ellipse
    let ellipse = Ellipse::new((250.0, 420.0), (100.0, 160.0), -90.0);
    let ellipse_fill_color = Color::rgb(0.7961, 0.651, 0.9686);
    scene.fill(
        vello::peniko::Fill::NonZero,
        Affine::IDENTITY,
        ellipse_fill_color,
        None,
        &ellipse,
    );

    // Draw a straight line
    let line = Line::new((260.0, 20.0), (620.0, 100.0));
    let line_stroke_color = Color::rgb(0.5373, 0.7059, 0.9804);
    scene.stroke(&stroke, Affine::IDENTITY, line_stroke_color, None, &line);
}

// This is very much a hack to get things working.
// On Windows, can set this to "c:\\Windows\\Fonts\\seguiemj.ttf" to get color emoji
const ROBOTO_FONT: &[u8] = include_bytes!("../assets/roboto/Roboto-Regular.ttf");
const INCONSOLATA_FONT: &[u8] = include_bytes!("../assets/inconsolata/Inconsolata.ttf");
const NOTO_EMOJI_CBTF_SUBSET: &[u8] =
    include_bytes!("../assets/noto_color_emoji/NotoColorEmoji-CBTF-Subset.ttf");
const NOTO_EMOJI_COLR_SUBSET: &[u8] =
    include_bytes!("../assets/noto_color_emoji/NotoColorEmoji-Subset.ttf");

pub struct SimpleText {
    roboto: Font,
    inconsolata: Font,
    noto_emoji_colr_subset: Font,
    noto_emoji_cbtf_subset: Font,
}

impl SimpleText {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            roboto: Font::new(Blob::new(Arc::new(ROBOTO_FONT)), 0),
            inconsolata: Font::new(Blob::new(Arc::new(INCONSOLATA_FONT)), 0),
            noto_emoji_colr_subset: Font::new(Blob::new(Arc::new(NOTO_EMOJI_COLR_SUBSET)), 0),
            noto_emoji_cbtf_subset: Font::new(Blob::new(Arc::new(NOTO_EMOJI_CBTF_SUBSET)), 0),
        }
    }

    /// Add a text run which supports some emoji.
    ///
    /// The supported Emoji are ✅, 👀, 🎉, and 🤠.
    /// This subset is chosen to demonstrate the emoji support, whilst
    /// not significantly increasing repository size.
    ///
    /// Note that Vello does support COLR emoji, but does not currently support
    /// any other forms of emoji.
    #[allow(clippy::too_many_arguments)]
    pub fn add_colr_emoji_run<'a>(
        &mut self,
        scene: &mut Scene,
        size: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let font = self.noto_emoji_colr_subset.clone();
        self.add_var_run(
            scene,
            Some(&font),
            size,
            &[],
            // This should be unused
            &Brush::Solid(Color::WHITE),
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    /// Add a text run which supports some emoji.
    ///
    /// The supported Emoji are ✅, 👀, 🎉, and 🤠.
    /// This subset is chosen to demonstrate the emoji support, whilst
    /// not significantly increasing repository size.
    ///
    /// This will use a CBTF font, which Vello supports.
    #[allow(clippy::too_many_arguments)]
    pub fn add_bitmap_emoji_run<'a>(
        &mut self,
        scene: &mut Scene,
        size: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let font = self.noto_emoji_cbtf_subset.clone();
        self.add_var_run(
            scene,
            Some(&font),
            size,
            &[],
            // This should be unused
            &Brush::Solid(Color::WHITE),
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_run<'a>(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        self.add_var_run(
            scene,
            font,
            size,
            &[],
            brush,
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_var_run<'a>(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        variations: &[(&str, f32)],
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let default_font = if variations.is_empty() {
            &self.roboto
        } else {
            &self.inconsolata
        };
        let font = font.unwrap_or(default_font);
        let font_ref = to_font_ref(font).unwrap();
        let brush = brush.into();
        let style = style.into();
        let axes = font_ref.axes();
        let font_size = vello::skrifa::instance::Size::new(size);
        let var_loc = axes.location(variations.iter().copied());
        let charmap = font_ref.charmap();
        let metrics = font_ref.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);
        let mut pen_x = 0f32;
        let mut pen_y = 0f32;
        scene
            .draw_glyphs(font)
            .font_size(size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .normalized_coords(var_loc.coords())
            .brush(brush)
            .hint(false)
            .draw(
                style,
                text.chars().filter_map(|ch| {
                    if ch == '\n' {
                        pen_y += line_height;
                        pen_x = 0.0;
                        return None;
                    }
                    let gid = charmap.map(ch).unwrap_or_default();
                    let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                    let x = pen_x;
                    pen_x += advance;
                    Some(Glyph {
                        id: gid.to_u32(),
                        x,
                        y: pen_y,
                    })
                }),
            );
    }

    pub fn add(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        brush: Option<&Brush>,
        transform: Affine,
        text: &str,
    ) {
        use vello::peniko::Fill;
        let brush = brush.unwrap_or(&Brush::Solid(Color::WHITE));
        self.add_run(
            scene,
            font,
            size,
            brush,
            transform,
            None,
            Fill::NonZero,
            text,
        );
    }
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    use vello::skrifa::raw::FileRef;
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}
