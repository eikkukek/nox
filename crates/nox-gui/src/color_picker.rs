use core::{
    hash::Hash,
    marker::PhantomData,
    f32::consts::{PI, TAU},
    fmt::Write,
};

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *
};

use nox_font::{text_segment, RenderedText};
use nox_geom::{
    shapes::*,
    *
};

use crate::*;

struct Contents<I, FontHash> {
    r_text: RenderedText,
    r_text_val: f32,
    g_text: RenderedText,
    g_text_val: f32,
    b_text: RenderedText,
    b_text_val: f32,
    alpha_text: RenderedText,
    alpha_text_val: f32,
    hue_text: RenderedText,
    hue_text_val: f32,
    hue_text_box_offset: Vec2,
    offset: Vec2,
    picker_handle_offset: Vec2,
    hue_picker_offset: Vec2,
    hue_picker_handle_offset_x: f32,
    alpha_picker_offset: Vec2,
    alpha_picker_handle_offset_x: f32,
    hsva: ColorHSVA,
    srgba: ColorSRGBA,
    rgba: ColorRGBA,
    window_rect: Rect,
    text_box_rect: Rect,
    picker_handle: Circle,
    hue_alpha_picker_handle_height: f32,
    picker_vertices: GlobalVec<ColorPickerVertex>,
    other_vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    other_vertices_draw_info_bg: DrawInfo,
    other_vertices_draw_info: DrawInfo,
    picker_draw_info: DrawInfo,
    window_vertex_range: VertexRange,
    window_outline_vertex_range: VertexRange,
    r_text_box_vertex_range: VertexRange,
    r_text_box_outline_vertex_range: VertexRange,
    g_text_box_vertex_range: VertexRange,
    g_text_box_outline_vertex_range: VertexRange,
    b_text_box_vertex_range: VertexRange,
    b_text_box_outline_vertex_range: VertexRange,
    alpha_text_box_vertex_range: VertexRange,
    alpha_text_box_outline_vertex_range: VertexRange,
    hue_text_box_vertex_range: VertexRange,
    hue_text_box_outline_vertex_range: VertexRange,
    picker_handle_vertex_range: VertexRange,
    picker_handle_outline_vertex_range: VertexRange,
    hue_picker_handle_vertex_range: VertexRange,
    alpha_picker_handle_vertex_range: VertexRange,
    outline_width: f32,
    rgb_text_size: Vec2,
    flags: u32,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> Contents<I, FontHash> {

    const WIDGET_HELD: u32 = 0x1;
    const SHOWN: u32 = 0x2;
    const PICKER_HELD: u32 = 0x4;
    const HUE_PICKER_HELD: u32 = 0x8;
    const ALPHA_PICKER_HELD: u32 = 0x10;
    const FONT_CHANGED: u32 = 0x20;
    const HOVERED_CHANGED: u32 = 0x40;
    const R_HOVERED: u32 = 0x80;
    const G_HOVERED: u32 = 0x100;
    const B_HOVERED: u32 = 0x200;
    const ALPHA_HOVERED: u32 = 0x400;
    const HUE_HOVERED: u32 = 0x800;

    fn new() -> Self {
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        let picker_rect = rect(vec2(0.0, 0.0), vec2(1.0, 1.0), 0.0);
        picker_rect.to_points(&mut |p| { points.push(p.into()); });
        let mut picker_vertices = GlobalVec::new();
        let mut indices = GlobalVec::new();
        earcut::earcut(&points, &[], false, &mut picker_vertices, &mut indices_usize).unwrap();
        let picker_draw_info = DrawInfo {
            first_index: 0,
            index_count: indices_usize.len() as u32,
            ..Default::default()
        };
        indices.append_map(&indices_usize, |&i| i as u32);
        Self {
            r_text: Default::default(),
            r_text_val: f32::NAN,
            g_text: Default::default(),
            g_text_val: f32::NAN,
            b_text: Default::default(),
            b_text_val: f32::NAN,
            alpha_text: Default::default(),
            alpha_text_val: f32::NAN,
            hue_text: Default::default(),
            hue_text_val: f32::NAN,
            hue_text_box_offset: Default::default(),
            offset: Default::default(),
            picker_handle_offset: Default::default(),
            hue_picker_offset: Default::default(),
            hue_picker_handle_offset_x: 0.0,
            alpha_picker_offset: Default::default(),
            alpha_picker_handle_offset_x: Default::default(),
            hsva: Default::default(),
            srgba: Default::default(),
            rgba: Default::default(),
            window_rect: Default::default(),
            text_box_rect: Default::default(),
            picker_handle: Default::default(),
            hue_alpha_picker_handle_height: 0.0,
            picker_vertices,
            other_vertices: Default::default(),
            indices,
            picker_draw_info,
            other_vertices_draw_info_bg: Default::default(),
            other_vertices_draw_info: Default::default(),
            window_vertex_range: Default::default(),
            window_outline_vertex_range: Default::default(),
            r_text_box_vertex_range: Default::default(),
            r_text_box_outline_vertex_range: Default::default(),
            g_text_box_vertex_range: Default::default(),
            g_text_box_outline_vertex_range: Default::default(),
            b_text_box_vertex_range: Default::default(),
            b_text_box_outline_vertex_range: Default::default(),
            alpha_text_box_vertex_range: Default::default(),
            alpha_text_box_outline_vertex_range: Default::default(),
            hue_text_box_vertex_range: Default::default(),
            hue_text_box_outline_vertex_range: Default::default(),
            picker_handle_vertex_range: Default::default(),
            picker_handle_outline_vertex_range: Default::default(),
            hue_picker_handle_vertex_range: Default::default(),
            alpha_picker_handle_vertex_range: Default::default(),
            outline_width: 0.0,
            rgb_text_size: Default::default(),
            flags: Self::FONT_CHANGED,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn shown(&self) -> bool {
        self.flags & Self::SHOWN == Self::SHOWN
    }

    #[inline(always)]
    fn set_shown(&mut self, value: bool) {
        self.flags &= !Self::SHOWN;
        self.flags |= Self::SHOWN * value as u32;
    }

    #[inline(always)]
    fn widget_held(&self) -> bool {
        self.flags & Self::WIDGET_HELD == Self::WIDGET_HELD
    }

    fn set_widget_held(&mut self, value: bool) {
        self.flags &= !Self::WIDGET_HELD;
        self.flags |= Self::WIDGET_HELD * value as u32;
    }
    
    #[inline(always)]
    fn picker_held(&self) -> bool {
        self.flags & Self::PICKER_HELD == Self::PICKER_HELD
    }
    
    #[inline(always)]
    fn hue_picker_held(&self) -> bool {
        self.flags & Self::HUE_PICKER_HELD == Self::HUE_PICKER_HELD
    }

    #[inline(always)]
    fn alpha_picker_held(&self) -> bool {
        self.flags & Self::ALPHA_PICKER_HELD == Self::ALPHA_PICKER_HELD
    }

    #[inline(always)]
    fn hovered_changed(&self) -> bool {
        self.flags & Self::HOVERED_CHANGED == Self::HOVERED_CHANGED
    }

    #[inline(always)]
    fn r_hovered(&self) -> bool {
        self.flags & Self::R_HOVERED == Self::R_HOVERED
    }

    #[inline(always)]
    fn g_hovered(&self) -> bool {
        self.flags & Self::G_HOVERED == Self::G_HOVERED
    }

    #[inline(always)]
    fn b_hovered(&self) -> bool {
        self.flags & Self::B_HOVERED == Self::B_HOVERED
    }

    #[inline(always)]
    fn alpha_hovered(&self) -> bool {
        self.flags & Self::ALPHA_HOVERED == Self::ALPHA_HOVERED
    }

    #[inline(always)]
    fn hue_hovered(&self) -> bool {
        self.flags & Self::HUE_HOVERED == Self::HUE_HOVERED
    }

    #[inline(always)]
    fn r_changed(&self) -> bool {
        self.r_hovered() && self.hovered_changed()
    }

    #[inline(always)]
    fn g_changed(&self) -> bool {
        self.g_hovered() && self.hovered_changed()
    }

    #[inline(always)]
    fn b_changed(&self) -> bool {
        self.b_hovered() && self.hovered_changed()
    }

    #[inline(always)]
    fn alpha_changed(&self) -> bool {
        self.alpha_hovered() && self.hovered_changed()
    }

    #[inline(always)]
    fn hue_changed(&self) -> bool {
        self.hue_hovered() && self.hovered_changed()
    }

    #[inline(always)]
    fn font_changed(&self) -> bool {
        self.flags & Self::FONT_CHANGED == Self::FONT_CHANGED
    }

    #[inline(always)]
    fn bounding_rect(&self, error_margin: f32) -> BoundingRect {
        BoundingRect::from_position_size(
            self.offset - vec2(error_margin, error_margin),
            self.window_rect.max + vec2(error_margin, error_margin),
        )
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
    ) -> bool
        where
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let item_pad_outer = style.item_pad_outer;
        let mut text_box_rect_max = self.text_box_rect.max;
        if self.font_changed() {
            let samples = (
                text_renderer.render(
                    &[text_segment("R 255", &style.font_regular)], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("G 255", &style.font_regular)], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("B 255", &style.font_regular)], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("A 255", &style.font_regular)], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("H 360°", &style.font_regular)], false, 0.0
                ).unwrap_or_default(),
            );
            let rgb_text_size_x = style.calc_text_box_width(
                samples.0.text_width
                    .max(samples.1.text_width)
                    .max(samples.2.text_width)
                    .max(samples.3.text_width)
                    .max(samples.4.text_width)
            );
            let text_box_height = style.calc_text_box_height(samples.0.row_height);
            let rgb_text_size_y =
                text_box_height * 4.0 +
                item_pad_outer.y * 5.0;
            self.rgb_text_size = vec2(rgb_text_size_x, rgb_text_size_y);
            text_box_rect_max.x = rgb_text_size_x;
            text_box_rect_max.y = style.calc_text_box_height(samples.0.row_height);
            self.flags &= !Self::FONT_CHANGED;
        }
        let srgba = self.srgba;
        let hsva = self.hsva;
        if self.r_text_val != srgba.r {
            let mut fmt = CompactString::default();
            write!(fmt, "R {}", ColorRGBA::map_channel(srgba.r)).ok();
            self.r_text = text_renderer.render(
                &[text_segment(&fmt, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
            self.r_text_val = srgba.r;
        }
        if self.g_text_val != srgba.g {
            let mut fmt = CompactString::default();
            write!(fmt, "G {}", ColorRGBA::map_channel(srgba.g)).ok();
            self.g_text = text_renderer.render(
                &[text_segment(&fmt, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
            self.g_text_val = srgba.g;
        }
        if self.b_text_val != srgba.b {
            let mut fmt = CompactString::default();
            write!(fmt, "B {}", ColorRGBA::map_channel(srgba.b)).ok();
            self.b_text = text_renderer.render(
                &[text_segment(&fmt, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
            self.b_text_val = srgba.b;
        }
        if self.alpha_text_val != srgba.alpha {
            let mut fmt = CompactString::default();
            write!(fmt, "A {}", (srgba.alpha * 255.0).round()).ok();
            self.alpha_text = text_renderer.render(
                &[text_segment(&fmt, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
            self.alpha_text_val = srgba.alpha;
        }
        if self.hue_text_val != hsva.hue {
            let mut fmt = CompactString::default();
            write!(fmt, "H {}°", (hsva.hue * 180.0 / PI).round()).ok();
            self.hue_text = text_renderer.render(
                &[text_segment(&fmt, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
            self.hue_text_val = hsva.hue;
        }
        let offset = self.offset;
        let picker_size = style.color_picker_size;
        let hue_alpha_picker_height = picker_size.y * 0.1;
        let item_pad_outer = style.item_pad_outer;
        let hue_picker_offset = vec2(
            offset.x,
            offset.y + (picker_size.y + item_pad_outer.y),
        ) + item_pad_outer;
        let alpha_picker_offset = hue_picker_offset + vec2(0.0, hue_alpha_picker_height + item_pad_outer.y);
        let text_offset = vec2(
            offset.x + item_pad_outer.x + picker_size.x + item_pad_outer.x,
            offset.y + item_pad_outer.y,
        );
        let hue_text_box_offset = vec2(
            offset.x + item_pad_outer.x + picker_size.x + item_pad_outer.x,
            (hue_picker_offset.y + picker_size.y * 0.05 - text_box_rect_max.y * 0.5)
                .max(offset.y + self.rgb_text_size.y),
        );
        self.hue_text_box_offset = hue_text_box_offset;
        let color_picker_offset = offset + item_pad_outer;
        let cursor_pos = cursor_pos - window_pos;
        let error_margin = vec2(style.cursor_error_margin, style.cursor_error_margin);
        let error_margin_2 = error_margin * 2.0;
        if self.picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::PICKER_HELD;
            } else {
                self.picker_handle_offset = cursor_pos.clamp(
                    color_picker_offset,
                    color_picker_offset + picker_size 
                );
            }
        }
        else if self.hue_picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HUE_PICKER_HELD;
            } else {
                self.hue_picker_handle_offset_x = cursor_pos.x.clamp(
                    hue_picker_offset.x,
                    hue_picker_offset.x + picker_size.x,
                );
            }
        }
        else if self.alpha_picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::ALPHA_PICKER_HELD;
            } else {
                self.alpha_picker_handle_offset_x = cursor_pos.x.clamp(
                    alpha_picker_offset.x,
                    alpha_picker_offset.x + picker_size.x,
                );
            }
        }
        else if self.r_changed() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HOVERED_CHANGED;
            } else {
                let r = self.rgba.r as f32 + delta_cursor_pos.x * style.default_value_drag_speed * 255.0;
                self.rgba.r = r.clamp(0.0, 255.0) as u8;
            }
        }
        else if self.g_changed() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HOVERED_CHANGED;
            } else {
                let g = self.rgba.g as f32 + delta_cursor_pos.x * style.default_value_drag_speed * 255.0;
                self.rgba.g = g.clamp(0.0, 255.0) as u8;
            }
        }
        else if self.b_changed() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HOVERED_CHANGED;
            } else {
                let b = self.rgba.b as f32 + delta_cursor_pos.x * style.default_value_drag_speed * 255.0;
                self.rgba.b = b.clamp(0.0, 255.0) as u8;
            }
        }
        else if self.alpha_changed() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HOVERED_CHANGED;
            } else {
                let alpha = (self.srgba.alpha + delta_cursor_pos.x * style.default_value_drag_speed)
                    .clamp(0.0, 1.0);
                self.srgba.alpha = alpha;
                self.hsva.alpha = alpha;
            }
        }
        else if self.hue_changed() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HOVERED_CHANGED
            } else {
                let hue = self.hsva.hue + (delta_cursor_pos.x * style.default_value_drag_speed) * TAU;
                self.hsva.hue = hue.clamp(0.0, TAU);
            }
        }
        else if nox.was_mouse_button_pressed(MouseButton::Left) {
            if BoundingRect::from_min_max(
                    color_picker_offset - error_margin,
                    color_picker_offset + picker_size + error_margin
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::PICKER_HELD;
            }
            else if BoundingRect::from_position_size(
                    hue_picker_offset - error_margin,
                    vec2(picker_size.x, hue_alpha_picker_height) +
                        error_margin_2,
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::HUE_PICKER_HELD;
            }
            else if BoundingRect::from_position_size(
                    alpha_picker_offset - error_margin,
                    vec2(picker_size.x, hue_alpha_picker_height) +
                        error_margin_2
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::ALPHA_PICKER_HELD;
            }
            else if self.r_hovered() || self.g_hovered() ||
                self.b_hovered() || self.hue_hovered() || self.alpha_hovered()
            {
                self.flags |= Self::HOVERED_CHANGED;
            }
        } else {
            self.flags &= !(
                Self::R_HOVERED | Self::G_HOVERED | Self::B_HOVERED | Self::ALPHA_HOVERED |
                Self::HUE_HOVERED
            );
            if BoundingRect::from_position_size(
                    text_offset - error_margin,
                    text_box_rect_max + error_margin_2
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::R_HOVERED;
            }
            else if BoundingRect::from_position_size(
                    text_offset + vec2(0.0, item_pad_outer.y + text_box_rect_max.y) - error_margin,
                    text_box_rect_max + error_margin_2,
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::G_HOVERED;
            }
            else if BoundingRect::from_position_size(
                    text_offset + vec2(0.0, item_pad_outer.y * 2.0 + text_box_rect_max.y * 2.0) - error_margin,
                    text_box_rect_max + error_margin_2
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::B_HOVERED;
            }
            else if BoundingRect::from_position_size(
                    text_offset + vec2(0.0, item_pad_outer.y * 3.0 + text_box_rect_max.y * 3.0) - error_margin,
                    text_box_rect_max + error_margin_2
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::ALPHA_HOVERED;
            }
            else if BoundingRect::from_position_size(
                    hue_text_box_offset - error_margin,
                    text_box_rect_max + error_margin_2
                )
                .is_point_inside(cursor_pos)
            {
                self.flags |= Self::HUE_HOVERED;
            }
            let hsva = self.hsva;
            self.picker_handle_offset = vec2(
                offset.x + picker_size.x * hsva.sat,
                offset.y + picker_size.y * (1.0 - hsva.val),
            ) + item_pad_outer;
            self.hue_picker_handle_offset_x = offset.x + picker_size.x * hsva.hue / TAU
                + item_pad_outer.x;
            self.alpha_picker_handle_offset_x = offset.x + picker_size.x * hsva.alpha
                + item_pad_outer.x;
        }
        if style.override_cursor {
            if self.r_hovered() || self.b_hovered() || self.g_hovered() || self.alpha_hovered() ||
                self.hue_hovered()
            {
                nox.set_cursor(CursorIcon::ColResize);
            } else if BoundingRect::from_position_size(
                    self.offset, self.window_rect.max
                )
                .is_point_inside(cursor_pos)
            {
                nox.set_cursor(CursorIcon::Default);
            }
        }
        let rgb_text_size = self.rgb_text_size;
        let hue_text_box_max_y = hue_text_box_offset.y - offset.y + text_box_rect_max.y + item_pad_outer.y;
        let mut window_rect_max = item_pad_outer + item_pad_outer + picker_size +
            vec2(
                item_pad_outer.x + rgb_text_size.x,
                hue_alpha_picker_height + item_pad_outer.y +
                hue_alpha_picker_height + item_pad_outer.y
            );
        window_rect_max.y = window_rect_max.y.max(hue_text_box_max_y);
        let handle_radius = style.default_handle_radius;
        let rounding = style.rounding;
        let outline_width = style.outline_width;
        let hue_alpha_picker_handle_height = picker_size.y * 0.06;
        let requires_triangulation =
            self.window_rect.max != window_rect_max ||
            self.window_rect.rounding != rounding ||
            self.text_box_rect.max != text_box_rect_max ||
            self.text_box_rect.rounding != rounding ||
            self.outline_width != outline_width ||
            self.picker_handle.radius != handle_radius ||
            self.hue_alpha_picker_handle_height != hue_alpha_picker_handle_height;
        self.window_rect.max = window_rect_max;
        self.window_rect.rounding = rounding;
        self.text_box_rect.max = text_box_rect_max;
        self.text_box_rect.rounding = rounding;
        self.outline_width = outline_width;
        self.picker_handle.radius = handle_radius;
        self.hue_alpha_picker_handle_height = hue_alpha_picker_handle_height;
        self.hue_picker_offset = hue_picker_offset;
        self.alpha_picker_offset = alpha_picker_offset;
        requires_triangulation
    }

    #[inline(always)]
    fn calc_color(&mut self, style: &Style<FontHash>) -> ColorHSVA {
        if self.alpha_changed() {
            let hsva = self.hsva;
            self.alpha_picker_handle_offset_x = self.offset.x + style.color_picker_size.x * hsva.alpha
                + style.item_pad_outer.x;
            return hsva
        }
        if self.r_changed() || self.g_changed() || self.b_changed() {
            self.srgba = self.rgba.to_srgba();
            self.hsva = self.srgba.to_hsva();
            let hsva = self.hsva;
            let offset = self.offset;
            let picker_size = style.color_picker_size;
            let item_pad_outer = style.item_pad_outer;
            self.picker_handle_offset = vec2(
                offset.x + picker_size.x * hsva.sat,
                offset.y + picker_size.y * (1.0 - hsva.val),
            ) + item_pad_outer;
            self.hue_picker_handle_offset_x = offset.x + picker_size.x * hsva.hue / TAU
                + item_pad_outer.x;
            return hsva
        }
        if self.hue_hovered() {
            self.srgba = self.hsva.to_srgba();
            let hsva = self.hsva;
            let offset = self.offset;
            let picker_size = style.color_picker_size;
            let item_pad_outer = style.item_pad_outer;
            self.picker_handle_offset = vec2(
                offset.x + picker_size.x * hsva.sat,
                offset.y + picker_size.y * (1.0 - hsva.val),
            ) + item_pad_outer;
            self.hue_picker_handle_offset_x = offset.x + picker_size.x * hsva.hue / TAU
                + item_pad_outer.x;
            return hsva
        }
        let picker_size = style.color_picker_size;
        let offset = self.offset;
        let item_pad_outer = style.item_pad_outer;
        let handle_offset = self.picker_handle_offset - self.offset - item_pad_outer;
        let t = vec2(
            handle_offset.x / picker_size.x,
            handle_offset.y / picker_size.y,
        );
        let mut color = ColorHSVA::new(
            (self.hue_picker_handle_offset_x - offset.x - item_pad_outer.x).clamp(0.0, picker_size.x)
                / picker_size.x * TAU,
            1.0,
            1.0,
            (self.alpha_picker_handle_offset_x - offset.x - item_pad_outer.x).clamp(0.0, picker_size.x)
                / picker_size.x,
        );
        color.val = lerp(1.0, 0.0, t.y);
        color.sat = lerp(0.0, 1.0, t.x);
        self.hsva = color;
        self.srgba = ColorSRGBA::from_hsva(color);
        color
    }

    #[inline(always)]
    fn set_color(&mut self, color: impl Color) {
        let hsva = color.to_hsva();
        let red = (hsva.hue - TAU).abs() < f32::EPSILON || hsva.hue.abs() < f32::EPSILON;
        if hsva.sat.abs() > f32::EPSILON && hsva.val.abs() > f32::EPSILON &&
            (red && (self.hsva.hue - hsva.hue).abs() < f32::EPSILON || !red)
        {
            self.hsva = hsva;
        }
        self.srgba = color.to_srgba();
        self.rgba = ColorRGBA::from_srgba(self.srgba);
    }

    fn triangulate(&mut self, style: &Style<FontHash>) {
        self.other_vertices.clear();
        self.indices.resize(
            self.indices.len() -
                (self.other_vertices_draw_info.index_count as usize +
                    self.other_vertices_draw_info_bg.index_count as usize
                ),
            Default::default()
        );
        let mut index_offset = self.indices.len() as u32;
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        let mut outline_points = GlobalVec::new();
        self.window_rect.to_points(&mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points,
            self.outline_width,
            false,
            &mut |p| { outline_points.push(p.into()); },
        );
        earcut::earcut(
            &outline_points, &[], false,
            &mut self.other_vertices, &mut indices_usize,
        ).unwrap();
        self.window_outline_vertex_range = VertexRange::new(0..self.other_vertices.len());
        let mut vertex_off = self.other_vertices.len();
        earcut::earcut(
            &points, &[], false,
            &mut self.other_vertices, &mut indices_usize,
        ).unwrap();
        self.window_vertex_range = VertexRange::new(vertex_off..self.other_vertices.len());
        points.clear();
        outline_points.clear();
        self.text_box_rect.to_points(&mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points, self.outline_width, false,
            &mut |p| { outline_points.push(p.into()); },
        );
        let mut tmp_vertices = GlobalVec::new();
        let mut tmp_indices = GlobalVec::new();
        earcut::earcut(
            &outline_points, &[], false,
            &mut tmp_vertices, &mut tmp_indices,
        ).unwrap();
        let mut f = || -> VertexRange {
            vertex_off = self.other_vertices.len();
            self.other_vertices.append(&tmp_vertices);
            indices_usize.append_map(&tmp_indices, |&i| vertex_off + i);
            VertexRange::new(vertex_off..self.other_vertices.len())
        };
        self.r_text_box_outline_vertex_range = f();
        self.g_text_box_outline_vertex_range = f();
        self.b_text_box_outline_vertex_range = f();
        self.alpha_text_box_outline_vertex_range = f();
        self.hue_text_box_outline_vertex_range = f();
        tmp_vertices.clear();
        tmp_indices.clear();
        earcut::earcut(
            &points, &[], false,
            &mut tmp_vertices, &mut tmp_indices,
        ).unwrap();
        let mut f = || -> VertexRange {
            vertex_off = self.other_vertices.len();
            self.other_vertices.append(&tmp_vertices);
            indices_usize.append_map(&tmp_indices, |&i| vertex_off + i);
            VertexRange::new(vertex_off..self.other_vertices.len())
        };
        self.r_text_box_vertex_range = f();
        self.g_text_box_vertex_range = f();
        self.b_text_box_vertex_range = f();
        self.alpha_text_box_vertex_range = f();
        self.hue_text_box_vertex_range = f();
        self.indices.append_map(&indices_usize, |&i| i as u32);
        self.other_vertices_draw_info_bg = DrawInfo {
            first_index: index_offset,
            index_count: self.indices.len() as u32 - index_offset,
            ..Default::default()
        };
        index_offset = self.indices.len() as u32;
        points.clear();
        indices_usize.clear();
        outline_points.clear();
        self.picker_handle.to_points(16, &mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points,
            style.outline_width,
            false,
            &mut |p| { outline_points.push(p.into()); }
        );
        vertex_off = self.other_vertices.len();
        earcut::earcut(
            &outline_points, &[], false,
            &mut self.other_vertices, &mut indices_usize
        ).unwrap();
        self.picker_handle_outline_vertex_range = VertexRange::new(vertex_off..self.other_vertices.len());
        vertex_off = self.other_vertices.len();
        earcut::earcut(
            &points, &[], false,
            &mut self.other_vertices, &mut indices_usize
        ).unwrap();
        self.picker_handle_vertex_range = VertexRange::new(vertex_off..self.other_vertices.len());
        points.clear();
        let hue_alpha_picker_handle_height = self.hue_alpha_picker_handle_height;
        let half_width = hue_alpha_picker_handle_height * 0.5;
        vertex_off = self.other_vertices.len();
        self.other_vertices.push([0.0, hue_alpha_picker_handle_height].into());
        self.other_vertices.push([-half_width, 0.0].into());
        self.other_vertices.push([half_width, 0.0].into());
        indices_usize.append(&[vertex_off, vertex_off + 1, vertex_off + 2]);
        self.hue_picker_handle_vertex_range = VertexRange::new(vertex_off..self.other_vertices.len());
        vertex_off += 3;
        self.other_vertices.push([0.0, self.hue_alpha_picker_handle_height].into());
        self.other_vertices.push([-half_width, 0.0].into());
        self.other_vertices.push([half_width, 0.0].into());
        indices_usize.append(&[vertex_off, vertex_off + 1, vertex_off + 2]);
        self.alpha_picker_handle_vertex_range = VertexRange::new(vertex_off..self.other_vertices.len());
        self.indices.append_map(&indices_usize, |&i| i as u32);
        self.other_vertices_draw_info = DrawInfo {
            first_index: index_offset,
            index_count: self.indices.len() as u32 - index_offset,
            ..Default::default()
        };
    }

    fn set_vertex_params(&mut self, style: &Style<FontHash>) {
        let hue_picker_black =
            if self.hsva.hue > PI * 0.05 && self.hsva.hue < PI * 1.2 {
                true
            } else {
                false
            };
        let picker_size = style.color_picker_size;
        let mut offset = self.offset;
        let mut target_color = style.hover_window_bg_col;
        set_vertex_params(&mut self.other_vertices, self.window_vertex_range, offset, target_color);
        target_color = style.window_outline_thin_col;
        set_vertex_params(&mut self.other_vertices, self.window_outline_vertex_range, offset, target_color);
        let item_pad_outer = style.item_pad_outer;
        let text_box_height = style.calc_text_box_height(self.r_text.row_height);
        let tmp_offset = vec2(
            self.offset.x + item_pad_outer.x + picker_size.x + item_pad_outer.x,
            self.offset.y + item_pad_outer.y,
        );
        let hue_text_box_offset = self.hue_text_box_offset;
        offset = tmp_offset;
        target_color = style.on_top_contents_widget_bg_col;
        set_vertex_params(&mut self.other_vertices, self.r_text_box_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        set_vertex_params(&mut self.other_vertices, self.g_text_box_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        set_vertex_params(&mut self.other_vertices, self.b_text_box_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        set_vertex_params(&mut self.other_vertices, self.alpha_text_box_vertex_range, offset, target_color);
        offset = hue_text_box_offset;
        set_vertex_params(&mut self.other_vertices, self.hue_text_box_vertex_range, offset, target_color);
        offset = tmp_offset;
        target_color = if self.r_changed() {
            style.on_top_contents_widget_outline_col_hl
        } else if self.r_hovered() {
            style.on_top_contents_widget_outline_col
        } else {
            ColorSRGBA::black(0.0)
        };
        set_vertex_params(&mut self.other_vertices, self.r_text_box_outline_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        target_color = if self.g_changed() {
            style.on_top_contents_widget_outline_col_hl
        } else if self.g_hovered() {
            style.on_top_contents_widget_outline_col
        } else {
            ColorSRGBA::black(0.0)
        };
        set_vertex_params(&mut self.other_vertices, self.g_text_box_outline_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        target_color = if self.b_changed() {
            style.on_top_contents_widget_outline_col_hl
        } else if self.b_hovered() {
            style.on_top_contents_widget_outline_col
        } else {
            ColorSRGBA::black(0.0)
        };
        set_vertex_params(&mut self.other_vertices, self.b_text_box_outline_vertex_range, offset, target_color);
        offset.y += text_box_height + item_pad_outer.y;
        target_color = if self.alpha_changed() {
            style.on_top_contents_widget_outline_col_hl
        } else if self.alpha_hovered() {
            style.on_top_contents_widget_outline_col
        } else {
            ColorSRGBA::black(0.0)
        };
        set_vertex_params(&mut self.other_vertices, self.alpha_text_box_outline_vertex_range, offset, target_color);
        offset = hue_text_box_offset; 
        target_color = if self.hue_changed() {
            style.on_top_contents_widget_outline_col_hl
        } else if self.hue_hovered() {
            style.on_top_contents_widget_outline_col
        } else {
            ColorSRGBA::black(0.0)
        };
        set_vertex_params(&mut self.other_vertices, self.hue_text_box_outline_vertex_range, offset, target_color);
        let hsva = self.hsva;
        let srgba = self.srgba;
        let tmp = hsva.val > 0.5;
        let sat_low = hsva.sat < 0.5;
        let picker_handle_col =
        if sat_low && tmp ||
            (tmp && hue_picker_black)
        {
            ColorSRGBA::black(1.0)
        } else {
            ColorSRGBA::white(1.0)
        };
        offset = self.picker_handle_offset;
        target_color = srgba.with_alpha(1.0);
        set_vertex_params(&mut self.other_vertices, self.picker_handle_vertex_range, offset, target_color);
        target_color = picker_handle_col;
        set_vertex_params(&mut self.other_vertices,
            self.picker_handle_outline_vertex_range, offset, target_color);
        target_color =
            if hue_picker_black {
                ColorSRGBA::black(1.0)
            } else {
                ColorSRGBA::white(1.0)
            };
        offset = vec2(
            self.hue_picker_handle_offset_x,
            self.offset.y + picker_size.y + item_pad_outer.y + item_pad_outer.y,
        );
        set_vertex_params(&mut self.other_vertices, self.hue_picker_handle_vertex_range, offset, target_color);
        target_color =
            if srgba.alpha < 0.4 || (!hue_picker_black && tmp && !sat_low) || !tmp   {
                ColorSRGBA::white(1.0)
            } else {
                ColorSRGBA::black(1.0)
            };
        offset = vec2(
            self.alpha_picker_handle_offset_x,
            self.offset.y + picker_size.y + picker_size.y * 0.1 +
                item_pad_outer.y + item_pad_outer.y + item_pad_outer.y,
        );
        set_vertex_params(&mut self.other_vertices, self.alpha_picker_handle_vertex_range, offset, target_color);
    }
}

impl<I, FontHash> OnTopContents<I, FontHash> for Contents<I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error> {
        let picker_vertex_count = self.picker_vertices.len();
        let other_vertex_count = self.other_vertices.len();
        let index_count = self.indices.len();
        let picker_vert_mem = unsafe { vertex_buffer
            .allocate(render_commands, picker_vertex_count)?
        };
        let other_vert_mem = unsafe { vertex_buffer
            .allocate(render_commands, other_vertex_count)?
        };
        let index_mem = unsafe { index_buffer
            .allocate(render_commands, index_count)?
        };
        unsafe {
            self.picker_vertices
                .as_ptr()
                .copy_to_nonoverlapping(picker_vert_mem.ptr.as_ptr(), picker_vertex_count);
            self.other_vertices
                .as_ptr()
                .copy_to_nonoverlapping(other_vert_mem.ptr.as_ptr(), other_vertex_count);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(index_mem.ptr.as_ptr(), index_count);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos,
            vec2(1.0, 1.0),
            inv_aspect_ratio
        );
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            self.other_vertices_draw_info_bg,
            [
                DrawBufferInfo {
                    id: vertex_buffer.id(),
                    offset: other_vert_mem.offset,
                },
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: index_mem.offset,
            },
        )?;
        render_commands.bind_pipeline(get_custom_pipeline(COLOR_PICKER_PIPELINE_HASH).unwrap())?;
        let picker_size = style.color_picker_size;
        let item_pad_outer = style.item_pad_outer;
        let item_pad_inner = style.item_pad_inner;
        let offset = self.offset;
        let hsva = self.hsva;
        let pc_vertex = push_constants_vertex(
            window_pos + offset + item_pad_outer,
            picker_size,
            inv_aspect_ratio,
        );
        let pc_fragment = color_picker_push_constants_fragments(
            hsva.hue,
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        let vertex_buffer_info = DrawBufferInfo {
            id: vertex_buffer.id(),
            offset: picker_vert_mem.offset,
        };
        let index_buffer_info = DrawBufferInfo {
            id: index_buffer.id(),
            offset: index_mem.offset,
        };
        render_commands.draw_indexed(
            self.picker_draw_info,
            [vertex_buffer_info],
            index_buffer_info,
        )?;
        render_commands.bind_pipeline(get_custom_pipeline(COLOR_PICKER_HUE_PIPELINE_HASH).unwrap())?;
        let hue_picker_size_y = picker_size.y * 0.1;
        let pc_vertex = push_constants_vertex(
            window_pos + offset + vec2(0.0, picker_size.y + item_pad_outer.y) + item_pad_outer,
            vec2(picker_size.x, hue_picker_size_y),
            inv_aspect_ratio
        );
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            self.picker_draw_info,
            [vertex_buffer_info],
            index_buffer_info,
        )?;
        render_commands.bind_pipeline(get_custom_pipeline(COLOR_PICKER_ALPHA_PIPELINE_HASH).unwrap())?;
        let pc_vertex = push_constants_vertex(
            window_pos + offset + vec2(0.0, picker_size.y + hue_picker_size_y + item_pad_outer.y + item_pad_outer.y) +
                item_pad_outer,
            vec2(picker_size.x, hue_picker_size_y),
            inv_aspect_ratio
        );
        let pc_fragment = aplha_picker_push_constants_fragment(hsva, picker_size.x, style.alpha_tile_width);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.picker_draw_info,
            [vertex_buffer_info],
            index_buffer_info
        )?;
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos,
            vec2(1.0, 1.0),
            inv_aspect_ratio
        );
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            self.other_vertices_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer.id(),
                    offset: other_vert_mem.offset,
                },
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: index_mem.offset,
            },
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let text_offset = window_pos + vec2(
            offset.x + item_pad_outer.x + picker_size.x + item_pad_outer.x,
            offset.y + item_pad_outer.y,
        );
        let text_height = style.calc_text_height(self.r_text.row_height);
        let font_scale = vec2(style.font_scale, style.font_scale);
        let text_box_size = self.text_box_rect.max;
        let mut pc_vertex = push_constants_vertex(
            text_offset,
            font_scale,
            inv_aspect_ratio
        );
        let pc_fragment = text_push_constants_fragment(style.text_col);
        let mut f = |text: &RenderedText| -> Result<(), Error> {
            let text_width = style.calc_text_width(text.text_width);
            pc_vertex.vert_off.x = text_offset.x + text_box_size.x * 0.5 - text_width * 0.5;
            pc_vertex.vert_off.y += item_pad_inner.y;
            render_text(render_commands, &text, pc_vertex, pc_fragment, vertex_buffer, index_buffer)?;
            pc_vertex.vert_off.y += text_height + item_pad_outer.y + item_pad_inner.y;
            Ok(())
        };
        f(&self.r_text)?;
        f(&self.g_text)?;
        f(&self.b_text)?;
        f(&self.alpha_text)?;
        let hue_text_box_offset = self.hue_text_box_offset;
        let text_width = style.calc_text_width(self.hue_text.text_width);
        pc_vertex.vert_off = window_pos + vec2(
            hue_text_box_offset.x + text_box_size.x * 0.5 - text_width * 0.5,
            hue_text_box_offset.y + item_pad_inner.y,
        );
        render_text(render_commands, &self.hue_text, pc_vertex, pc_fragment, vertex_buffer, index_buffer)?;
        Ok(())
    }
}

pub(crate) struct ColorPicker<I, FontHash> {
    title: CompactString,
    title_text: Option<RenderedText>,
    color_rect: Rect,
    color_rect_vertex_range: VertexRange,
    contents: Contents<I, FontHash>,
    offset: Vec2,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> ColorPicker<I, FontHash> {

    #[inline(always)]
    pub fn new(title: &str) -> Self {
        Self {
            title: CompactString::new(title),
            title_text: None,
            color_rect: Default::default(),
            color_rect_vertex_range: Default::default(),
            contents: Contents::new(),
            offset: Default::default(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn picking(&self) -> bool {
        self.contents.picker_held() ||
        self.contents.hue_picker_held() ||
        self.contents.alpha_picker_held() ||
        self.contents.r_changed() ||
        self.contents.g_changed() ||
        self.contents.b_changed() ||
        self.contents.hue_hovered() ||
        self.contents.alpha_changed()
    }

    #[inline(always)]
    pub fn set_color(&mut self, color: impl Color) {
        self.contents.set_color(color);
    }

    #[inline(always)]
    pub fn calc_color(&mut self, style: &Style<FontHash>) -> ColorHSVA {
        self.contents.calc_color(style)
    }
}

impl<I, FontHash> Widget<I, FontHash> for ColorPicker<I, FontHash>
    where 
        FontHash: Clone + Eq + Hash,
        I: Interface,
{

    #[inline(always)]
    fn hover_text(&self) -> Option<&str> {
        None
    }

    #[inline(always)]
    fn set_offset(
        &mut self,
        offset: nox_geom::Vec2,
    ) {
        self.offset = offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
    ) -> Vec2 {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(&self.title, &style.font_regular)], false, 0.0).unwrap_or_default()
        );
        style.calc_text_size(vec2(title_text.text_width, title_text.row_height))
    }

    fn is_active(&self, style: &Style<FontHash>, window_pos: Vec2, cursor_pos: Vec2) -> bool {
        let error_margin = style.cursor_error_margin;
        let error_margin_2 = style.cursor_error_margin + style.cursor_error_margin;
        self.picking() || (self.contents.shown() && BoundingRect::from_position_size(
            self.contents.offset - vec2(error_margin, error_margin),
            self.contents.window_rect.max + vec2(error_margin_2, error_margin_2)
        ).is_point_inside(cursor_pos - window_pos))
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        other_widget_active: bool,
    ) -> UpdateResult {
        let title_text = self.title_text.as_ref().unwrap();
        let text_size = style.calc_text_size(vec2(title_text.text_width, title_text.row_height));
        let color_rect_max = vec2(text_size.y, text_size.y);
        let requires_triangulation = self.color_rect.max != color_rect_max;
        self.color_rect.max = color_rect_max;
        let color_rect_off_x = text_size.x + style.item_pad_outer.x;
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = style.cursor_error_margin;
        let error_margin_2 = error_margin + error_margin;
        let cursor_in_color_rect =
            BoundingRect::from_position_size(
                self.offset + vec2(color_rect_off_x, 0.0) - vec2(error_margin, error_margin),
                color_rect_max + vec2(error_margin_2, error_margin_2),
            ).is_point_inside(rel_cursor_pos);
        let cursor_in_contents = self.contents
            .bounding_rect(error_margin)
            .is_point_inside(rel_cursor_pos);
        if self.contents.widget_held() {
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.contents.set_widget_held(false);
                self.contents.set_shown(cursor_in_color_rect);
            }
        }
        if nox.was_mouse_button_pressed(MouseButton::Left) {
            if self.contents.shown() {
                self.contents.set_shown(cursor_in_contents);
            }
            if cursor_in_color_rect {
                self.contents.set_widget_held(true);
            }
        }
        if other_widget_active {
            self.contents.set_shown(false);
        }
        let shown = self.contents.shown();
        if shown {
            self.contents.offset = self.offset + vec2(
                text_size.x + style.item_pad_outer.x,
                text_size.y + style.outline_width,
            );
            if self.contents.update(
                nox,
                style,
                text_renderer,
                window_pos,
                cursor_pos,
                delta_cursor_pos,
            ) {
                self.contents.triangulate(style);
            }
        }
        UpdateResult {
            min_widget_width: text_size.x,
            requires_triangulation: requires_triangulation,
            cursor_in_widget: (shown && cursor_in_contents) || cursor_in_color_rect || self.picking(),
        }
    }

    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.color_rect.to_points(&mut |p| { points.push(p.into()); });
        self.color_rect_vertex_range = tri(points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style<FontHash>,
        vertices: &mut [Vertex],
    ) {
        if self.contents.shown() {
            self.contents.set_vertex_params(style);
        }
        let title_text = self.title_text.as_ref().unwrap();
        let offset = self.offset + vec2(
            style.calc_text_width(title_text.text_width) + style.item_pad_outer.x,
            0.0,
        );
        let target_color = self.contents.srgba.with_alpha(1.0);
        let vertex_sample = vertices[self.color_rect_vertex_range.start()];
        if vertex_sample.offset != offset || vertex_sample.color != target_color {
            for vertex in &mut vertices[self.color_rect_vertex_range.range()] {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        _base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn OnTopContents<I, FontHash>>, Error>
    {
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.offset,
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio,
        );
        let pc_fragment = text_push_constants_fragment(style.text_col);
        render_text(render_commands, self.title_text.as_ref().unwrap(),
            pc_vertex, pc_fragment, vertex_buffer, index_buffer)?;
        if self.contents.shown() {
            Ok(Some(&self.contents))
        } else {
            Ok(None)
        }
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    )
    {
        let vertex_sample = vertices[self.color_rect_vertex_range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[self.color_rect_vertex_range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
        }
    }
}
