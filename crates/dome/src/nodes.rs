use kurbo::{RoundedRectRadii, Vec2};
use std::collections::HashMap;
use std::sync::Mutex;

use super::*;

/// Event Handler ID
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EHId(usize);

static LAST_EH_ID: Mutex<usize> = Mutex::new(0);

fn get_new_eh_id() -> EHId {
    let mut last_id = LAST_EH_ID.lock().unwrap();
    let id = *last_id;
    *last_id += 1;
    EHId(id)
}

pub struct UiNode {
    pub entity: Entity,
    pub styles: Styles,
    pub event_handlers: HashMap<EHId, EventHandler>,
    free_ids: Vec<EHId>,
}

impl UiNode {
    pub fn assign_event_handler(&mut self, handler: EventHandler) -> EHId {
        let id = *self.free_ids.last().unwrap_or(&get_new_eh_id());
        self.event_handlers.insert(id, handler);
        id
    }

    pub fn get_event_handler(&self, id: EHId) -> Option<&EventHandler> {
        self.event_handlers.get(&id)
    }

    pub fn take_event_handler(&mut self, id: EHId) -> Option<EventHandler> {
        if let Some(handler) = self.event_handlers.remove(&id) {
            self.free_ids.push(id);
            Some(handler)
        } else {
            None
        }
    }
}

pub enum Entity {
    Box(BoxEntity),
    Stack(StackEntity),
    Scroll(ScrollEntity),
    Switch(SwitchEntity),
    Scale(ScaleEntity),
    Image(ImageEntity),
    Rect(RectEntity),
    Text(TextEntity),
    Paragraph(ParagraphEntity),
    Canvas(CanvasEntity),
}

/// We should somehow cache the bounds that entity inside the scrollable used for drawing
/// We can use it for normal scrolling, to check that the content reach the end and we should not scroll down
pub struct ScrollEntity {
    pub inner: ViewId,
    pub offset: Vec2,
    pub v_enabled: bool,
    pub h_enabled: bool,
    pub enable_inner_min_size: bool,
}

pub struct ScaleEntity {
    pub inner: ViewId,
    pub scale: f64,
}

pub struct BoxEntity {
    pub inner: ViewId,
}

pub struct StackEntity {
    pub direction: StackDirection,
    pub inner: Vec<ViewId>,
    pub padding: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StackDirection {
    X,
    Y,
    Z,
}

pub struct SwitchEntity {
    pub mode: usize,
    pub inner: Vec<ViewId>,
}

pub struct RectEntity {
    pub paint: Brush,
    pub radii: RoundedRectRadii,
}

pub struct ImageEntity {
    pub image: Image,
}

pub struct TextEntity {
    pub text: String,
    pub styles: TextStyles,
}

pub struct ParagraphEntity {
    pub text: String,
    pub styles: TextStyles,
    pub extended_styles: ParagraphStyles,
}

#[derive(Clone)]
pub struct TextStyles {
    pub color: Color,
    pub font_family: String,
    pub size: f64,
}

#[derive(Clone, Copy)]
pub struct ParagraphStyles {
    pub offset: f64,
    pub align: TextAlign,
    pub v_align: VAlign,
}

#[derive(Clone, Copy)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

pub struct CanvasEntity {
    pub draw: Box<dyn Fn(&mut DrawCtx, Region)>,
}
