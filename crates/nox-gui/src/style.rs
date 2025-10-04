use crate::*;

use nox_geom::*;

pub struct Style<FontHash> {
    pub window_bg_col: ColorRGBA,
    pub widget_bg_col: ColorRGBA,
    pub text_col: ColorRGBA,
    pub font_regular: FontHash,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale_regular: f32,
    pub font_scale_header: f32,
    pub rounding: f32,
    pub slider_min_width: f32,
}

impl<FontHash> Style<FontHash> {

    pub fn new(font_regular: FontHash) -> Self {
        Self {
            window_bg_col: ColorRGBA::from_rgba(0.1, 0.1, 0.1, 1.0),
            widget_bg_col: ColorRGBA::from_rgba(0.43, 0.43, 0.43, 1.0),
            text_col: ColorRGBA::white(),
            font_regular,
            item_pad_outer: vec2(0.02, 0.02),
            item_pad_inner: vec2(0.01, 0.01),
            font_scale_regular: 0.03,
            font_scale_header: 0.04,
            rounding: 0.01,
            slider_min_width: 0.05,
        }
    }

    #[inline(always)]
    pub fn calc_text_size_regular(&self, text_size: Vec2) -> Vec2 {
        text_size * self.font_scale_regular
    }

    #[inline(always)]
    pub fn calc_text_width_regular(&self, text_width: f32) -> f32 {
        text_width * self.font_scale_regular
    }

    #[inline(always)]
    pub fn calc_text_height_regular(&self, text_height: f32) -> f32 {
        text_height * self.font_scale_regular
    }

    #[inline(always)]
    pub fn calc_text_size_header(&self, text_size: Vec2) -> Vec2 {
        text_size * self.font_scale_header
    }

    #[inline(always)]
    pub fn calc_text_height_header(&self, text_height: f32) -> f32 {
        text_height * self.font_scale_header
    }

    #[inline(always)]
    pub fn calc_text_width_header(&self, text_width: f32) -> f32 {
        text_width * self.font_scale_header
    }

    #[inline(always)]
    pub fn calc_text_box_size(&self, text_size: Vec2) -> Vec2 {
        self.item_pad_inner * 2.0 + text_size * self.font_scale_regular
    } 

    #[inline(always)]
    pub fn calc_text_box_height(&self, text_height: f32) -> f32 {
        self.item_pad_inner.y * 2.0 + text_height * self.font_scale_regular
    }

    #[inline(always)]
    pub fn calc_text_box_width(&self, text_width: f32) -> f32 {
        self.item_pad_inner.x * 2.0 + text_width * self.font_scale_regular
    } 
}
