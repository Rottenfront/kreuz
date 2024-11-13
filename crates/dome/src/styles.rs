use kurbo::Size;

use super::Brush;

#[derive(Default, Clone)]
pub struct Styles {
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub padding: Padding,
    pub borders: Borders,
    pub background: Brush,
    pub clip: bool,
    pub size: SizeStyles,
}

#[derive(Default, Clone, Copy)]
pub struct SizeStyles {
    pub zip: bool,
    pub max_size: SizeConstraints,
    pub min_size: SizeConstraints,
}

#[derive(Default, Clone, Copy)]
pub struct SizeConstraints {
    pub width: Option<f64>,
    pub height: Option<f64>,
}

pub fn min_options(lhs: Option<f64>, rhs: Option<f64>) -> Option<f64> {
    match lhs {
        None => rhs,
        Some(lhs) => match rhs {
            None => Some(lhs),
            Some(rhs) => Some(min_f64(lhs, rhs)),
        },
    }
}

fn min_f64(lhs: f64, rhs: f64) -> f64 {
    if lhs < rhs {
        lhs
    } else {
        rhs
    }
}

pub fn max_options(lhs: Option<f64>, rhs: Option<f64>) -> Option<f64> {
    match lhs {
        None => rhs,
        Some(lhs) => match rhs {
            None => Some(lhs),
            Some(rhs) => Some(max_f64(lhs, rhs)),
        },
    }
}

fn max_f64(lhs: f64, rhs: f64) -> f64 {
    if lhs > rhs {
        lhs
    } else {
        rhs
    }
}

pub fn sum_options(lhs: Option<f64>, rhs: Option<f64>) -> Option<f64> {
    match lhs {
        None => rhs,
        Some(lhs) => match rhs {
            None => Some(lhs),
            Some(rhs) => Some(lhs + rhs),
        },
    }
}

impl SizeConstraints {
    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: min_options(self.width, rhs.width),
            height: min_options(self.height, rhs.height),
        }
    }

    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: max_options(self.width, rhs.width),
            height: max_options(self.height, rhs.height),
        }
    }

    pub fn apply_max(self, size: Size) -> Size {
        Size::new(
            if let Some(width) = self.width {
                min_f64(width, size.width)
            } else {
                size.width
            },
            if let Some(height) = self.height {
                min_f64(height, size.height)
            } else {
                size.height
            },
        )
    }

    pub fn apply_min(self, size: Size) -> Size {
        Size::new(
            if let Some(width) = self.width {
                max_f64(width, size.width)
            } else {
                size.width
            },
            if let Some(height) = self.height {
                max_f64(height, size.height)
            } else {
                size.height
            },
        )
    }
}

impl From<Size> for SizeConstraints {
    fn from(value: Size) -> Self {
        SizeConstraints {
            width: Some(value.width),
            height: Some(value.height),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default, Clone, Copy)]
pub enum VAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Default, Clone, Copy)]
pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Default, Clone)]
pub struct Borders {
    pub width: f64,
    pub radius: f64,
    pub paint: Brush,
    pub disabled: BordersMode,
}

#[derive(Clone, Copy)]
pub struct BordersMode {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
    pub lt: bool,
    pub rt: bool,
    pub rb: bool,
    pub lb: bool,
}

impl Default for BordersMode {
    fn default() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
            lt: false,
            rt: false,
            rb: false,
            lb: false,
        }
    }
}
