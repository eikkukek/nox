use core::fmt::Write;

use crate::*;

use nox_font::VertexTextRenderer;
use nox_geom::*;

pub trait WindowStyle<FontHash> {

    fn font_regular(&self) -> &FontHash;

    #[inline(always)]
    fn window_bg_col(&self) -> ColorSRGBA {
        DEFAULT_WINDOW_BG_COL
    }

    #[inline(always)]
    fn window_title_bar_col(&self) -> ColorSRGBA {
        DEFAULT_WINDOW_TITLE_BAR_COL
    }

    #[inline(always)]
    fn window_outline_col(&self) -> ColorSRGBA {
        DEFAULT_WINDOW_OUTLINE_COL
    }

    #[inline(always)]
    fn focused_window_outline_col(&self) -> ColorSRGBA {
        DEFAULT_FOCUSED_WINDOW_OUTLINE_COL
    }

    #[inline(always)]
    fn widget_bg_col(&self) -> ColorSRGBA {
        DEFAULT_WIDGET_BG_COL
    }

    #[inline(always)]
    fn focused_widget_outline_col(&self) -> ColorSRGBA {
        DEFAULT_FOCUSED_WIDGET_OUTLINE_COL
    }

    #[inline(always)]
    fn active_widget_outline_col(&self) -> ColorSRGBA {
        DEFAULT_ACTIVE_WIDGET_OUTLINE_COL
    }

    #[inline(always)]
    fn text_col(&self) -> ColorSRGBA {
        DEFAULT_TEXT_COL
    }

    #[inline(always)]
    fn separator_col(&self) -> ColorSRGBA {
        DEFAULT_SEPARATOR_COL
    }

    #[inline(always)]
    fn handle_col(&self) -> ColorSRGBA {
        DEFAULT_HANDLE_COL
    }

    #[inline(always)]
    fn input_text_bg_col(&self) -> ColorSRGBA {
        DEFAULT_INPUT_TEXT_BG_COL
    }

    #[inline(always)]
    fn input_text_active_outline_col(&self) -> ColorSRGBA {
        DEFAULT_INPUT_TEXT_ACTIVE_OUTLINE_COL
    }

    #[inline(always)]
    fn input_text_selection_bg_col(&self) -> ColorSRGBA {
        DEFAULT_INPUT_TEXT_SELECTION_BG_COL
    }

    #[inline(always)]
    fn input_text_empty_text_color(&self) -> ColorSRGBA {
        DEFAULT_TEXT_COL.with_alpha(0.4)
    }

    #[inline(always)]
    fn checkbox_symbol(&self) -> char {
        '󰄬'
    }

    #[inline(always)]
    fn item_pad_outer(&self) -> Vec2 {
        vec2(0.02, 0.02)
    }

    #[inline(always)]
    fn item_pad_inner(&self) -> Vec2 {
        vec2(0.01, 0.01)
    }

    #[inline(always)]
    fn font_scale(&self) -> f32 {
        0.02
    }

    #[inline(always)]
    fn rounding(&self) -> f32 {
        0.005
    }

    #[inline(always)]
    fn cursor_error_margin(&self) -> f32 {
        0.02
    }

    #[inline(always)]
    fn outline_width(&self) -> f32 {
        0.002
    }

    #[inline(always)]
    fn focused_outline_width(&self) -> f32 {
        0.0035
    }

    #[inline(always)]
    fn separator_height(&self) -> f32 {
        0.0015
    }

    #[inline(always)]
    fn default_handle_radius(&self) -> f32 {
        0.01
    }

    #[inline(always)]
    fn color_picker_size(&self) -> Vec2 {
        vec2(0.3, 0.3)
    }

    #[inline(always)]
    fn alpha_tile_width(&self) -> f32 {
        0.3 / 20.0
    }

    #[inline(always)]
    fn default_value_drag_speed(&self) -> f32 {
        5.0
    }

    #[inline(always)]
    fn input_text_cursor_width(&self) -> f32 {
        0.005
    }

    #[inline(always)]
    fn input_text_cursor_switch_speed(&self) -> f32 {
        0.5
    }

    #[inline(always)]
    fn input_text_selection_scroll_speed(&self) -> f32 {
        3.0
    }

    #[inline(always)]
    fn double_click_secs(&self) -> f32 {
        0.5
    }

    #[inline(always)]
    fn override_cursor(&self) -> bool {
        true
    }

    #[inline(always)]
    fn f32_format(&self, value: f32, to: &mut impl Write) -> core::fmt::Result {
        write!(to, "{:.2}", value)
    }

    #[inline(always)]
    fn f64_format(&self, value: f64, to: &mut impl Write) -> core::fmt::Result {
        write!(to, "{:.2}", value)
    }

    #[inline(always)]
    fn min_slider_width(&self) -> f32 {
        0.05
    }

    #[inline(always)]
    fn min_input_text_width(&self) -> f32 {
        0.05
    }

    #[inline(always)]
    fn max_input_text_width(&self) -> f32 {
        0.5
    }

    #[inline(always)]
    fn calc_font_height(&self, text_renderer: &mut VertexTextRenderer<FontHash>) -> f32
        where 
            FontHash: Clone + Eq + core::hash::Hash,
    {
        text_renderer.font_height(self.font_regular()).unwrap_or_default() as f32 *
            self.font_scale()
    }

    #[inline(always)]
    fn calc_text_size(&self, text: &font::RenderedText) -> Vec2 {
        vec2(text.text_width, text.row_height * text.text_rows as f32) * self.font_scale()
    }

    #[inline(always)]
    fn calc_text_width(&self, text: &font::RenderedText) -> f32 {
        text.text_width * self.font_scale()
    }

    #[inline(always)]
    fn calc_text_height(&self, text: &font::RenderedText) -> f32 {
        text.row_height * text.text_rows as f32 * self.font_scale()
    }

    #[inline(always)]
    fn calc_text_box_size(&self, text: &font::RenderedText) -> Vec2 {
        self.item_pad_inner() + self.item_pad_inner() + self.calc_text_size(text)
    } 

    #[inline(always)]
    fn calc_text_box_width(&self, text: &font::RenderedText) -> f32 {
        self.item_pad_inner().x + self.item_pad_inner().x +  self.calc_text_width(text)
    }

    #[inline(always)]
    fn calc_text_box_height(&self, text: &font::RenderedText) -> f32 {
        self.item_pad_inner().y + self.item_pad_inner().y + self.calc_text_height(text)
    }

    #[inline(always)]
    fn calc_text_box_size_from_text_size(&self, text_size: Vec2) -> Vec2 {
        self.item_pad_inner() + self.item_pad_inner() +  text_size
    }

    #[inline(always)]
    fn calc_text_box_width_from_text_width(&self, text_width: f32) -> f32 {
        self.item_pad_inner().x + self.item_pad_inner().x +  text_width
    }

    #[inline(always)]
    fn calc_text_box_height_from_text_height(&self, text_height: f32) -> f32 {
        self.item_pad_inner().y + self.item_pad_inner().y +  text_height
    }
}

pub struct DefaultStyle<FontHash>(pub FontHash);

impl<FontHash> DefaultStyle<FontHash> {

    #[inline(always)]
    pub fn new(font_regular: FontHash) -> Self {
        Self(font_regular)
    }
}

impl<FontHash> WindowStyle<FontHash> for DefaultStyle<FontHash> {

    #[inline(always)]
    fn font_regular(&self) -> &FontHash {
        &self.0
    }
}

pub struct DefaultHoverStyle<FontHash>(pub FontHash);

impl<FontHash> DefaultHoverStyle<FontHash> {

    #[inline(always)]
    pub fn new(font_regular: FontHash) -> Self {
        Self(font_regular)
    }
}

impl<FontHash> WindowStyle<FontHash> for DefaultHoverStyle<FontHash> {

    #[inline(always)]
    fn font_regular(&self) -> &FontHash {
        &self.0
    }

    #[inline(always)]
    fn window_bg_col(&self) -> ColorSRGBA {
        DEFAULT_HOVER_CONTENTS_WINDOW_BG_COL
    }

    #[inline(always)]
    fn widget_bg_col(&self) -> ColorSRGBA {
        DEFAULT_HOVER_CONTENTS_WIDGET_BG_COL
    }

    #[inline(always)]
    fn focused_widget_outline_col(&self) -> ColorSRGBA {
        DEFAULT_HOVER_CONTENTS_FOCUSED_WIDGET_OUTLINE_COL
    }

    #[inline(always)]
    fn active_widget_outline_col(&self) -> ColorSRGBA {
        DEFAULT_HOVER_CONTENTS_ACTIVE_WIDGET_OUTLINE_COL
    }
}

const DEFAULT_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(31.0 / 255.0, 44.0 / 255.0, 46.0 / 255.0, 1.0);

const DEFAULT_WINDOW_TITLE_BAR_COL: ColorSRGBA =
    ColorSRGBA::new(17.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0);

const DEFAULT_WINDOW_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(17.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0);

const DEFAULT_FOCUSED_WINDOW_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0);

const DEFAULT_WIDGET_BG_COL: ColorSRGBA =
    ColorSRGBA::new(52.0 / 255.0, 74.0 / 255.0, 74.0 / 255.0, 1.0);

const DEFAULT_FOCUSED_WIDGET_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0);

const DEFAULT_ACTIVE_WIDGET_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(21.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0);

const DEFAULT_TEXT_COL: ColorSRGBA =
    ColorSRGBA::new(194.0 / 255.0, 212.0 / 255.0, 214.0 / 255.0, 1.0);

const DEFAULT_SEPARATOR_COL: ColorSRGBA =
    ColorSRGBA::new(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0);

const DEFAULT_HANDLE_COL: ColorSRGBA =
    ColorSRGBA::new(83.0 / 255.0, 118.0 / 255.0, 121.0 / 255.0, 1.0);

const DEFAULT_INPUT_TEXT_BG_COL: ColorSRGBA =
    ColorSRGBA::new(21.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0);

const DEFAULT_INPUT_TEXT_ACTIVE_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(40.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.7);

const DEFAULT_INPUT_TEXT_SELECTION_BG_COL: ColorSRGBA
    = ColorSRGBA::new(24.0 / 255.0, 129.0 / 255.0, 129.0 / 255.0, 0.7);

const DEFAULT_HOVER_CONTENTS_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(10.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0, 1.0);

const DEFAULT_HOVER_CONTENTS_WIDGET_BG_COL: ColorSRGBA =
    ColorSRGBA::new(20.0 / 255.0, 31.0 / 255.0, 31.0 / 255.0, 1.0);

const DEFAULT_HOVER_CONTENTS_FOCUSED_WIDGET_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(31.0 / 255.0, 46.0 / 255.0, 46.0 / 255.0, 1.0);

const DEFAULT_HOVER_CONTENTS_ACTIVE_WIDGET_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(16.0 / 255.0, 24.0 / 255.0, 24.0 / 255.0, 1.0);
