use core::fmt::Write;

use crate::*;

use nox_geom::*;

pub struct Style<FontHash> {
    pub window_bg_col: ColorRGBA,
    pub window_title_bar_col: ColorRGBA,
    pub widget_bg_col: ColorRGBA,
    pub outline_col: ColorRGBA,
    pub outline_col_hl: ColorRGBA,
    pub outline_thin_col: ColorRGBA,
    pub separator_col: ColorRGBA,
    pub handle_col: ColorRGBA,
    pub text_col: ColorRGBA,
    pub hover_window_bg_col: ColorRGBA,
    pub font_regular: FontHash,
    pub checkbox_symbol: char,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale: f32,
    pub rounding: f32,
    pub slider_min_width: f32,
    pub outline_width: f32,
    pub outline_thin_width: f32,
    pub separator_height: f32,
    pub override_cursor: bool,
    pub f32_format: fn(f32, &mut CompactString) -> core::fmt::Result,
    pub f64_format: fn(f64, &mut CompactString) -> core::fmt::Result,
}

impl<FontHash> Style<FontHash> {

    pub fn new(font_regular: FontHash) -> Self {
        Self {
            window_bg_col: ColorRGBA::from_rgba(31.0 / 255.0, 44.0 / 255.0, 46.0 / 255.0, 1.0),
            window_title_bar_col: ColorRGBA::from_rgba(17.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0),
            widget_bg_col: ColorRGBA::from_rgba(52.0 / 255.0, 74.0 / 255.0, 76.0 / 255.0, 1.0),
            outline_col: ColorRGBA::from_rgba(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0),
            outline_col_hl: ColorRGBA::from_rgba(17.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0),
            outline_thin_col: ColorRGBA::from_rgba(17.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0),
            separator_col: ColorRGBA::from_rgba(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0),
            handle_col: ColorRGBA::from_rgba(83.0 / 255.0, 118.0 / 255.0, 121.0 / 255.0, 1.0),
            text_col: ColorRGBA::from_rgba(194.0 / 255.0, 212.0 / 255.0, 214.0 / 255.0, 1.0),
            hover_window_bg_col: ColorRGBA::from_rgba(10.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0, 1.0),
            font_regular,
            checkbox_symbol: '󰄬',
            item_pad_outer: vec2(0.02, 0.02),
            item_pad_inner: vec2(0.01, 0.01),
            font_scale: 0.02,
            rounding: 0.000,
            slider_min_width: 0.05,
            outline_width: 0.0035,
            outline_thin_width: 0.002,
            separator_height: 0.0015,
            override_cursor: true,
            f32_format: |value: f32, to: &mut CompactString| -> core::fmt::Result {
                write!(to, "{:.2}", value)
            },
            f64_format: |value: f64, to: &mut CompactString| -> core::fmt::Result {
                write!(to, "{:.2}", value)
            },
        }
    }

    #[inline(always)]
    pub fn calc_text_size(&self, text_size: Vec2) -> Vec2 {
        text_size * self.font_scale
    }

    #[inline(always)]
    pub fn calc_text_width(&self, text_width: f32) -> f32 {
        text_width * self.font_scale
    }

    #[inline(always)]
    pub fn calc_text_height(&self, text_height: f32) -> f32 {
        text_height * self.font_scale
    }

    #[inline(always)]
    pub fn calc_text_box_size(&self, text_size: Vec2) -> Vec2 {
        self.item_pad_inner * 2.0 + text_size * self.font_scale
    } 

    #[inline(always)]
    pub fn calc_text_box_height(&self, text_height: f32) -> f32 {
        self.item_pad_inner.y * 2.0 + text_height * self.font_scale
    }

    #[inline(always)]
    pub fn calc_text_box_width(&self, text_width: f32) -> f32 {
        self.item_pad_inner.x * 2.0 + text_width * self.font_scale
    } 
}
