use kurbo::Size;

use crate::Context;

pub enum Event {
    Update,
    Resize(Size),
}

pub enum EventHandler {
    /// Update event will be sent to every widget by the DOM, there is no need to call the handler
    /// of children.
    Update(Box<dyn Fn(&mut Context)>),
}
