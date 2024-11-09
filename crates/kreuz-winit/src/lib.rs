mod subwindow;
mod window;

pub use subwindow::*;
pub use window::*;

use anyhow::Result;
use kreuz_window::{
    AppHandler, ButtonState, MouseButton, SubwindowHandler, SubwindowParams, WindowEvent,
    WindowHandler, WindowId, WindowParams,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::{collections::HashMap, sync::Arc};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{
        ElementState as WinitElementState, MouseButton as WinitMouseButton,
        WindowEvent as WinitWindowEvent,
    },
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId as WinitId},
};

struct WinitEventLoop<A: AppHandler<WinitWinHandler, WinitSubwinHandler>> {
    app: A,
    windows: HashMap<WinitId, (WindowId, Option<WinitWinHandler>)>,
    subwindows: HashMap<WinitId, (WindowId, Option<WinitSubwinHandler>)>,
}

impl<A: AppHandler<WinitWinHandler, WinitSubwinHandler>> ApplicationHandler for WinitEventLoop<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            let window = WinitWinHandler(create_winit_window(
                event_loop,
                &self.app.default_window_params(),
            ));
            let winit_id = window.0.id();
            let window_id = WindowId::new();

            self.app.handle_window_update(window_id, window.clone());

            self.windows.insert(winit_id, (window_id, Some(window)));
            return;
        }

        let default_window_params = self.app.default_window_params();
        for (_, (id, cached_window)) in &mut self.windows {
            let window = cached_window.take().unwrap_or_else(|| {
                WinitWinHandler(create_winit_window(event_loop, &default_window_params))
            });

            self.app.handle_window_update(*id, window.clone());

            *cached_window = Some(window);
        }
        // let default_subwindow_params = self.app.default_subwindow_params();
        // for (win_id, (id, cached_window)) in &mut self.subwindows {
        //     todo!("Subwindows aren't supported for winit")
        // }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WinitWindowEvent,
    ) {
        macro_rules! pass_event {
            ($event:expr) => {
                if let Some((_, (id, _))) = self.windows.get_key_value(&window_id) {
                    self.app.handle_window_event(*id, $event);
                }
                if let Some((_, (id, _))) = self.subwindows.get_key_value(&window_id) {
                    self.app.handle_window_event(*id, $event);
                }
            };
        }

        match event {
            WinitWindowEvent::CloseRequested => event_loop.exit(),

            WinitWindowEvent::Resized(size) => {
                let new_size = (size.width as f64, size.height as f64).into();
                pass_event!(WindowEvent::Resize { new_size });
            }

            WinitWindowEvent::RedrawRequested => {
                pass_event!(WindowEvent::Redraw);
            }

            WinitWindowEvent::CursorLeft { .. } => {
                pass_event!(WindowEvent::CursorLeft);
            }

            WinitWindowEvent::CursorEntered { .. } => {
                pass_event!(WindowEvent::CursorEntered);
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x, position.y).into();
                pass_event!(WindowEvent::CursorMove { pos });
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let state = match state {
                    WinitElementState::Pressed => ButtonState::Pressed,
                    WinitElementState::Released => ButtonState::Released,
                };
                let button = match button {
                    WinitMouseButton::Left => Some(MouseButton::Left),
                    WinitMouseButton::Right => Some(MouseButton::Right),
                    WinitMouseButton::Middle => Some(MouseButton::Middle),
                    WinitMouseButton::Back => Some(MouseButton::Back),
                    WinitMouseButton::Forward => Some(MouseButton::Forward),
                    WinitMouseButton::Other(_) => None,
                };
                if let Some(button) = button {
                    pass_event!(WindowEvent::MouseButton { button, state });
                }
            }

            _ => {}
        }
    }
}

pub fn run_with_winit<A: AppHandler<WinitWinHandler, WinitSubwinHandler>>(app: A) -> Result<()> {
    // Setup a bunch of state:
    let mut app = WinitEventLoop {
        app,
        windows: HashMap::new(),
        subwindows: HashMap::new(),
    };

    // Create and run a winit event loop
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &ActiveEventLoop, window_params: &WindowParams) -> Arc<Window> {
    let WindowParams {
        size,
        position,
        resizable,
        title,
        ..
    } = window_params;
    let mut attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(size.width, size.height))
        .with_resizable(*resizable)
        .with_title(title);
    if let Some(pos) = position {
        attr = attr.with_position(LogicalPosition::new(pos.x as u32, pos.y as u32));
    }
    Arc::new(event_loop.create_window(attr).unwrap())
}
