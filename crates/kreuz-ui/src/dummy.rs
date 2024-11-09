use peniko::Color;
use vello::kurbo::{Affine, Rect};

use super::*;

pub struct DummyView;

impl View for DummyView {
    fn render(&self, scene: &mut Scene) {
        scene.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::WHITE,
            None,
            &Rect::new(0.0, 0.0, 100., 100.),
        );
    }

    fn handle_event(&mut self, event: &ViewEvent) -> ViewResponce {
        println!("{event:?}");
        ViewResponce::Skipped
    }
}
