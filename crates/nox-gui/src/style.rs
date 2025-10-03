use crate::*;

pub struct Style<FontHash> {
    pub window_bg_col: ColorRGBA,
    pub widget_bg_col: ColorRGBA,
    pub text_col: ColorRGBA,
    pub font_regular: FontHash,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale: f32,
    pub rounding: f32,
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
            font_scale: 0.03,
            rounding: 0.01,
        }
    }

    pub fn calc_item_height(&self, text_height: f32) -> f32 {
        self.item_pad_inner.y * 2.0 + text_height * self.font_scale
    }

    pub fn calc_text_box_width(&self, text_width: f32) -> f32 {
        self.item_pad_inner.x * 2.0 + text_width * self.font_scale
    }
}
