use super::*;

pub enum AppResponce {
    Handled,
    Error(Box<dyn std::error::Error>),
    CloseWindow(WindowId),
    CreateNewWindow(WindowParams),
    CreateNewSubwindow(SubwindowParams),
}

pub trait AppHandler<W: WindowHandler, SW: SubwindowHandler> {
    fn default_window_params(&self) -> WindowParams {
        WindowParams {
            size: (1000., 600.).into(),
            scale: 1.,
            position: None,
            resizable: true,
            title: "Window".into(),
        }
    }

    fn default_subwindow_params(&self) -> SubwindowParams {
        SubwindowParams {
            size: (200., 100.).into(),
            scale: 1.,
            position: (0., 0.).into(),
        }
    }

    fn handle_window_event(&mut self, window: WindowId, event: WindowEvent) -> AppResponce;

    fn handle_window_update(&mut self, id: WindowId, window: W) -> AppResponce;

    fn handle_subwindow_update(&mut self, id: WindowId, window: SW) -> AppResponce;
}
