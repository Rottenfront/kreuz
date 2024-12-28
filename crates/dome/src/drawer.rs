use std::f64::consts::PI;

use kreuz_ui::Scene;
use kurbo::{Affine, Arc, Line, Size, Stroke};
use peniko::{BlendMode, BrushRef, Style};

use super::*;

pub struct Context {
    pub window_size: Size,
}

pub struct DrawCtx<'a, 'b, 'c, 'd> {
    drawer: &'a mut Scene,
    _context: &'b mut Context,
    arena: &'c Arena,
    text: &'d SimpleText,
}

impl<'a, 'b, 'c, 'd> DrawCtx<'a, 'b, 'c, 'd> {
    pub fn new(
        drawer: &'a mut Scene,
        context: &'b mut Context,
        arena: &'c Arena,
        text: &'d mut SimpleText,
    ) -> Self {
        Self {
            drawer,
            _context: context,
            arena,
            text,
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
            Entity::Scale(entity) => self.draw_scale(&entity, region),
        }
    }

    fn draw_decorations(&mut self, styles: &Styles, region: Region) -> Region {
        let mut region = self.draw_borders(&styles.borders, region);

        self.draw_background(&styles.background, styles.borders.radius, region);

        let padding = styles.padding;
        region.x0 += padding.left;
        region.y0 += padding.top;
        region.x1 -= padding.right;
        region.y1 -= padding.bottom;

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

        if !disabled.left && !disabled.lt && !disabled.lb {
            region.x0 += borders.width / 2.0;
        }
        if !disabled.right && !disabled.rt && !disabled.rb {
            region.x1 -= borders.width / 2.0;
        }
        if !disabled.top && !disabled.lt && !disabled.rt {
            region.y0 += borders.width / 2.0;
        }
        if !disabled.bottom && !disabled.lb && !disabled.rb {
            region.y1 -= borders.width / 2.0;
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

        draw_line!(
            left,
            region.x0,
            region.y0 + radius,
            region.x0,
            region.y1 - radius
        );
        draw_line!(
            right,
            region.x1,
            region.y0 + radius,
            region.x1,
            region.y1 - radius
        );
        draw_line!(
            top,
            region.x0 + radius,
            region.y0,
            region.x1 - radius,
            region.y0
        );
        draw_line!(
            bottom,
            region.x0 + radius,
            region.y1,
            region.x1 - radius,
            region.y1
        );

        macro_rules! draw_arc {
            ($orientation:ident, $x0:expr, $y0:expr, $angle:expr) => {
                if !disabled.$orientation {
                    self.drawer.stroke(
                        &Stroke::new(width),
                        Affine::IDENTITY,
                        paint,
                        None,
                        &Arc::new(
                            (region.x0 + radius, region.y0 + radius),
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
            region.x0 += borders.width / 2.0;
        }
        if !disabled.right && !disabled.rt && !disabled.rb {
            region.x1 -= borders.width / 2.0;
        }
        if !disabled.top && !disabled.lt && !disabled.rt {
            region.y0 += borders.width / 2.0;
        }
        if !disabled.bottom && !disabled.lb && !disabled.rb {
            region.y1 -= borders.width / 2.0;
        }

        region
    }

    fn draw_background<'e>(
        &mut self,
        background: impl Into<BrushRef<'e>>,
        border_radius: f64,
        region: Region,
    ) {
        self.drawer.fill(
            peniko::Fill::NonZero,
            Affine::IDENTITY,
            background,
            None,
            &region.to_rounded_rect(border_radius),
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
            &region.to_rounded_rect(entity.radii),
        );
    }

    fn draw_text(&mut self, entity: &TextEntity, region: Region) {
        let text_run = self.text.make_font_run(entity.styles.size as _, None);
        let (mut x, y) = region.origin().into();
        let space_width = text_run.get_char_data(' ').width;
        entity.text.split_whitespace().for_each(|word| {
            x += text_run.draw_word(
                &mut self.drawer,
                &entity.styles.color,
                &Style::Fill(Fill::NonZero),
                Affine::translate((x, y)),
                word,
            ) as f64
                + space_width as f64;
        });
    }

    fn draw_paragraph(&mut self, entity: &ParagraphEntity, region: Region) {
        let text_run = self.text.make_font_run(entity.styles.size as _, None);
        let (mut x, mut y) = region.origin().into();
        let line_height = text_run.get_line_height() as f64;
        let space_width = text_run.get_char_data(' ').width as f64;
        entity.text.split_whitespace().for_each(|word| {
            if x + text_run.get_word_width(word) as f64 >= region.x1 {
                x = region.origin().x;
                y += line_height;
            }
            x += text_run.draw_word(
                &mut self.drawer,
                &entity.styles.color,
                &Style::Fill(Fill::NonZero),
                Affine::translate((x, y)),
                word,
            ) as f64
                + space_width;
        });
    }

    fn draw_canvas(&mut self, entity: &CanvasEntity, region: Region) {
        (entity.draw)(self, region);
    }

    fn draw_scale(&mut self, entity: &ScaleEntity, region: Region) {
        let id = entity.inner.clone();
        let scale = entity.scale.clone();
        self.drawer.push_layer(
            BlendMode::new(peniko::Mix::Normal, peniko::Compose::DestAtop),
            1.0,
            Affine::scale(scale),
            &region,
        );
        self.draw(id);
        self.drawer.pop_layer();
    }
}
