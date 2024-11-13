use super::*;

pub enum ViewResponce {
    Skipped,
    Handled,
}

pub trait RootView {
    fn render(&self, scene: &mut Scene);

    fn handle_event(&mut self, event: &ViewEvent) -> ViewResponce;
}
