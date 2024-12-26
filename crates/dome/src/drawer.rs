use std::f64::consts::PI;

use kreuz_ui::Scene;
use kurbo::{Affine, Arc, Line, RoundedRect, Size, Stroke};
use peniko::{BlendMode, BrushRef};

use super::*;

pub struct Context {
    pub window_size: Size,
}

pub struct DrawCtx<'a, 'b, 'c> {
    drawer: &'a mut Scene,
    context: &'b mut Context,
    arena: &'c Arena,
}

impl<'a, 'b, 'c> DrawCtx<'a, 'b, 'c> {
    pub fn new(drawer: &'a mut Scene, context: &'b mut Context, arena: &'c Arena) -> Self {
        Self {
            drawer,
            context,
            arena,
        }
    }

    pub fn draw(&mut self, root: ViewId) {
        let node = self.arena.get_view(root).unwrap();
        let region = self.arena.get_relative_region(root).unwrap();
        let region = self.draw_decorations(&node.styles, region);

        match &node.entity {
            Entity::Box(entity) => self.draw_box(&entity),
            Entity::Stack(entity) => self.draw_stack(&entity),
            Entity::Scroll(entity) => self.draw_scrollable(&entity, region),
            Entity::Switch(entity) => self.draw_switchable(&entity),
            Entity::Image(entity) => self.draw_image(&entity, region),
            Entity::Rect(entity) => self.draw_rect(&entity, region),
            Entity::Text(entity) => self.draw_text(&entity, region),
            Entity::Paragraph(entity) => self.draw_paragraph(&entity, region),
            Entity::Canvas(entity) => self.draw_canvas(&entity, region),
            Entity::Scale(entity) => self.draw_scale(&entity),
        }
    }

    fn draw_decorations(&mut self, styles: &Styles, region: Region) -> Region {
        let mut region = self.draw_borders(&styles.borders, region);

        self.draw_background(&styles.background, styles.borders.radius, region);

        let padding = styles.padding;
        let mut rect = region.rect();
        rect.x0 += padding.left;
        rect.y0 += padding.top;
        rect.x1 -= padding.right;
        rect.y1 -= padding.bottom;
        region = RoundedRect::from_rect(rect, region.radii());

        region
    }

    /// Returns size of remaining region
    fn draw_borders(&mut self, borders: &Borders, mut region: Region) -> Region {
        let Borders {
            width,
            radius,
            paint,
            disabled,
        } = borders;

        let width = *width;
        let radius = *radius;
        let mut rect = region.rect();

        if !disabled.left && !disabled.lt && !disabled.lb {
            rect.x0 += borders.width / 2.0;
        }
        if !disabled.right && !disabled.rt && !disabled.rb {
            rect.x1 -= borders.width / 2.0;
        }
        if !disabled.top && !disabled.lt && !disabled.rt {
            rect.y0 += borders.width / 2.0;
        }
        if !disabled.bottom && !disabled.lb && !disabled.rb {
            rect.y1 -= borders.width / 2.0;
        }

        macro_rules! draw_line {
            ($orientation:ident, $x0:expr, $y0:expr, $x1:expr, $y1:expr) => {
                if !disabled.$orientation {
                    self.drawer.stroke(
                        &Stroke::new(width),
                        Affine::IDENTITY,
                        paint,
                        None,
                        &Line::new(($x0, $y0), ($x1, $y1)),
                    );
                }
            };
        }

        draw_line!(left, rect.x0, rect.y0 + radius, rect.x0, rect.y1 - radius);
        draw_line!(right, rect.x1, rect.y0 + radius, rect.x1, rect.y1 - radius);
        draw_line!(top, rect.x0 + radius, rect.y0, rect.x1 - radius, rect.y0);
        draw_line!(bottom, rect.x0 + radius, rect.y1, rect.x1 - radius, rect.y1);

        macro_rules! draw_arc {
            ($orientation:ident, $x0:expr, $y0:expr, $angle:expr) => {
                if !disabled.$orientation {
                    self.drawer.stroke(
                        &Stroke::new(width),
                        Affine::IDENTITY,
                        paint,
                        None,
                        &Arc::new(
                            (rect.x0 + radius, rect.y0 + radius),
                            (radius, radius),
                            0.0,
                            PI / 2.0,
                            $angle,
                        ),
                    );
                }
            };
        }

        draw_arc!(lt, region.x0 + radius, region.y0 + radius, PI * 1.5);
        draw_arc!(rt, region.x1 - radius, region.y0 + radius, 0.0);
        draw_arc!(lb, region.x0 + radius, region.y1 - radius, PI);
        draw_arc!(rb, region.x1 - radius, region.y1 - radius, PI / 2.0);

        if !disabled.left && !disabled.lt && !disabled.lb {
            rect.x0 += borders.width / 2.0;
        }
        if !disabled.right && !disabled.rt && !disabled.rb {
            rect.x1 -= borders.width / 2.0;
        }
        if !disabled.top && !disabled.lt && !disabled.rt {
            rect.y0 += borders.width / 2.0;
        }
        if !disabled.bottom && !disabled.lb && !disabled.rb {
            rect.y1 -= borders.width / 2.0;
        }
        region = RoundedRect::from_rect(rect, region.radii());

        region
    }

    fn draw_background<'d>(
        &mut self,
        background: impl Into<BrushRef<'d>>,
        border_radius: f64,
        region: Region,
    ) {
        self.drawer.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            background,
            None,
            &region.rect().to_rounded_rect(border_radius),
        );
    }

    fn draw_box(&mut self, entity: &BoxEntity) {
        let id = entity.inner.clone();
        self.draw(id);
    }

    fn draw_stack(&mut self, entity: &StackEntity) {
        let ids = entity.inner.clone();
        for id in ids {
            self.draw(id);
        }
    }

    fn draw_scrollable(&mut self, entity: &ScrollEntity, region: Region) {
        let id = entity.inner.clone();
        self.drawer
            .push_layer(peniko::Mix::Clip, 1.0, Affine::IDENTITY, &region);
        self.draw(id);
        self.drawer.pop_layer();
    }

    fn draw_switchable(&mut self, entity: &SwitchEntity) {
        let id = entity.inner[entity.mode].clone();
        self.draw(id);
    }

    fn draw_image(&mut self, entity: &ImageEntity, region: Region) {
        let image = &entity.image;
        let affine = Affine::translate(region.origin().to_vec2()).then_scale_non_uniform(
            image.width as f64 / region.width(),
            image.height as f64 / region.height(),
        );
        self.drawer.draw_image(&entity.image, affine);
    }

    fn draw_rect(&mut self, entity: &RectEntity, region: Region) {
        self.drawer.fill(
            Fill::EvenOdd,
            Affine::IDENTITY,
            &entity.paint,
            None,
            &region.rect().to_rounded_rect(entity.radii),
        );
    }

    fn draw_text(&mut self, entity: &TextEntity, region: Region) {
        // let layout = self
        //     .drawer
        //     .text()
        //     .new_text_layout(&entity.text)
        //     .default_attribute(TextAttribute::FontFamily(entity.styles.font_family))
        //     .default_attribute(TextAttribute::FontSize(entity.styles.size))
        //     .max_width(region.width())
        //     .build();
        // let layout = if let Some(layout) = layout {
        //     layout
        // } else {
        //     return;
        // };
        // self.drawer.draw_text(&layout, region.origin());
    }

    fn draw_paragraph(&mut self, entity: &ParagraphEntity, region: Region) {
        // let layout = self
        //     .drawer
        //     .text()
        //     .new_text_layout(&entity.text)
        //     .default_attribute(TextAttribute::FontFamily(entity.styles.font_family))
        //     .default_attribute(TextAttribute::FontSize(entity.styles.size))
        //     .max_width(region.width())
        //     .build();
        // let layout = if let Some(layout) = layout {
        //     layout
        // } else {
        //     return;
        // };
        // self.drawer.draw_text(&layout, region.origin());
    }

    fn draw_canvas(&mut self, entity: &CanvasEntity, region: Region) {
        (entity.draw)(self, region);
    }

    fn draw_scale(&mut self, entity: &ScaleEntity) {
        let id = entity.inner.clone();
        let scale = entity.scale.clone();
        self.drawer.push_layer(BlendMode::new(
            peniko::Mix::Normal,
            peniko::Compose::DestAtop,
        ));
        self.drawer.transform(Affine::scale(scale));
        self.draw(id);
        self.drawer.restore();
    }
}
