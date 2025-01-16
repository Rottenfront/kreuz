use kurbo::{Affine, Rect};
use peniko::Color;

use super::*;

pub struct DummyView;

impl RootView for DummyView {
    fn render(&self, scene: &mut Scene) {}

    fn handle_event(&mut self, event: &ViewEvent) -> ViewResponce {
        println!("{event:?}");
        ViewResponce::Skipped
    }
}
