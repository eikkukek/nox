use core::{
    fmt::Write,
};

use std::f32::consts::FRAC_PI_2;

use nox_proc::BuildStructure;
use nox::mem::{vec::{ArrayVec, Vec32}, AsRaw};
use nox_geom::*;

use crate::*;

#[repr(u32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, AsRaw)]
pub enum StrokeType {
    #[default]
    Type1 = 0,
    Type2 = 1,
    Type3 = 2,
    Custom = 3,
}

pub type InteractStrokes = ArrayVec<Stroke, 4>;

pub struct InteractVisuals {
    pub fill_col: ColorSRGBA,
    pub bg_strokes: InteractStrokes,
    pub bg_stroke_type: StrokeType,
    pub fg_strokes: InteractStrokes,
    pub fg_stroke_type: StrokeType,
    pub rounding: f32,
}

impl InteractVisuals {

    #[inline(always)]
    pub fn fg_stroke_col(&self) -> ColorSRGBA {
        self.fg_strokes[self.fg_stroke_type as usize].col
    }
}

#[derive(Clone, BuildStructure)]
#[by_mut]
pub struct UiStyle {
    #[default(ConstName::new(""))]
    pub regular_font: ConstName,
    #[default(DEFAULT_WINDOW_BG_COL)]
    pub window_bg_col: ColorSRGBA,
    #[default(DEFAULT_WINDOW_TITLE_BAR_COL)]
    pub window_title_bar_col: ColorSRGBA,
    #[default(DEFAULT_WINDOW_STROKE_COL)]
    pub window_stroke_col: ColorSRGBA,
    #[default(DEFAULT_FOCUSED_WINDOW_STROKE_COL)]
    pub focused_window_stroke_col: ColorSRGBA,
    #[default(DEFAULT_WIDGET_BG_COL)]
    pub widget_bg_col: ColorSRGBA,
    #[default(DEFAULT_INACTIVE_TEXT_COL)]
    pub inactive_widget_fg_col: ColorSRGBA,
    #[default(DEFAULT_FOCUSED_TEXT_COL)]
    pub focused_widget_fg_col: ColorSRGBA,
    #[default(DEFAULT_ACTIVE_TEXT_COL)]
    pub active_widget_fg_col: ColorSRGBA,
    #[default(DEFAULT_FOCUSED_WIDGET_STROKE_COL)]
    pub focused_widget_stroke_col: ColorSRGBA,
    #[default(DEFAULT_ACTIVE_WIDGET_STROKE_COL)]
    pub active_widget_stroke_col: ColorSRGBA,
    #[default(DEFAULT_INACTIVE_TEXT_COL)]
    pub inactive_text_col: ColorSRGBA,
    #[default(DEFAULT_FOCUSED_TEXT_COL)]
    pub focused_text_col: ColorSRGBA,
    #[default(DEFAULT_ACTIVE_TEXT_COL)]
    pub active_text_col: ColorSRGBA,
    #[default(DEFAULT_SELECTION_COL)]
    pub selection_col: ColorSRGBA,
    #[default(DEFAULT_HOVER_WINDOW_BG_COL)]
    pub hover_window_bg_col: ColorSRGBA,
    #[default(DEFAULT_SCROLL_BAR_COL)]
    pub scroll_bar_col: ColorSRGBA,
    #[default(DEFAULT_SCROLL_BAR_HANDLE_COL)]
    pub scroll_bar_handle_col: ColorSRGBA,
    #[default(DEFAULT_INPUT_TEXT_BG_COL)]
    pub input_text_bg_col: ColorSRGBA,
    #[default(DEFAULT_INPUT_TEXT_ACTIVE_STROKE_COL)]
    pub input_text_active_stroke_col: ColorSRGBA,
    #[default(DEFAULT_INPUT_TEXT_SELECTION_BG_COL)]
    pub input_text_selection_bg_col: ColorSRGBA,
    #[default(DEFAULT_INACTIVE_TEXT_COL)]
    pub input_text_empty_text_col: ColorSRGBA,
    #[default(1300.0)]
    pub pixels_per_unit: f32,
    #[default(vec2(0.02, 0.02))]
    pub item_pad_outer: Vec2,
    #[default(vec2(0.007, 0.005))]
    pub item_pad_inner: Vec2,
    #[default(0.018)]
    pub font_scale: f32,
    #[default(1.2)]
    pub title_add_scale: f32,
    #[default(0.003)]
    pub rounding: f32,
    #[default(0.02)]
    pub cursor_error_margin: f32,
    #[default(2.0)]
    pub scroll_speed: f32,
    #[default(false)]
    pub natural_scroll: bool,
    #[default(0.01)]
    pub scroll_bar_width: f32,
    #[default(0.02)]
    pub scroll_bar_fat_width: f32,
    #[default(0.002)]
    pub window_stroke_thickness: f32,
    #[default(0.0035)]
    pub focused_window_stroke_thickness: f32,
    #[default(0.002)]
    pub focused_widget_stroke_thickness: f32,
    #[default(0.0026)]
    pub active_widget_stroke_thickness: f32,
    #[default(0.02)]
    pub plot_line_width: f32,
    #[default(0.01)]
    pub default_handle_radius: f32,
    #[default(0.01)]
    pub collapse_symbol_scale: f32,
    #[default(0.012)]
    pub focused_collapse_symbol_scale: f32,
    #[default(vec2(0.3, 0.3))]
    pub color_picker_size: Vec2,
    #[default(0.3 / 20.0)]
    pub alpha_tile_width: f32,
    #[default(0.003)]
    pub input_text_cursor_width: f32,
    #[default(0.5)]
    pub input_text_cursor_switch_speed: f32,
    #[default(3.0)]
    pub input_text_selection_scroll_speed: f32,
    #[default(0.5)]
    pub double_click_secs: f32,
    #[default(16.0)]
    pub animation_speed: f32,
    #[default(true)]
    pub override_cursor: bool,
    #[default(|value, to| write!(to, "{:.2}", value))]
    pub f32_format: fn(value: f32, to: &mut dyn Write) -> core::fmt::Result,
    #[default(|value, to| write!(to, "{:.2}", value))]
    pub f64_format: fn(value: f64, to: &mut dyn Write) -> core::fmt::Result,
    #[default(0.2)]
    pub default_slider_width: f32,
    #[default(0.1)]
    pub slider_min_width: f32,
    #[default(0.05)]
    pub min_input_text_width: f32,
    #[default(0.2)]
    pub default_input_text_width: f32,
}

impl UiStyle {

    #[inline]
    pub fn interact_visuals(&self, reaction: &Reaction) -> InteractVisuals {
        let bg_strokes = ArrayVec::from([
            Stroke {
                col: self.active_widget_stroke_col,
                thickness: self.active_widget_stroke_thickness,
            },
            Stroke {
                col: self.focused_widget_stroke_col,
                thickness: self.focused_widget_stroke_thickness,
            },
        ].as_slice());
        let mut fg_strokes = ArrayVec::from([
            Stroke {
                col: self.active_widget_fg_col,
                thickness: self.active_widget_stroke_thickness,
            },
            Stroke {
                col: self.focused_widget_fg_col,
                thickness: self.focused_widget_stroke_thickness,
            },
        ].as_slice());
        if reaction.held() {
            InteractVisuals {
                fill_col: self.widget_bg_col,
                bg_strokes,
                bg_stroke_type: StrokeType::Type1,
                fg_strokes,
                fg_stroke_type: StrokeType::Type1,
                rounding: self.rounding,
            }
        } else if reaction.hovered() {
            InteractVisuals {
                fill_col: self.widget_bg_col,
                bg_strokes,
                bg_stroke_type: StrokeType::Type2,
                fg_strokes,
                fg_stroke_type: StrokeType::Type2,
                rounding: self.rounding,
            }
        } else {
            fg_strokes[1].col = self.inactive_widget_fg_col;
            InteractVisuals {
                fill_col: self.widget_bg_col,
                bg_strokes,
                bg_stroke_type: StrokeType::Type3,
                fg_strokes,
                fg_stroke_type: StrokeType::Type2,
                rounding: self.rounding,
            }
        }
    }

    #[inline]
    pub fn get_checkmark_points(
        &self,
        text_renderer: &mut TextRenderer, 
        points: &mut Vec32<[f32; 2]>,
    )
    {
        let scale = 1.0 / text_renderer.font_height(&self.regular_font).unwrap();
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
        points.extend(checkmark.iter().map(|&p| (p * self.font_scale * scale * 0.7).into()));
    }

    #[inline]
    pub fn calc_font_height(&self, text_renderer: &mut TextRenderer) -> f32
    {
        text_renderer.font_height(&self.regular_font).unwrap_or_default() *
            self.font_scale
    }

    #[inline]
    pub fn calc_text_size(&self, text: &font::RenderedText) -> Vec2 {
        vec2(text.text_width, text.row_height * text.text_rows as f32) * self.font_scale
    }

    #[inline]
    pub fn calc_text_width(&self, text: &font::RenderedText) -> f32 {
        text.text_width * self.font_scale
    }

    #[inline]
    pub fn calc_text_height(&self, text: &font::RenderedText) -> f32 {
        text.row_height * text.text_rows as f32 * self.font_scale
    }

    #[inline]
    pub fn calc_text_box_size(&self, text: &font::RenderedText) -> Vec2 {
        self.item_pad_inner + self.item_pad_inner + self.calc_text_size(text)
    } 

    #[inline]
    pub fn calc_text_box_width(&self, text: &font::RenderedText) -> f32 {
        self.item_pad_inner.x + self.item_pad_inner.x +  self.calc_text_width(text)
    }

    #[inline]
    pub fn calc_text_box_height(&self, text: &font::RenderedText) -> f32 {
        self.item_pad_inner.y + self.item_pad_inner.y + self.calc_text_height(text)
    }

    #[inline]
    pub fn calc_text_box_size_from_text_size(&self, text_size: Vec2) -> Vec2 {
        self.item_pad_inner + self.item_pad_inner +  text_size
    }

    #[inline]
    pub fn calc_text_box_width_from_text_width(&self, text_width: f32) -> f32 {
        self.item_pad_inner.x + self.item_pad_inner.x +  text_width
    }

    #[inline]
    pub fn calc_text_box_height_from_text_height(&self, text_height: f32) -> f32 {
        self.item_pad_inner.y + self.item_pad_inner.y + text_height
    }
}

const DEFAULT_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(8.0 / 255.0, 12.0 / 255.0, 12.0 / 255.0, 1.0);

const DEFAULT_WINDOW_TITLE_BAR_COL: ColorSRGBA = DEFAULT_WINDOW_BG_COL;

const DEFAULT_WINDOW_STROKE_COL: ColorSRGBA =
    ColorSRGBA::new(38.0 / 255.0, 54.0 / 255.0, 54.0 / 255.0, 1.0);

const DEFAULT_FOCUSED_WINDOW_STROKE_COL: ColorSRGBA =
    ColorSRGBA::new(103.0 / 255.0, 148.0 / 255.0, 152.0 / 255.0, 1.0);

const DEFAULT_WIDGET_BG_COL: ColorSRGBA =
    ColorSRGBA::new(16.0 / 255.0, 25.0 / 255.0, 25.0 / 255.0, 1.0);

const DEFAULT_SELECTION_COL: ColorSRGBA =
    ColorSRGBA::new(20.0 / 255.0, 41.0 / 255.0, 56.0 / 255.0, 1.0);

const DEFAULT_INACTIVE_TEXT_COL: ColorSRGBA =
    ColorSRGBA::new(194.0 / 255.0, 212.0 / 256.0, 214.0 / 255.0, 0.5);

const DEFAULT_FOCUSED_TEXT_COL: ColorSRGBA =
    DEFAULT_INACTIVE_TEXT_COL.with_alpha(0.8);

const DEFAULT_ACTIVE_TEXT_COL: ColorSRGBA =
    ColorSRGBA::white(1.0);

const DEFAULT_FOCUSED_WIDGET_STROKE_COL: ColorSRGBA =
    ColorSRGBA::new(65.0 / 255.0, 95.0 / 255.0, 98.0 / 255.0, 1.0);

const DEFAULT_ACTIVE_WIDGET_STROKE_COL: ColorSRGBA =
    ColorSRGBA::new(82.0 / 255.0, 118.0 / 255.0, 122.0 / 255.0, 1.0);

const DEFAULT_HOVER_WINDOW_BG_COL: ColorSRGBA =
    ColorSRGBA::new(6.0 / 255.0, 9.0 / 255.0, 9.0 / 255.0, 1.0);

const DEFAULT_SCROLL_BAR_COL: ColorSRGBA =
    DEFAULT_INPUT_TEXT_BG_COL;

const DEFAULT_SCROLL_BAR_HANDLE_COL: ColorSRGBA =
    DEFAULT_FOCUSED_TEXT_COL;

const DEFAULT_INPUT_TEXT_BG_COL: ColorSRGBA =
    ColorSRGBA::new(4.0 / 255.0, 6.0 / 255.0, 6.0 / 255.0, 1.0);

const DEFAULT_INPUT_TEXT_ACTIVE_STROKE_COL: ColorSRGBA =
    ColorSRGBA::new(40.0 / 255.0, 215.0 / 255.0, 215.0 / 255.0, 0.7);

const DEFAULT_INPUT_TEXT_SELECTION_BG_COL: ColorSRGBA
    = ColorSRGBA::new(24.0 / 255.0, 129.0 / 255.0, 129.0 / 255.0, 0.7);
