use kreuz_ui::{RootView, ViewEvent};
use kreuz_window::{
    AppHandler, AppResponce, MouseButton, SubwindowHandler, WindowEvent, WindowHandler, WindowId,
};
use peniko::kurbo::Point;
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use vello::{
    peniko::Color,
    util::{RenderContext, RenderSurface},
    wgpu, AaConfig, Renderer, RendererOptions, Scene,
};

// Simple struct to hold the state of the renderer
pub struct ActiveRenderState<'s, W> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<W>,
}

enum RenderState<'s, W> {
    Active(ActiveRenderState<'s, W>),
    Suspended,
}

struct Context {
    cursor_pos: Option<Point>,
    pressed_mb: HashMap<kreuz_ui::MouseButton, bool>,
}

fn window_to_view_event(event: WindowEvent, ctx: &mut Context) -> Option<ViewEvent> {
    match event {
        WindowEvent::Resize { .. } => None,
        WindowEvent::Redraw => None,
        WindowEvent::CursorEntered => Some(ViewEvent::CursorEntered),
        WindowEvent::CursorLeft => Some(ViewEvent::CursorLeft),
        WindowEvent::CursorMove { pos } => {
            ctx.cursor_pos = Some(pos);
            Some(ViewEvent::CursorMove { pos })
        }
        WindowEvent::MouseButton { button, state } => {
            let button = match button {
                MouseButton::Left => kreuz_ui::MouseButton::Left,
                MouseButton::Right => kreuz_ui::MouseButton::Right,
                MouseButton::Middle => kreuz_ui::MouseButton::Middle,
                MouseButton::Back => kreuz_ui::MouseButton::Back,
                MouseButton::Forward => kreuz_ui::MouseButton::Forward,
            };
            let pos = ctx.cursor_pos.unwrap_or(Default::default());
            Some(match state {
                kreuz_window::ButtonState::Pressed => {
                    ctx.pressed_mb.insert(button, true);
                    ViewEvent::MouseButtonPress { pos, button }
                }
                kreuz_window::ButtonState::Released => {
                    ctx.pressed_mb.remove(&button);
                    ViewEvent::MouseButtonRelease { pos, button }
                }
            })
        }
    }
}

pub struct OneWindowVelloApp<'s, W: WindowHandler, V: RootView> {
    context: RenderContext,
    renderer: Option<Renderer>,
    ctx: Context,
    state: RenderState<'s, W>,
    scene: Scene,
    root_view: V,
}

impl<'s, W: WindowHandler + 's, SW: SubwindowHandler, V: RootView> AppHandler<W, SW>
    for OneWindowVelloApp<'s, W, V>
{
    fn handle_window_event(&mut self, _window: WindowId, event: WindowEvent) -> AppResponce {
        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return AppResponce::Handled,
        };
        match event {
            WindowEvent::Resize { new_size } => {
                self.context.resize_surface(
                    &mut render_state.surface,
                    new_size.width as u32,
                    new_size.height as u32,
                );
                if let RenderState::Active(state) = &self.state {
                    state.window.request_redraw();
                }
            }
            WindowEvent::Redraw => {
                // Empty the scene of objects to draw. You could create a new Scene each time, but in this case
                // the same Scene is reused so that the underlying memory allocation can also be reused.
                self.scene.reset();

                self.root_view.render(&mut kreuz_ui::Scene {});

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
                self.renderer
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
            event => {
                let event = window_to_view_event(event, &mut self.ctx);
                if let Some(event) = event {
                    self.root_view.handle_event(&event);
                }
            }
        }

        AppResponce::Handled
    }

    fn handle_window_update(&mut self, _id: WindowId, window: W) -> AppResponce {
        let RenderState::Suspended = &mut self.state else {
            return AppResponce::Handled;
        };

        let window = Arc::new(window);

        window.request_redraw();

        // Create a vello Surface
        let params = window.get_params();
        let size = params.size;
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width as u32,
            size.height as u32,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        self.renderer = Some(create_vello_renderer(&self.context, &surface));

        // Save the Window and Surface to a state variable
        self.state = RenderState::Active(ActiveRenderState { window, surface });

        AppResponce::Handled
    }

    fn handle_subwindow_update(&mut self, _id: WindowId, _window: SW) -> AppResponce {
        AppResponce::Handled
    }
}

pub fn make_vello_app<'s, W: WindowHandler + 's, V: RootView>(
    view: V,
) -> OneWindowVelloApp<'s, W, V> {
    OneWindowVelloApp {
        context: RenderContext::new(),
        renderer: Default::default(),
        ctx: Context {
            cursor_pos: None,
            pressed_mb: HashMap::new(),
        },
        state: RenderState::Suspended,
        scene: Scene::new(),
        root_view: view,
    }
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
