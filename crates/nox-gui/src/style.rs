use crate::*;

pub struct Style<FontHash> {
    pub window_bg_col: ColorRGBA,
    pub widget_bg_col: ColorRGBA,
    pub text_col: ColorRGBA,
    pub font_regular: FontHash,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale: f32,
}

impl<FontHash> Style<FontHash> {

    pub fn new(font_regular: FontHash) -> Self {
        Self {
            window_bg_col: ColorRGBA::from_rgba(0.1, 0.1, 0.1, 1.0),
            widget_bg_col: ColorRGBA::from_rgba(0.43, 0.43, 0.43, 1.0),
            text_col: ColorRGBA::white(),
            font_regular,
            item_pad_outer: vec2(0.05, 0.05),
            item_pad_inner: vec2(0.01, 0.01),
            font_scale: 0.1,
        }
    }

    pub fn calc_item_height(&self) -> f32 {
        self.item_pad_inner.y * 2.0 + self.font_scale
    }
}
