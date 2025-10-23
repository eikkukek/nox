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

use nox_font::{text_segment, RenderedText, CombinedRenderedText};
use nox_geom::{
    shapes::*,
    *
};

use crate::*;

struct Contents<I, FontHash, Style> {
    r_drag_value: DragValue<EmptyText, I, FontHash, Style>,
    g_drag_value: DragValue<EmptyText, I, FontHash, Style>,
    b_drag_value: DragValue<EmptyText, I, FontHash, Style>,
    alpha_drag_value: DragValue<EmptyText, I, FontHash, Style>,
    hue_drag_value: DragValue<EmptyText, I, FontHash, Style>,
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
    picker_handle_radius: f32,
    hue_alpha_picker_handle_height: f32,
    picker_vertices: GlobalVec<ColorPickerVertex>,
    other_vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    other_vertices_draw_info_bg: DrawInfo,
    other_vertices_draw_info: DrawInfo,
    picker_draw_info: DrawInfo,
    window_vertex_range: VertexRange,
    window_outline_vertex_range: VertexRange,
    picker_handle_vertex_range: VertexRange,
    picker_handle_outline_vertex_range: VertexRange,
    hue_picker_handle_vertex_range: VertexRange,
    alpha_picker_handle_vertex_range: VertexRange,
    combined_text: CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>,
    outline_width: f32,
    focused_outline_width: f32,
    rgba_text_size: Vec2,
    flags: u32,
    _marker: PhantomData<(I, FontHash, Style)>,
}

impl<I, FontHash, Style> Contents<I, FontHash, Style>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    const WIDGET_HELD: u32 = 0x1;
    const SHOWN: u32 = 0x2;
    const PICKER_HELD: u32 = 0x4;
    const HUE_PICKER_HELD: u32 = 0x8;
    const ALPHA_PICKER_HELD: u32 = 0x10;
    const FONT_CHANGED: u32 = 0x20;
    const R_CHANGED: u32 = 0x40;
    const G_CHANGED: u32 = 0x80;
    const B_CHANGED: u32 = 0x100;
    const ALPHA_CHANGED: u32 = 0x200;
    const HUE_CHANGED: u32 = 0x400;
    const CLICKED: u32 = 0x800;
    const DRAG_VALUE_ACTIVE: u32 = 0x1000;

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
            r_drag_value: DragValue::new(""),
            g_drag_value: DragValue::new(""),
            b_drag_value: DragValue::new(""),
            alpha_drag_value: DragValue::new(""),
            hue_drag_value: DragValue::new(""),
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
            picker_handle_radius: 0.0,
            hue_alpha_picker_handle_height: 0.0,
            picker_vertices,
            other_vertices: Default::default(),
            indices,
            picker_draw_info,
            other_vertices_draw_info_bg: Default::default(),
            other_vertices_draw_info: Default::default(),
            window_vertex_range: Default::default(),
            window_outline_vertex_range: Default::default(),
            picker_handle_vertex_range: Default::default(),
            picker_handle_outline_vertex_range: Default::default(),
            hue_picker_handle_vertex_range: Default::default(),
            alpha_picker_handle_vertex_range: Default::default(),
            combined_text: CombinedRenderedText::new(),
            outline_width: 0.0,
            focused_outline_width: 0.0,
            rgba_text_size: Default::default(),
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
    fn r_changed(&self) -> bool {
        self.flags & Self::R_CHANGED == Self::R_CHANGED
    }

    #[inline(always)]
    fn g_changed(&self) -> bool {
        self.flags & Self::G_CHANGED == Self::G_CHANGED
    }

    #[inline(always)]
    fn b_changed(&self) -> bool {
        self.flags & Self::B_CHANGED == Self::B_CHANGED
    }

    #[inline(always)]
    fn alpha_changed(&self) -> bool {
        self.flags & Self::ALPHA_CHANGED == Self::ALPHA_CHANGED
    }

    #[inline(always)]
    fn hue_changed(&self) -> bool {
        self.flags & Self::HUE_CHANGED == Self::HUE_CHANGED
    }

    #[inline(always)]
    fn drag_value_active(&self) -> bool {
        self.flags & Self::DRAG_VALUE_ACTIVE == Self::DRAG_VALUE_ACTIVE
    }

    #[inline(always)]
    fn drag_value_changed(&self) -> bool {
        self.r_changed() ||
        self.g_changed() ||
        self.b_changed() ||
        self.alpha_changed() ||
        self.hue_changed()
    }

    #[inline(always)]
    fn font_changed(&self) -> bool {
        self.flags & Self::FONT_CHANGED == Self::FONT_CHANGED
    }

    #[inline(always)]
    fn clicked(&self) -> bool {
        self.flags & Self::CLICKED == Self::CLICKED
    }

    #[inline(always)]
    fn set_clicked(&mut self, value: bool) {
        self.flags &= !Self::CLICKED;
        self.flags |= value as u32;
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
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        window_moving: bool,
    ) -> bool
        where
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let item_pad_outer = style.item_pad_outer();
        let mut text_box_rect_max = vec2(self.rgba_text_size.x, self.r_drag_value.calc_height(style, text_renderer));
        if self.font_changed() {
            let samples = (
                text_renderer.render(
                    &[text_segment("R 255", &style.font_regular())], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("G 255", &style.font_regular())], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("B 255", &style.font_regular())], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("A 255", &style.font_regular())], false, 0.0
                ).unwrap_or_default(),
                text_renderer.render(
                    &[text_segment("H 360°", &style.font_regular())], false, 0.0
                ).unwrap_or_default(),
            );
            let rgba_text_size_x = style.calc_text_box_width_from_text_width(
                samples.0.text_width
                    .max(samples.1.text_width)
                    .max(samples.2.text_width)
                    .max(samples.3.text_width)
                    .max(samples.4.text_width) * style.font_scale()
            );
            let text_box_height = style.calc_text_box_height(&samples.0);
            let rgba_text_size_y =
                text_box_height * 4.0 +
                item_pad_outer.y * 5.0;
            self.rgba_text_size = vec2(rgba_text_size_x, rgba_text_size_y);
            text_box_rect_max.x = rgba_text_size_x;
            text_box_rect_max.y = text_box_height;
            self.flags &= !Self::FONT_CHANGED;
        } 
        let offset = self.offset;
        let picker_size = style.color_picker_size();
        let hue_alpha_picker_height = picker_size.y * 0.1;
        let item_pad_outer = style.item_pad_outer();
        let hue_picker_offset = vec2(
            offset.x,
            offset.y + (picker_size.y + item_pad_outer.y),
        ) + item_pad_outer;
        let alpha_picker_offset = hue_picker_offset +
            vec2(0.0, hue_alpha_picker_height + item_pad_outer.y);
        let color_picker_offset = offset + item_pad_outer;
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = vec2(style.cursor_error_margin(), style.cursor_error_margin());
        let error_margin_2 = error_margin + error_margin;
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        if self.picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::PICKER_HELD;
            } else {
                self.picker_handle_offset = rel_cursor_pos.clamp(
                    color_picker_offset,
                    color_picker_offset + picker_size 
                );
            }
        }
        else if self.hue_picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HUE_PICKER_HELD;
            } else {
                self.hue_picker_handle_offset_x = rel_cursor_pos.x.clamp(
                    hue_picker_offset.x,
                    hue_picker_offset.x + picker_size.x,
                );
            }
        }
        else if self.alpha_picker_held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::ALPHA_PICKER_HELD;
            } else {
                self.alpha_picker_handle_offset_x = rel_cursor_pos.x.clamp(
                    alpha_picker_offset.x,
                    alpha_picker_offset.x + picker_size.x,
                );
            }
        }
        else if mouse_pressed {
            if BoundingRect::from_min_max(
                    color_picker_offset - error_margin,
                    color_picker_offset + picker_size + error_margin
                )
                .is_point_inside(rel_cursor_pos)
            {
                self.flags |= Self::PICKER_HELD;
            }
            else if BoundingRect::from_position_size(
                    hue_picker_offset - error_margin,
                    vec2(picker_size.x, hue_alpha_picker_height) +
                        error_margin,
                )
                .is_point_inside(rel_cursor_pos)
            {
                self.flags |= Self::HUE_PICKER_HELD;
            }
            else if BoundingRect::from_position_size(
                    alpha_picker_offset - error_margin,
                    vec2(picker_size.x, hue_alpha_picker_height) +
                        error_margin_2
                )
                .is_point_inside(rel_cursor_pos)
            {
                self.flags |= Self::ALPHA_PICKER_HELD;
            }
        } else if !self.drag_value_changed() {
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
        self.flags &= !(
            Self::R_CHANGED | Self::G_CHANGED | Self::B_CHANGED |
            Self::ALPHA_CHANGED | Self::HUE_CHANGED
        );
        let rgba_text_size = self.rgba_text_size;
        let hue_text_box_offset_y = 
            (hue_picker_offset.y + picker_size.y * 0.05 - text_box_rect_max.y * 0.5)
            .max(offset.y + self.rgba_text_size.y);
        let hue_text_box_max_y = hue_text_box_offset_y - offset.y + text_box_rect_max.y + item_pad_outer.y;
        let mut window_rect_max = item_pad_outer + item_pad_outer + picker_size +
            vec2(
                item_pad_outer.x + rgba_text_size.x,
                hue_alpha_picker_height + item_pad_outer.y +
                hue_alpha_picker_height + item_pad_outer.y
            );
        window_rect_max.y = window_rect_max.y.max(hue_text_box_max_y); 
        let cursor_in_window = BoundingRect::from_position_size(
            offset,
            window_rect_max,
        ).is_point_inside(rel_cursor_pos);
        let mut drag_value_offset = offset + vec2(item_pad_outer.x + picker_size.x + item_pad_outer.x, item_pad_outer.x);
        self.r_drag_value.set_offset(drag_value_offset);
        drag_value_offset.y += self.r_drag_value.calc_height(style, text_renderer) + item_pad_outer.y;
        self.g_drag_value.set_offset(drag_value_offset);
        drag_value_offset.y += self.g_drag_value.calc_height(style, text_renderer) + item_pad_outer.y;
        self.b_drag_value.set_offset(drag_value_offset);
        drag_value_offset.y += self.b_drag_value.calc_height(style, text_renderer) + item_pad_outer.y;
        self.alpha_drag_value.set_offset(drag_value_offset);
        drag_value_offset.y = hue_text_box_offset_y;
        self.hue_drag_value.set_offset(drag_value_offset);
        let drag_value_active = 
            if self.picker_held() || self.hue_picker_held() || self.alpha_picker_held() {
                None
            }
            else if self.r_drag_value.is_active(nox, style, window_pos, cursor_pos) {
                Some(0)
            } else if self.g_drag_value.is_active(nox, style, window_pos, cursor_pos) {
                Some(1)
            } else if self.b_drag_value.is_active(nox, style, window_pos, cursor_pos) {
                Some(2)
            } else if self.alpha_drag_value.is_active(nox, style, window_pos, cursor_pos) {
                Some(3)
            } else if self.hue_drag_value.is_active(nox, style, window_pos, cursor_pos) {
                Some(4)
            } else {
                None
            };
        self.flags &= !Self::DRAG_VALUE_ACTIVE;
        self.flags |= Self::DRAG_VALUE_ACTIVE * drag_value_active.is_some() as u32;
        if cursor_in_window && style.override_cursor() && drag_value_active.is_none() {
            nox.set_cursor(CursorIcon::Default);
        }
        self.r_drag_value.set_input_params(
            style,
            text_box_rect_max.x, true,
            Some(
                |fmt, str| -> core::fmt::Result {
                    write!(fmt, "R {}", str)
                }
            )
        );
        let mut val = self.rgba.r;
        self.r_drag_value.calc_value(style, &mut val, 0, 255, style.default_value_drag_speed() * 255.0);
        self.flags |= Self::R_CHANGED * (self.rgba.r != val) as u32;
        self.rgba.r = val;
        self.combined_text.clear();
        let font_scale = style.font_scale();
        let mut update_result = self.r_drag_value.update(
            nox, style,
            text_renderer, window_rect_max, window_pos,
            cursor_pos, delta_cursor_pos, cursor_in_window,
            if let Some(cursor_in_drag_value) = drag_value_active {
                cursor_in_drag_value != 0
            } else {
                false
            },
            window_moving,
            &mut |text, offset, bounded_instance| self.combined_text.add_text(text, offset / font_scale, bounded_instance).unwrap(),
        );
        let mut f = |
                drag_value: &mut DragValue<EmptyText, I, FontHash, Style>,
                idx: usize,
                format_result: fn(&mut dyn Write, &str) -> core::fmt::Result,
            |
        {
            drag_value.set_input_params(style, text_box_rect_max.x, true, Some(format_result));
            let res = drag_value.update(nox, style,
                text_renderer, window_rect_max, window_pos,
                cursor_pos, delta_cursor_pos, cursor_in_window,
                if let Some(cursor_in_drag_value) = drag_value_active {
                    cursor_in_drag_value != idx
                } else {
                    false
                },
                window_moving,
                &mut |text, offset, bounded_text_instance| self.combined_text.add_text(text, offset / font_scale, bounded_text_instance).unwrap(),
            );
            update_result.min_window_width = update_result.min_window_width.max(res.min_window_width);
            update_result.cursor_in_widget |= res.cursor_in_widget;
            update_result.requires_triangulation |= res.requires_triangulation;
        };

        let mut rgba = self.rgba;

        let mut val = rgba.g;
        self.g_drag_value.calc_value(style, &mut val, 0, 255, style.default_value_drag_speed() * 255.0);
        self.flags |= Self::G_CHANGED * (rgba.g != val) as u32;
        rgba.g = val;
        f(&mut self.g_drag_value, 1, |fmt, str| { write!(fmt, "G {}", str) });

        let mut val = rgba.b;
        self.b_drag_value.calc_value(style, &mut val, 0, 255, style.default_value_drag_speed() * 255.0);
        self.flags |= Self::B_CHANGED * (rgba.b != val) as u32;
        rgba.b = val;
        f(&mut self.b_drag_value, 2, |fmt, str| { write!(fmt, "B {}", str) });

        self.rgba = rgba;

        let mut hsva = self.hsva;

        let mut val = hsva.alpha;
        self.alpha_drag_value.calc_and_map_value(style, &mut val, 0.0, 1.0, style.default_value_drag_speed(),
            |t| (t * 255.0).round() as u8,
            |t| t as f32 / 255.0,
        );
        self.flags |= Self::ALPHA_CHANGED * (hsva.alpha != val) as u32;
        hsva.alpha = val;
        f(&mut self.alpha_drag_value, 3, |fmt, str| { write!(fmt, "A {}", str) });

        let mut val = hsva.hue;
        self.hue_drag_value.calc_and_map_value(style, &mut val, 0.0, TAU, style.default_value_drag_speed() * TAU,
            |t| (t * 180.0 / PI).round() as u32,
            |t| (t as f32 * PI / 180.0).clamp(0.0, TAU),
        );
        self.flags |= Self::HUE_CHANGED * (hsva.hue != val) as u32;
        hsva.hue = val;
        f(&mut self.hue_drag_value, 4, |fmt, str| { write!(fmt, "H {}°", str) });

        self.hsva = hsva;

        window_rect_max.x = update_result.min_window_width - offset.x;

        if mouse_pressed && cursor_in_window
        {
            self.flags |= Self::CLICKED;
        }
        let handle_radius = style.default_handle_radius();
        let rounding = style.rounding();
        let outline_width = style.outline_width();
        let focused_outline_width = style.focused_outline_width();
        let hue_alpha_picker_handle_height = picker_size.y * 0.06;
        let requires_triangulation =
            self.window_rect.max != window_rect_max ||
            self.window_rect.rounding != rounding ||
            self.outline_width != outline_width ||
            self.focused_outline_width != focused_outline_width ||
            self.picker_handle_radius != handle_radius ||
            self.hue_alpha_picker_handle_height != hue_alpha_picker_handle_height;
        self.window_rect.max = window_rect_max;
        self.window_rect.rounding = rounding;
        self.outline_width = outline_width;
        self.focused_outline_width = focused_outline_width;
        self.picker_handle_radius = handle_radius;
        self.hue_alpha_picker_handle_height = hue_alpha_picker_handle_height;
        self.hue_picker_offset = hue_picker_offset;
        self.alpha_picker_offset = alpha_picker_offset;
        requires_triangulation | update_result.requires_triangulation
    }

    #[inline(always)]
    fn calc_color(&mut self, style: &Style) -> ColorHSVA {
        let picker_size = style.color_picker_size();
        if self.alpha_changed() {
            let hsva = self.hsva;
            self.alpha_picker_handle_offset_x = self.offset.x + picker_size.x * hsva.alpha
                + style.item_pad_outer().x;
            return hsva
        }
        if self.r_changed() || self.g_changed() || self.b_changed() {
            self.srgba = self.rgba.to_srgba();
            self.hsva = self.srgba.to_hsva();
            let hsva = self.hsva;
            let offset = self.offset;
            let item_pad_outer = style.item_pad_outer();
            self.picker_handle_offset = vec2(
                offset.x + picker_size.x * hsva.sat,
                offset.y + picker_size.y * (1.0 - hsva.val),
            ) + item_pad_outer;
            self.hue_picker_handle_offset_x = offset.x + picker_size.x * hsva.hue / TAU
                + item_pad_outer.x;
            return hsva
        }
        if self.hue_changed() {
            self.srgba = self.hsva.to_srgba();
            let hsva = self.hsva;
            let offset = self.offset;
            let item_pad_outer = style.item_pad_outer();
            self.picker_handle_offset = vec2(
                offset.x + picker_size.x * hsva.sat,
                offset.y + picker_size.y * (1.0 - hsva.val),
            ) + item_pad_outer;
            self.hue_picker_handle_offset_x = offset.x + picker_size.x * hsva.hue / TAU
                + item_pad_outer.x;
            return hsva
        }
        let offset = self.offset;
        let item_pad_outer = style.item_pad_outer();
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
        self.rgba = ColorRGBA::from_srgba(self.srgba);
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

    fn triangulate(&mut self) {
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
        let mut tri = |points: &[[f32; 2]]| -> VertexRange {
            let vertex_begin = self.other_vertices.len();
            earcut::earcut(points, &[], false, &mut self.other_vertices, &mut indices_usize).unwrap();
            VertexRange::new(vertex_begin..self.other_vertices.len())
        };
        self.r_drag_value.triangulate(&mut points, &mut tri);
        self.g_drag_value.triangulate(&mut points, &mut tri);
        self.b_drag_value.triangulate(&mut points, &mut tri);
        self.alpha_drag_value.triangulate(&mut points, &mut tri);
        self.hue_drag_value.triangulate(&mut points, &mut tri);
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
        circle(vec2(0.0, 0.0), self.picker_handle_radius).to_points(16, &mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points,
            self.focused_outline_width,
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

    fn set_vertex_params(&mut self, style: &Style) {
        let hue_picker_black =
            if self.hsva.hue > PI * 0.05 && self.hsva.hue < PI * 1.2 {
                true
            } else {
                false
            };
        let picker_size = style.color_picker_size();
        let mut offset = self.offset;
        let mut target_color = style.window_bg_col();
        set_vertex_params(&mut self.other_vertices, self.window_vertex_range, offset, target_color);
        target_color = style.window_outline_col();
        set_vertex_params(&mut self.other_vertices, self.window_outline_vertex_range, offset, target_color);
        let item_pad_outer = style.item_pad_outer();
        self.r_drag_value.set_vertex_params(style, &mut self.other_vertices);
        self.g_drag_value.set_vertex_params(style, &mut self.other_vertices);
        self.b_drag_value.set_vertex_params(style, &mut self.other_vertices);
        self.alpha_drag_value.set_vertex_params(style, &mut self.other_vertices);
        self.hue_drag_value.set_vertex_params(style, &mut self.other_vertices);
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

impl<I, FontHash, Style> HoverContents<I, FontHash, Style> for Contents<I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
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
            inv_aspect_ratio,
            unit_scale,
        );
        let outline_width = style.outline_width();
        let min_bounds = window_pos + self.offset;
        let pc_fragment = base_push_constants_fragment(
            min_bounds - vec2(outline_width, outline_width),
            min_bounds + self.window_rect.max + vec2(outline_width, outline_width),
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
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
        let picker_size = style.color_picker_size();
        let item_pad_outer = style.item_pad_outer();
        let offset = self.offset;
        let hsva = self.hsva;
        let pc_vertex = push_constants_vertex(
            window_pos + offset + item_pad_outer,
            picker_size,
            inv_aspect_ratio,
            unit_scale,
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
            inv_aspect_ratio,
            unit_scale,
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
            inv_aspect_ratio,
            unit_scale,
        );
        let pc_fragment = aplha_picker_push_constants_fragment(
            hsva, picker_size.x, style.alpha_tile_width()
        );
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
            inv_aspect_ratio,
            unit_scale,
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
        let mut f = |drag_value: &DragValue<EmptyText, I, FontHash, Style>| -> Result<(), Error> {
            drag_value.render_commands(
                render_commands, style, base_pipeline_id,
                text_pipeline_id, vertex_buffer, index_buffer, window_pos,
                inv_aspect_ratio, unit_scale, get_custom_pipeline,
            )?;
            Ok(())
        };
        f(&self.r_drag_value)?;
        f(&self.g_drag_value)?;
        f(&self.b_drag_value)?;
        f(&self.alpha_drag_value)?;
        f(&self.hue_drag_value)?;
        let font_scale = style.font_scale();
        let pc_vertex = push_constants_vertex(
            window_pos, vec2(font_scale, font_scale), inv_aspect_ratio, unit_scale
        );
        render_commands.bind_pipeline(text_pipeline_id)?;
        render_text(render_commands,
            self.combined_text
                .iter()
                .map(|(&c, (t, b))| (c, t, b.as_slice())),
            pc_vertex, vertex_buffer, index_buffer
        )?;
        Ok(())
    }
}

pub(crate) struct ColorPicker<I, FontHash, Style> {
    title: CompactString,
    title_text: Option<RenderedText>,
    color_rect: Rect,
    color_rect_vertex_range: VertexRange,
    contents: Contents<I, FontHash, Style>,
    offset: Vec2,
    _marker: PhantomData<(I, FontHash, Style)>,
}

impl<I, FontHash, Style> ColorPicker<I, FontHash, Style>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
{

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
        self.contents.hue_changed() ||
        self.contents.alpha_changed() ||
        self.contents.drag_value_active()
    }

    #[inline(always)]
    pub fn set_color(&mut self, color: impl Color) {
        self.contents.set_color(color);
    }

    #[inline(always)]
    pub fn calc_color(&mut self, style: &Style) -> ColorHSVA {
        self.contents.calc_color(style)
    }
}

impl<I, FontHash, Style> Widget<I, FontHash, Style> for ColorPicker<I, FontHash, Style>
    where 
        FontHash: Clone + Eq + Hash,
        I: Interface,
        Style: WindowStyle<FontHash>,
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
    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
    ) -> f32 {
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(&[text_segment(&self.title, &style.font_regular())], false, 0.0).unwrap_or_default()
        );
        style.calc_text_height(title_text)
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        style: &Style,
        window_pos: Vec2,
        cursor_pos: Vec2
    ) -> bool
    {
        let error_margin = style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        self.contents.widget_held() || self.contents.shown() && (self.picking() || BoundingRect::from_position_size(
            self.contents.offset - vec2(error_margin, error_margin),
            self.contents.window_rect.max + vec2(error_margin_2, error_margin_2)
        ).is_point_inside(cursor_pos - window_pos))
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult {
        let title_text = self.title_text.as_ref().unwrap();
        let offset = self.offset;
        let text_size = style.calc_text_size(title_text);
        let color_rect_max = vec2(text_size.y, text_size.y);
        let requires_triangulation = self.color_rect.max != color_rect_max;
        self.color_rect.max = color_rect_max;
        let color_rect_off_x = text_size.x + style.item_pad_outer().x;
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        let cursor_in_color_rect =
            BoundingRect::from_position_size(
                offset + vec2(color_rect_off_x, 0.0) - vec2(error_margin, error_margin),
                color_rect_max + vec2(error_margin_2, error_margin_2),
            ).is_point_inside(rel_cursor_pos);
        let cursor_in_contents = self.contents
            .bounding_rect(error_margin)
            .is_point_inside(rel_cursor_pos);
        if nox.was_mouse_button_released(MouseButton::Left) {
            if self.contents.widget_held() {
                self.contents.set_widget_held(false);
                self.contents.set_shown(cursor_in_color_rect);
            } else if self.contents.shown() {
                self.contents.set_shown(cursor_in_contents || window_moving || self.contents.clicked());
                self.contents.set_clicked(false);
            }
        }
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        if cursor_in_color_rect && mouse_pressed {
            self.contents.set_widget_held(true);
        }
        if other_widget_active {
            self.contents.set_shown(false);
        }
        let shown = self.contents.shown();
        if shown {
            self.contents.offset = self.offset + vec2(
                text_size.x + style.item_pad_outer().x,
                text_size.y + style.item_pad_inner().y,
            );
            if self.contents.update(
                nox,
                style,
                text_renderer,
                window_pos,
                cursor_pos,
                delta_cursor_pos,
                window_moving,
            ) {
                self.contents.triangulate();
            }
        }
        let (min_bounds, max_bounds) = calc_bounds(window_pos, offset, window_size);
        collect_text(title_text, self.offset, BoundedTextInstance {
            add_scale: vec2(1.0, 1.0),
            min_bounds,
            max_bounds,
            color: style.text_col(),
        });
        let item_pad_outer = style.item_pad_outer();
        UpdateResult {
            min_window_width: offset.x + text_size.x + item_pad_outer.x + color_rect_max.x + item_pad_outer.x,
            requires_triangulation,
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
        style: &Style,
        vertices: &mut [Vertex],
    ) {
        if self.contents.shown() {
            self.contents.set_vertex_params(style);
        }
        let title_text = self.title_text.as_ref().unwrap();
        let offset = self.offset + vec2(
            style.calc_text_width(title_text) + style.item_pad_outer().x,
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
        _render_commands: &mut RenderCommands,
        _style: &Style,
        _base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error>
    {
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
