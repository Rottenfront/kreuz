use std::sync::{atomic::AtomicUsize, Mutex};

use super::*;
use peniko::kurbo::{Point, Size};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct WindowId(usize);

static mut LAST_ID: Mutex<usize> = Mutex::new(0);

impl WindowId {
    pub fn new() -> WindowId {
        let mut last_id = unsafe { LAST_ID.lock() }.unwrap();
        let id = WindowId(*last_id);
        *last_id += 1;
        id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WindowParams {
    pub size: Size,
    pub scale: f64,
    /// Position from (0, 0) of parent window's surface
    pub position: Option<Point>,
    pub resizable: bool,
    pub title: String,
}

pub trait WindowHandler: Send + Sync + HasWindowHandle + HasDisplayHandle {
    fn request_redraw(&self);

    fn set_title(&self, title: &str);

    fn get_params(&self) -> WindowParams;
}

#[derive(Clone, PartialEq, Debug)]
pub struct SubwindowParams {
    pub size: Size,
    pub scale: f64,
    /// Position from (0, 0) of parent window's surface
    pub position: Point,
}

pub trait SubwindowHandler: Send + Sync + HasWindowHandle + HasDisplayHandle {
    fn request_redraw(&self);

    fn get_params(&self) -> SubwindowParams;
}
