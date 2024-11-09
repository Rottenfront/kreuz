use peniko::kurbo::{Point, Size};

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resize {
        new_size: Size,
    },

    Redraw,

    CursorEntered,
    CursorLeft,

    CursorMove {
        pos: Point,
    },

    MouseButton {
        button: MouseButton,
        state: ButtonState,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}
