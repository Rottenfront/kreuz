use std::collections::HashMap;

use kreuz_ui::Scene;
use kurbo::Rect;

use super::*;

pub struct DocumentModel {
    arena: Arena,
    context: Context,
    text: SimpleText,
    root: ViewId,
}

impl DocumentModel {
    pub fn draw(&mut self, drawer: &mut Scene) {
        let mut region_calc = RegionCalc {
            arena: &mut self.arena,
            text: &mut self.text,
            calc_data: HashMap::new(),
        };
        region_calc.compute_regions(
            self.root,
            Rect::from_origin_size((0.0, 0.0), self.context.window_size),
        );
        drop(region_calc);

        let mut draw_ctx = DrawCtx::new(drawer, &mut self.context, &mut self.arena, &mut self.text);
        draw_ctx.draw(self.root);
    }

    pub fn process_event(&mut self, _event: Event) {}
}
