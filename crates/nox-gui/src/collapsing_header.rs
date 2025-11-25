use core::f32::consts::FRAC_PI_2;

use compact_str::CompactString;

use nox::{
    *
};

use nox_font::{RenderedText, text_segment};

use nox_geom::*;

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollapsingHeaderId(pub usize);

pub struct CollapsingHeader {
    label: CompactString,
    label_text: RenderedText,
    pub offset: Vec2,
    pub symbol_vertex_range: Option<VertexRange>,
    pub beam_vertex_range: Option<VertexRange>,
    beam_width: f32,
    beam_height: f32,
    rotation: f32,
    flags: u32,
}

impl CollapsingHeader {

    const COLLAPSED: u32 = 0x1;
    const HOVERED: u32 = 0x2;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            label: Default::default(),
            label_text: Default::default(),
            offset: Default::default(),
            symbol_vertex_range: None,
            beam_vertex_range: None,
            beam_width: 0.0,
            beam_height: 0.0,
            rotation: 0.0,
            flags: Self::COLLAPSED,
        }
    }

    #[inline(always)]
    pub fn collapsed(&self) -> bool {
        self.flags & Self::COLLAPSED == Self::COLLAPSED
    }

    #[inline(always)]
    pub fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    pub fn set_label(
        &mut self,
        style: &impl UiStyle,
        text_renderer: &mut TextRenderer,
        label: &str
    )
    {
        if self.label != label {
            self.label = CompactString::new(label);
            self.label_text = text_renderer.render(
                &[text_segment(&self.label, style.font_regular())], false, 0.0 
            ).unwrap_or_default();
        }
    }

    #[inline(always)]
    pub fn label_text(&self) -> &RenderedText {
        &self.label_text
    }

    #[inline(always)]
    pub fn set_beam_height(&mut self, height: f32) {
        self.beam_height = height;
    }

    #[inline(always)]
    pub fn update(
        &mut self,
        ctx: &WindowCtx,
        window_pos: Vec2,
        min_bounds: Vec2,
        max_bounds: Vec2,
        cursor_pos: Vec2,
        style: &impl UiStyle,
        widget_active: bool,
        mut collect_text: impl FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> f32
    {
        let item_pad_outer = style.item_pad_outer();
        let collapse_scale = style.collapse_symbol_scale();
        let text_size = style.calc_text_size(&self.label_text);
        let offset = self.offset;
        let bounding_rect = BoundingRect::from_position_size(
            window_pos + offset,
            vec2(collapse_scale + item_pad_outer.x + text_size.x, text_size.y)
        );
        self.flags &= !Self::HOVERED;
        or_flag!(self.flags, Self::HOVERED, bounding_rect.is_point_inside(cursor_pos) && !widget_active);
        if !widget_active && self.hovered() && ctx.mouse_button_state(MouseButton::Left).pressed() {
            self.flags ^= Self::COLLAPSED;
        }
        if self.collapsed() {
            self.rotation =
                (self.rotation - FRAC_PI_2 * style.animation_speed() * ctx.delta_time_secs_f32())
                .clamp(0.0, FRAC_PI_2);
        } else {
            self.rotation =
                (self.rotation + FRAC_PI_2 * style.animation_speed() * ctx.delta_time_secs_f32())
                .clamp(0.0, FRAC_PI_2);
        }
        collect_text(
            &self.label_text, offset + vec2(collapse_scale + style.item_pad_inner().x, 0.0),
            BoundedTextInstance {
                add_scale: vec2(1.0, 1.0),
                min_bounds,
                max_bounds,
                color: if self.hovered() {
                    style.focused_text_col()
                } else {
                    style.inactive_text_col()
                }
            }
        );
        self.beam_width = style.window_stroke_thickness();
        offset.x + collapse_scale + text_size.x + item_pad_outer.x
    }

    #[inline(always)]
    pub fn set_vertex_params(&self, style: &impl UiStyle, vertices: &mut [Vertex]) {
        let rotation = self.rotation;
        let (scale, color) = 
            if self.hovered() {
                (
                    style.focused_collapse_symbol_scale(),
                    style.focused_text_col(),
                )
            } else {
                (
                    style.collapse_symbol_scale(),
                    style.inactive_text_col(),
                )
            };
        let offset = self.offset + vec2(scale * 0.5, style.calc_text_height(&self.label_text) * 0.5);
        if let Some(range) = self.symbol_vertex_range {
            let start = range.start();
            vertices[start] = Vertex {
                pos: vec2(0.5, 0.0).rotated(rotation) * scale,
                offset,
                color,
            };
            vertices[start + 1] = Vertex {
                pos: vec2(-0.5, 0.5).rotated(rotation) * scale,
                offset,
                color,
            };
            vertices[start + 2] = Vertex {
                pos: vec2(-0.5, -0.5).rotated(rotation) * scale,
                offset,
                color,
            };
        }
        if let Some(range) = self.beam_vertex_range {
            let item_pad_outer = style.item_pad_outer();
            let beam_width_half = self.beam_width * 0.5;
            let offset = self.offset + vec2(style.collapse_symbol_scale() * 0.5, item_pad_outer.y + item_pad_outer.y);
            let beam_height = self.beam_height - item_pad_outer.y;
            let start = range.start();
            let color = color.scale_alpha(0.3);
            vertices[start] = Vertex {
                pos: vec2(-beam_width_half, 0.0),
                offset,
                color,
            };
            vertices[start + 1] = Vertex {
                pos: vec2(-beam_width_half, beam_height),
                offset,
                color,
            };
            vertices[start + 2] = Vertex {
                pos: vec2(beam_width_half, beam_height),
                offset,
                color,
            };
            vertices[start + 3] = Vertex {
                pos: vec2(beam_width_half, 0.0),
                offset,
                color,
            };
        }
    }

    #[inline(always)]
    pub fn hide(&self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.symbol_vertex_range);
        hide_vertices(vertices, self.beam_vertex_range);
    }
}
