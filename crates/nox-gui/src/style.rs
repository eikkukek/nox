use crate::*;

pub struct Style {
    pub widget_bg: ColorRGBA,
    pub widget_bg_hl: ColorRGBA,
}

impl Default for Style {

    fn default() -> Self {
        Self {
            widget_bg: ColorRGBA::from_rgba(0.1, 0.1, 0.1, 1.0),
            widget_bg_hl: ColorRGBA::from_rgba(0.43, 0.43, 0.43, 1.0),
        }
    }
}
