// use kreuz_ui::{
//     skrifa::{
//         self,
//         charmap::Charmap,
//         instance::Location,
//         metrics::{GlyphMetrics, Metrics},
//         prelude::Size as SkrifaSize,
//         raw::FileRef,
//         FontRef, GlyphId, MetadataProvider,
//     },
//     Glyph, Scene,
// };
// use kurbo::Affine;
// use peniko::{Blob, Brush, BrushRef, Color, Fill, Font, StyleRef};
// use std::sync::Arc;

// // This is very much a hack to get things working.
// // On Windows, can set this to "c:\\Windows\\Fonts\\seguiemj.ttf" to get color emoji
// const ROBOTO_FONT: &[u8] = include_bytes!("../assets/roboto/Roboto-Regular.ttf");
// const INCONSOLATA_FONT: &[u8] = include_bytes!("../assets/inconsolata/Inconsolata.ttf");
// const NOTO_EMOJI_CBTF_SUBSET: &[u8] =
//     include_bytes!("../assets/noto_color_emoji/NotoColorEmoji-CBTF-Subset.ttf");
// const NOTO_EMOJI_COLR_SUBSET: &[u8] =
//     include_bytes!("../assets/noto_color_emoji/NotoColorEmoji-Subset.ttf");

pub struct SimpleText {
    // roboto: Font,
    // inconsolata: Font,
    // noto_emoji_colr_subset: Font,
    // noto_emoji_cbtf_subset: Font,
}
/*
pub struct TextRunner<'a> {
    font: &'a Font,
    size: f32,
    glyph_transform: Option<Affine>,
    var_loc: Arc<Location>,
    charmap: Charmap<'a>,
    line_height: f32,
    glyph_metrics: GlyphMetrics<'a>,
}

pub struct GlyphData {
    pub gid: GlyphId,
    pub width: f32,
}

impl<'a> TextRunner<'a> {
    fn new(
        font: &'a Font,
        size: f32,
        variations: &[(&str, f32)],
        glyph_transform: Option<Affine>,
    ) -> Self {
        let font_ref = to_font_ref(font).unwrap();
        let axes = font_ref.axes();
        let font_size = SkrifaSize::new(size);
        let var_loc = Arc::new(axes.location(variations.iter().copied()));
        let charmap = font_ref.charmap();
        // TODO: dumb borrow checker, fix that later
        let metrics = Metrics::new(&font_ref, font_size, unsafe {
            std::mem::transmute::<_, &'static Location>(var_loc.as_ref())
        });
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font_ref.glyph_metrics(font_size, unsafe {
            std::mem::transmute::<_, &'static Location>(var_loc.as_ref())
        });
        Self {
            font,
            size,
            glyph_transform,
            charmap,
            line_height,
            glyph_metrics,
            var_loc,
        }
    }

    pub fn get_line_height(&self) -> f32 {
        self.line_height
    }

    pub fn get_char_data(&self, ch: char) -> GlyphData {
        let gid = self.charmap.map(ch).unwrap_or_default();
        GlyphData {
            gid,
            width: self.glyph_metrics.advance_width(gid).unwrap_or_default(),
        }
    }

    pub fn get_word_width(&self, word: &str) -> f32 {
        word.chars()
            .map(|ch| self.get_char_data(ch).width)
            .fold(0.0, |res, width| res + width)
    }

    pub fn draw_word<'b>(
        &self,
        scene: &mut Scene,
        brush: impl Into<BrushRef<'b>>,
        style: impl Into<StyleRef<'b>>,
        transform: Affine,
        word: &str,
    ) -> f32 {
        let mut pen_x = 0.0;
        // scene
        //     .draw_glyphs(self.font)
        //     .font_size(self.size)
        //     .transform(transform)
        //     .glyph_transform(self.glyph_transform)
        //     .normalized_coords(self.var_loc.coords())
        //     .brush(brush.into())
        //     .hint(false)
        //     .draw(
        //         style.into(),
        //         word.chars().map(|ch| {
        //             let GlyphData { gid, width } = self.get_char_data(ch);
        //             let x = pen_x;
        //             pen_x += width;
        //             Glyph {
        //                 id: gid.to_u32(),
        //                 x,
        //                 y: 0.0,
        //             }
        //         }),
        //     );
        pen_x
    }
}

impl SimpleText {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            roboto: Font::new(Blob::new(Arc::new(ROBOTO_FONT)), 0),
            inconsolata: Font::new(Blob::new(Arc::new(INCONSOLATA_FONT)), 0),
            noto_emoji_colr_subset: Font::new(Blob::new(Arc::new(NOTO_EMOJI_COLR_SUBSET)), 0),
            noto_emoji_cbtf_subset: Font::new(Blob::new(Arc::new(NOTO_EMOJI_CBTF_SUBSET)), 0),
        }
    }

    /// Add a text run which supports some emoji.
    ///
    /// The supported Emoji are ✅, 👀, 🎉, and 🤠.
    /// This subset is chosen to demonstrate the emoji support, whilst
    /// not significantly increasing repository size.
    ///
    /// Note that Vello does support COLR emoji, but does not currently support
    /// any other forms of emoji.
    #[allow(clippy::too_many_arguments)]
    pub fn add_colr_emoji_run<'a>(
        &mut self,
        scene: &mut Scene,
        size: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let font = self.noto_emoji_colr_subset.clone();
        self.add_var_run(
            scene,
            Some(&font),
            size,
            &[],
            // This should be unused
            &Brush::Solid(Color::WHITE),
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    /// Add a text run which supports some emoji.
    ///
    /// The supported Emoji are ✅, 👀, 🎉, and 🤠.
    /// This subset is chosen to demonstrate the emoji support, whilst
    /// not significantly increasing repository size.
    ///
    /// This will use a CBTF font, which Vello supports.
    #[allow(clippy::too_many_arguments)]
    pub fn add_bitmap_emoji_run<'a>(
        &mut self,
        scene: &mut Scene,
        size: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let font = self.noto_emoji_cbtf_subset.clone();
        self.add_var_run(
            scene,
            Some(&font),
            size,
            &[],
            // This should be unused
            &Brush::Solid(Color::WHITE),
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_run<'a>(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        self.add_var_run(
            scene,
            font,
            size,
            &[],
            brush,
            transform,
            glyph_transform,
            style,
            text,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_var_run<'a>(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        variations: &[(&str, f32)],
        brush: impl Into<BrushRef<'a>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'a>>,
        text: &str,
    ) {
        let default_font = if variations.is_empty() {
            &self.roboto
        } else {
            &self.inconsolata
        };
        let font = font.unwrap_or(default_font);
        let font_ref = to_font_ref(font).unwrap();
        let brush = brush.into();
        let style = style.into();
        let axes = font_ref.axes();
        let font_size = skrifa::instance::Size::new(size);
        let var_loc = axes.location(variations.iter().copied());
        let charmap = font_ref.charmap();
        let metrics = font_ref.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);
        let mut pen_x = 0f32;
        let mut pen_y = 0f32;
        scene
            .draw_glyphs(font)
            .font_size(size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .normalized_coords(var_loc.coords())
            .brush(brush)
            .hint(false)
            .draw(
                style,
                text.chars().filter_map(|ch| {
                    if ch == '\n' {
                        pen_y += line_height;
                        pen_x = 0.0;
                        return None;
                    }
                    let gid = charmap.map(ch).unwrap_or_default();
                    let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                    let x = pen_x;
                    pen_x += advance;
                    Some(Glyph {
                        id: gid.to_u32(),
                        x,
                        y: pen_y,
                    })
                }),
            );
    }

    pub fn make_font_run(&self, size: f32, glyph_transform: Option<Affine>) -> TextRunner {
        TextRunner::new(&self.inconsolata, size, &[], glyph_transform)
    }

    pub fn add(
        &mut self,
        scene: &mut Scene,
        font: Option<&Font>,
        size: f32,
        brush: Option<&Brush>,
        transform: Affine,
        text: &str,
    ) {
        let brush = brush.unwrap_or(&Brush::Solid(Color::WHITE));
        self.add_run(
            scene,
            font,
            size,
            brush,
            transform,
            None,
            Fill::NonZero,
            text,
        );
    }
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    let file_ref = FileRef::new(font.data.as_ref()).ok()?;
    match file_ref {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}
*/
