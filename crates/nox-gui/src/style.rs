use crate::*;

use nox_geom::*;

pub struct Style<FontHash> {
    pub window_bg_col: ColorRGBA,
    pub widget_bg_col: ColorRGBA,
    pub widget_outline_col: ColorRGBA,
    pub widget_outline_col_hl: ColorRGBA,
    pub slider_handle_col: ColorRGBA,
    pub text_col: ColorRGBA,
    pub font_regular: FontHash,
    pub item_pad_outer: Vec2,
    pub item_pad_inner: Vec2,
    pub font_scale_regular: f32,
    pub font_scale_header: f32,
    pub rounding: f32,
    pub slider_min_width: f32,
    pub outline_width: f32,
}

impl<FontHash> Style<FontHash> {

    pub fn new(font_regular: FontHash) -> Self {
        Self {
            window_bg_col: ColorRGBA::from_rgba(38.0 / 255.0, 37.0 / 255.0, 57.0 / 255.0, 1.0),
            widget_bg_col: ColorRGBA::from_rgba(58.0 / 255.0, 56.0 / 255.0, 83.0 / 255.0, 1.0),
            //widget_outline_col: ColorRGBA::from_rgba(25.0 / 255.0, 25.0 / 255.0, 36.0 / 255.0, 1.0),
            widget_outline_col: ColorRGBA::from_rgba(21.0 / 255.0, 21.0 / 255.0, 29.0 / 255.0, 1.0),
            widget_outline_col_hl: ColorRGBA::from_rgba(112.0 / 255.0, 110.0 / 255.0, 146.0 / 255.0, 1.0),
            //slider_handle_col: ColorRGBA::from_rgba(151.0 / 255.0, 149.0 / 255.0, 188.0 / 255.0, 1.0),
            slider_handle_col: ColorRGBA::from_rgba(112.0 / 255.0, 110.0 / 255.0, 146.0 / 255.0, 1.0),
            text_col: ColorRGBA::white(),
            font_regular,
            item_pad_outer: vec2(0.02, 0.02),
            item_pad_inner: vec2(0.01, 0.01),
            font_scale_regular: 0.03,
            font_scale_header: 0.04,
            rounding: 0.01,
            slider_min_width: 0.05,
            outline_width: 0.005,
        }
    }

    #[inline(always)]
    pub(crate) fn calc_outline_scale(&self, rect_size: Vec2) -> Vec2 {
        vec2(
            (rect_size.x + self.outline_width) / rect_size.x,
            (rect_size.y + self.outline_width) / rect_size.y,
        )
    }

    #[inline(always)]
    pub(crate) fn calc_outline_push_constant(
        &self,
        position: Vec2,
        rect_size: Vec2,
        inv_aspect_ratio: f32
    ) -> PushConstantsVertex
    {
        let width_half = self.outline_width * 0.5;
        push_constants_vertex(position - vec2(width_half, width_half), self.calc_outline_scale(rect_size), inv_aspect_ratio)
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
