use crate::*;

pub struct Style {
    pub window_bg: ColorRGBA,
    pub widget_bg: ColorRGBA,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale: f32,
}

impl Default for Style {

    fn default() -> Self {
        Self {
            window_bg: ColorRGBA::from_rgba(0.1, 0.1, 0.1, 1.0),
            widget_bg: ColorRGBA::from_rgba(0.43, 0.43, 0.43, 1.0),
            item_pad_outer: vec2(0.05, 0.05),
            item_pad_inner: vec2(0.01, 0.01),
            font_scale: 0.1,
        }
    }
}

impl Style {

    pub fn calc_item_height(&self) -> f32 {
        self.item_pad_inner.y * 2.0 + self.font_scale
    }
}
