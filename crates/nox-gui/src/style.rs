use core::{
    fmt::Write,
};

use std::f32::consts::FRAC_PI_2;

use nox::mem::vec_types::GlobalVec;

use nox_geom::*;

use crate::*;

pub trait WindowStyle {

    fn font_regular(&self) -> &CompactString;

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
    fn inactive_widget_outline_col(&self) -> ColorSRGBA {
        DEFAULT_INACTIVE_WIDGET_OUTLINE_COL
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
    fn inactive_text_col(&self) -> ColorSRGBA {
        DEFAULT_INACTIVE_TEXT_COL
    }

    #[inline(always)]
    fn focused_text_col(&self) -> ColorSRGBA {
        DEFAULT_FOCUSED_TEXT_COL
    }

    #[inline(always)]
    fn active_text_col(&self) -> ColorSRGBA {
        DEFAULT_ACTIVE_TEXT_COL
    }

    #[inline(always)]
    fn selection_col(&self) -> ColorSRGBA {
        DEFAULT_SELECTION_COL
    }

    #[inline(always)]
    fn hover_window_bg_col(&self) -> ColorSRGBA {
        DEFAULT_HOVER_WINDOW_BG_COL
    }

    #[inline(always)]
    fn scroll_bar_col(&self) -> ColorSRGBA {
        DEFAULT_SCROLL_BAR_COL
    }

    #[inline(always)]
    fn scroll_bar_handle_col(&self) -> ColorSRGBA {
        DEFAULT_SCROLL_BAR_HANDLE_COL
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
        DEFAULT_INACTIVE_TEXT_COL.with_alpha(0.4)
    }

    #[inline(always)]
    fn get_checkmark_points(
        &self,
        text_renderer: &mut TextRenderer, 
        points: &mut GlobalVec<[f32; 2]>,
    )
    {
        let scale = 1.0 / text_renderer.font_height(self.font_regular()).unwrap();
        let mut checkmark = [Vec2::default(); 6];
        checkmark[0] = vec2(0.5, -0.75);
        checkmark[1] = vec2(-0.2, 0.5);
        checkmark[2] =
            (checkmark[1] - checkmark[0]).rotated(FRAC_PI_2) * 0.5;
        checkmark[3] =
            checkmark[2] + (checkmark[2].normalized().rotated(-FRAC_PI_2) * 0.25);
        checkmark[4] =
            checkmark[1] + vec2(0.0, 0.25);
        checkmark[5] = checkmark[0] +
            (checkmark[0] - checkmark[1]).normalized().rotated(FRAC_PI_2) * 0.25;
        checkmark.reverse();
        points.append_map(&checkmark, |&p| (p * self.font_scale() * scale * 0.7).into());
    }

    #[inline(always)]
    fn pixels_per_unit(&self) -> f32 {
        1300.0
    }

    #[inline(always)]
    fn item_pad_outer(&self) -> Vec2 {
        vec2(0.02, 0.02)
    }

    #[inline(always)]
    fn item_pad_inner(&self) -> Vec2 {
        vec2(0.01, 0.008)
    }

    #[inline(always)]
    fn font_scale(&self) -> f32 {
        0.018
    }

    #[inline(always)]
    fn title_add_scale(&self) -> f32 {
        1.2
    }

    #[inline(always)]
    fn rounding(&self) -> f32 {
        0.003
    }

    #[inline(always)]
    fn cursor_error_margin(&self) -> f32 {
        0.02
    }

    #[inline(always)]
    fn scroll_speed(&self) -> f32 {
        2.0
    }

    #[inline(always)]
    fn natural_scroll(&self) -> bool {
        false
    }

    #[inline(always)]
    fn scroll_bar_width(&self) -> f32 {
        0.01
    }

    #[inline(always)]
    fn scroll_bar_fat_width(&self) -> f32 {
        0.02
    }

    #[inline(always)]
    fn window_outline_width(&self) -> f32 {
        0.002
    }

    #[inline(always)]
    fn focused_window_outline_width(&self) -> f32 {
        0.0035
    }

    #[inline(always)]
    fn focused_widget_outline_width(&self) -> f32 {
        0.001
    }

    #[inline(always)]
    fn active_widget_outline_width(&self) -> f32 {
        0.002
    }

    #[inline(always)]
    fn plot_line_width(&self) -> f32 {
        0.002
    }

    #[inline(always)]
    fn default_handle_radius(&self) -> f32 {
        0.01
    }

    #[inline(always)]
    fn collapse_symbol_scale(&self) -> f32 {
        0.01
    }

    #[inline(always)]
    fn focused_collapse_symbol_scale(&self) -> f32 {
        0.012
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
    fn input_text_cursor_width(&self) -> f32 {
        0.003
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
    fn animation_speed(&self) -> f32 {
        16.0
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
    fn default_slider_width(&self) -> f32 {
        0.2
    }

    #[inline(always)]
    fn slider_min_width(&self) -> f32 {
        0.1
    }

    #[inline(always)]
    fn min_input_text_width(&self) -> f32 {
        0.05
    }

    #[inline(always)]
    fn default_input_text_width(&self) -> f32 {
        0.2
    }

    #[inline(always)]
    fn calc_font_height(&self, text_renderer: &mut TextRenderer) -> f32
    {
        text_renderer.font_height(&self.font_regular()).unwrap_or_default() *
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
        self.item_pad_inner().y + self.item_pad_inner().y + text_height
    }
}

pub struct DefaultStyle(pub CompactString);

impl DefaultStyle {

    #[inline(always)]
    pub fn new(font_regular: &str) -> Self {
        Self(font_regular.into())
    }
}

impl WindowStyle for DefaultStyle {

    #[inline(always)]
    fn font_regular(&self) -> &CompactString {
        &self.0
    }
}

const DEFAULT_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(8.0 / 255.0, 12.0 / 255.0, 12.0 / 255.0, 1.0);

const DEFAULT_WINDOW_TITLE_BAR_COL: ColorSRGBA = DEFAULT_WINDOW_BG_COL;

const DEFAULT_WINDOW_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(38.0 / 255.0, 54.0 / 255.0, 54.0 / 255.0, 1.0);

const DEFAULT_FOCUSED_WINDOW_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0);

const DEFAULT_WIDGET_BG_COL: ColorSRGBA =
    ColorSRGBA::new(16.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0, 1.0);

const DEFAULT_SELECTION_COL: ColorSRGBA =
    ColorSRGBA::new(20.0 / 255.0, 41.0 / 255.0, 56.0 / 255.0, 1.0);

const DEFAULT_INACTIVE_TEXT_COL: ColorSRGBA =
    ColorSRGBA::new(194.0 / 255.0, 212.0 / 255.0, 214.0 / 255.0, 0.6);

const DEFAULT_FOCUSED_TEXT_COL: ColorSRGBA =
    DEFAULT_INACTIVE_TEXT_COL.with_alpha(1.0);

const DEFAULT_ACTIVE_TEXT_COL: ColorSRGBA =
    ColorSRGBA::white(1.0);

const DEFAULT_INACTIVE_WIDGET_OUTLINE_COL: ColorSRGBA = DEFAULT_INACTIVE_TEXT_COL;

const DEFAULT_FOCUSED_WIDGET_OUTLINE_COL: ColorSRGBA = DEFAULT_FOCUSED_TEXT_COL;

const DEFAULT_ACTIVE_WIDGET_OUTLINE_COL: ColorSRGBA = DEFAULT_ACTIVE_TEXT_COL;

const DEFAULT_HOVER_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(6.0 / 255.0, 9.0 / 255.0, 9.0 / 255.0, 1.0);

const DEFAULT_SCROLL_BAR_COL: ColorSRGBA =
    DEFAULT_INPUT_TEXT_BG_COL;

const DEFAULT_SCROLL_BAR_HANDLE_COL: ColorSRGBA =
    DEFAULT_FOCUSED_TEXT_COL;

const DEFAULT_INPUT_TEXT_BG_COL: ColorSRGBA =
    ColorSRGBA::new(4.0 / 255.0, 6.0 / 255.0, 6.0 / 255.0, 1.0);

const DEFAULT_INPUT_TEXT_ACTIVE_OUTLINE_COL: ColorSRGBA =
    ColorSRGBA::new(40.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.7);

const DEFAULT_INPUT_TEXT_SELECTION_BG_COL: ColorSRGBA
    = ColorSRGBA::new(24.0 / 255.0, 129.0 / 255.0, 129.0 / 255.0, 0.7);
