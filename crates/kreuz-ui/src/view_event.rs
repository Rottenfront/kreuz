use peniko::kurbo::Point;

#[derive(Debug, Clone)]
pub enum ViewEvent {
    CursorEntered,
    CursorLeft,

    CursorMove { pos: Point },

    MouseButtonPress { pos: Point, button: MouseButton },

    MouseButtonRelease { pos: Point, button: MouseButton },
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
