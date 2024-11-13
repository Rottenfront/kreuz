use anyhow::Result;
use kreuz_window::{WindowHandler, WindowParams};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use std::sync::Arc;
use winit::window::Window;

#[derive(Clone)]
pub struct WinitWinHandler(pub Arc<Window>);

impl HasWindowHandle for WinitWinHandler {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for WinitWinHandler {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

impl WindowHandler for WinitWinHandler {
    fn request_redraw(&self) {
        self.0.request_redraw();
    }

    fn set_title(&self, title: &str) {
        self.0.set_title(title);
    }

    fn get_params(&self) -> WindowParams {
        let size = self.0.inner_size();
        let size = (size.width as f64, size.height as f64).into();
        let scale = self.0.scale_factor();
        let position = self.0.inner_position();
        let position = match position {
            Ok(pos) => Some((pos.x as f64, pos.y as f64).into()),
            Err(_) => None,
        };
        let resizable = self.0.is_resizable();
        let title = self.0.title();
        WindowParams {
            size,
            scale,
            position,
            resizable,
            title,
        }
    }
}
