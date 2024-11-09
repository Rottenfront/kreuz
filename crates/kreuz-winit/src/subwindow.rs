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

#[derive(Clone)]
pub struct WinitSubwinHandler;

impl HasWindowHandle for WinitSubwinHandler {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        todo!("Subwindows aren't supported right now")
    }
}

impl HasDisplayHandle for WinitSubwinHandler {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        todo!("Subwindows aren't supported right now")
    }
}

impl SubwindowHandler for WinitSubwinHandler {
    fn request_redraw(&self) {
        todo!("Subwindows aren't supported right now")
    }

    fn get_params(&self) -> SubwindowParams {
        todo!("Subwindows aren't supported right now")
    }
}
