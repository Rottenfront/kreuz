use kreuz_ui::Scene;
use kurbo::RoundedRect;

use super::*;

pub struct DocumentModel {
    arena: Arena,
    context: Context,
    root: ViewId,
}

impl DocumentModel {
    pub fn draw(&mut self, drawer: &mut Scene) {
        let mut region_calc = RegionCalc::new(std::mem::take(&mut self.arena));
        region_calc.compute_regions(
            self.root,
            RoundedRect::from_origin_size((0.0, 0.0), self.context.window_size, 0.0),
        );
        let _ = std::mem::replace(&mut self.arena, region_calc.destroy());

        let mut draw_ctx = DrawCtx::new(drawer, &mut self.context, &mut self.arena);
        draw_ctx.draw(self.root);
    }

    pub fn process_event(&mut self, event: Event) {}
}
