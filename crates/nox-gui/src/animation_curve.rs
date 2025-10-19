use core::{
    marker::PhantomData,
    hash::Hash,
};

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use nox_geom::{
    shapes::*, *
};

use crate::*;

#[derive(Clone, Copy)]
struct CubicData {
    line_to_mid_0: VertexRange,
    line_to_mid_1: VertexRange,
}

impl CubicData {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            line_to_mid_0: Default::default(),
            line_to_mid_1: Default::default(),
        }
    }
}

pub struct AnimationCurve<TitleText, I, FontHash, Style, HoverStyle> {
    title: TitleText,
    offset: Vec2,
    widget_rect: Rect,
    content_offset: Vec2,
    content_window_rect: Rect,
    curve: Option<bezier::AnimationCurve>,
    cubic_datas: GlobalVec<CubicData>,
    vertices: GlobalVec<Vertex>,
    handle_vertices: GlobalVec<Vec2>,
    handle_vertex_offsets: GlobalVec<Vec2>,
    indices: GlobalVec<u32>,
    window_rect_draw_info: DrawInfo,
    handle_draw_info: DrawInfo,
    widget_rect_vertex_range: VertexRange,
    window_rect_vertex_range: VertexRange,
    held_index: u32,
    held_offset: Vec2,
    min_coords: Vec2,
    max_coords: Vec2,
    handle_radius: f32,
    flags: u32,
    _marker: PhantomData<(I, FontHash, Style, HoverStyle)>,
}

impl<TitleText, I, FontHash, Style, HoverStyle> AnimationCurve<TitleText, I, FontHash, Style, HoverStyle>
    where
        I: Interface,
        TitleText: Text,
        FontHash: Clone + Eq + Hash,
        HoverStyle: WindowStyle<FontHash>,
{

    const HOVER_CONTENTS_SHOWN: u32 = 0x1;
    const WIDGET_HELD: u32 = 0x2;
    const END_POINT_HELD: u32 = 0x4;
    const CONTROL_POINT_HELD: u32 = 0x8;
    const LEFT_POINT_HELD: u32 = 0x10;
    
    #[inline(always)]
    pub fn new(title: &str) -> Self {
        Self {
            title: TitleText::new(title),
            offset: Default::default(),
            widget_rect: Default::default(),
            content_offset: Default::default(),
            content_window_rect: Default::default(),
            curve: None,
            cubic_datas: Default::default(),
            vertices: Default::default(),
            handle_vertices: Default::default(),
            indices: Default::default(),
            handle_vertex_offsets: Default::default(),
            window_rect_draw_info: Default::default(),
            handle_draw_info: Default::default(),
            widget_rect_vertex_range: Default::default(),
            window_rect_vertex_range: Default::default(),
            held_index: 0,
            held_offset: Default::default(),
            min_coords: Default::default(),
            max_coords: vec2(1.0, 1.0),
            handle_radius: 0.0,
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_curve(&mut self, curve: &bezier::AnimationCurve) {
        if let Some(c) = &mut self.curve {
            c.clone_from_other(curve);
            self.cubic_datas.clear();
            self.cubic_datas.resize(c.len(), CubicData::new());
        } else {
            self.curve = Some(curve.clone());
        }
    }

    #[inline(always)]
    pub fn get_curve(&mut self, curve: &mut bezier::AnimationCurve) {
        curve.clone_from_other(self.curve.as_ref().unwrap());
    }

    #[inline(always)]
    fn contents_shown(&self) -> bool {
        self.flags & Self::HOVER_CONTENTS_SHOWN == Self::HOVER_CONTENTS_SHOWN
    }

    #[inline(always)]
    fn end_point_held(&self) -> bool {
        self.flags & Self::END_POINT_HELD == Self::END_POINT_HELD
    }

    #[inline(always)]
    fn control_point_held(&self) -> bool {
        self.flags & Self::CONTROL_POINT_HELD == Self::CONTROL_POINT_HELD
    }

    #[inline(always)]
    fn left_point_held(&self) -> bool {
        self.flags & Self::LEFT_POINT_HELD == Self::LEFT_POINT_HELD
    }

    #[inline(always)]
    fn contents_held(&self) -> bool {
        self.end_point_held() || self.control_point_held()
    }

    #[inline(always)]
    fn widget_held(&self) -> bool {
        self.flags & Self::WIDGET_HELD == Self::WIDGET_HELD
    }

    #[inline(always)]
    fn update_contents(
        &mut self,
        nox: &Nox<I>,
        style: &HoverStyle,
        _text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        _window_moving: bool,
    ) -> (bool, bool)
    {
        let curve = self.curve.as_ref().unwrap();
        let curve_min = curve.min_coords();
        let curve_max = curve.max_coords();
        let curve_range = curve_max - curve_min;
        let gui_curve_size = style.animation_curve_size();
        let gui_min = self.min_coords;
        let gui_max = self.max_coords;
        let gui_range = gui_max - gui_min;
        let bounding_rect = BoundingRect::from_position_size(
            self.content_offset,
            gui_curve_size,
        );
        let scale = vec2(
            curve_range.x / gui_range.x * gui_curve_size.x,
            curve_range.y / gui_range.y * gui_curve_size.y,
        );
        let rel_cursor_pos = cursor_pos - window_pos;
        let handle_radius = style.default_handle_radius();
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        let mut held = self.contents_held();
        let mut held_offset = self.held_offset;
        let mut held_index = self.held_index as usize;
        let pos_offset = gui_min;
        self.handle_vertex_offsets.clear();
        for (i, &cubic) in curve.iter().enumerate() {
            let mut point_offset = vec2(cubic.start.x * scale.x, cubic.start.y * scale.y) - pos_offset;
            if bounding_rect.is_point_inside(point_offset) {
                self.handle_vertex_offsets.push(point_offset);
                if !held &&
                    mouse_pressed &&
                    (point_offset - rel_cursor_pos).mag() < handle_radius
                {
                    self.flags |= Self::END_POINT_HELD;
                    self.flags |= Self::LEFT_POINT_HELD;
                    held_index = i;
                    held = true;
                    held_offset = point_offset;
                }
            }
            point_offset = vec2(cubic.mid_0.x * scale.x, cubic.mid_0.y * scale.y) - pos_offset;
            if bounding_rect.is_point_inside(point_offset) {
                self.handle_vertex_offsets.push(point_offset);
                if !held &&
                    mouse_pressed &&
                    (point_offset - rel_cursor_pos).mag() < handle_radius
                {
                    self.flags |= Self::CONTROL_POINT_HELD;
                    self.flags |= Self::LEFT_POINT_HELD;
                    held_index = i;
                    held = true;
                    held_offset = point_offset;
                }
            }
            point_offset = vec2(cubic.mid_1.x * scale.x, cubic.mid_1.y * scale.y) - pos_offset;
            if bounding_rect.is_point_inside(point_offset) {
                self.handle_vertex_offsets.push(point_offset);
                if !held &&
                    mouse_pressed &&
                    (point_offset - rel_cursor_pos).mag() < handle_radius
                {
                    self.flags |= Self::CONTROL_POINT_HELD;
                    held_index = i;
                    held = true;
                    held_offset = point_offset;
                }
            }
            point_offset = vec2(cubic.end.x * scale.x, cubic.end.y * scale.y) - pos_offset;
            if bounding_rect.is_point_inside(point_offset) {
                self.handle_vertex_offsets.push(point_offset);
                if !held &&
                    mouse_pressed &&
                    (point_offset - rel_cursor_pos).mag() < handle_radius
                {
                    self.flags |= Self::END_POINT_HELD;
                    held_index = i;
                    held = true;
                    held_offset = point_offset;
                }
            }
        }
        self.handle_draw_info.instance_count = self.handle_vertex_offsets.len() as u32;
        self.held_index = held_index as u32;
        self.held_offset = held_offset;
        if held {
            // off = point * scale - pos_offset
            // off + pos_offset = point * scale
            // point = (off + poss_offset) / scale
            let mut pos = held_offset + pos_offset;
            pos.x /= scale.x;
            pos.y /= scale.y;
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.flags &= !(Self::END_POINT_HELD | Self::CONTROL_POINT_HELD | Self::LEFT_POINT_HELD);
            }
            else if self.end_point_held() {
                if self.left_point_held() {
                    unsafe {
                        self.curve
                            .as_mut()
                            .unwrap_unchecked()
                            .set_start(held_index, pos)
                    };
                } else {
                    unsafe {
                        self.curve
                            .as_mut()
                            .unwrap_unchecked()
                            .set_end(held_index, pos)
                    };
                }
            } else {
                if self.left_point_held() {
                    unsafe {
                        self.curve
                            .as_mut()
                            .unwrap_unchecked()
                            .set_mid_0(held_index, pos)
                    };
                } else {
                    unsafe {
                        self.curve
                            .as_mut()
                            .unwrap_unchecked()
                            .set_mid_1(held_index, pos)
                    };
                }
            }
        }
        let rounding = style.rounding();
        let requires_triangulation =
            self.handle_radius != handle_radius ||
            self.content_window_rect.max != gui_curve_size ||
            self.content_window_rect.rounding != rounding;
        self.handle_radius = handle_radius;
        self.content_window_rect = rect(Default::default(), gui_curve_size, rounding);
        (requires_triangulation, bounding_rect.is_point_inside(rel_cursor_pos))
    }

    #[inline(always)]
    fn triangulate_contents(&mut self) {
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        self.vertices.clear();
        self.content_window_rect.to_points(&mut |p| { points.push(p.into()); });
        let vertex_offset = 0;
        earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap();
        self.window_rect_vertex_range = VertexRange::new(vertex_offset..self.vertices.len());
        self.window_rect_draw_info = DrawInfo {
            index_count: indices_usize.len() as u32,
            ..Default::default()
        };
        points.clear();
        let index_offset = indices_usize.len() as u32;
        circle(vec2(0.0, 0.0), self.handle_radius).to_points(16, &mut |p| { points.push(p.into()); });
        earcut::earcut(&points, &[], false, &mut self.handle_vertices, &mut indices_usize).unwrap();
        self.handle_draw_info.first_index = index_offset;
        self.handle_draw_info.index_count = indices_usize.len() as u32 - index_offset;
        self.indices.append_map(&indices_usize, |&i| i as u32);
    }

    #[inline(always)]
    fn set_contents_vertex_params(&mut self, style: &HoverStyle) {
        let offset = self.content_offset;
        let target_color = style.window_bg_col();
        set_vertex_params(
            &mut self.vertices, self.window_rect_vertex_range, offset, target_color
        );
    }
}

impl<TitleText, I, FontHash, Style, HoverStyle> Widget<I, FontHash, Style, HoverStyle> for 
        AnimationCurve<TitleText, I, FontHash, Style, HoverStyle> 
    where 
        TitleText: Text,
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
        HoverStyle: WindowStyle<FontHash>,
{

    fn hover_text(&self) -> Option<&str> {
        None
    }

    fn set_offset(
        &mut self,
        offset: Vec2,
    )
    {
        self.offset = offset;
    }

    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
    ) -> f32
    {
        text_renderer.font_height(style.font_regular()).unwrap()
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        _style: &Style,
        hover_style: &HoverStyle,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> bool
    {
        let error_margin = hover_style.cursor_error_margin();
        let error_margin_2 = error_margin + error_margin;
        self.contents_held() ||
        self.widget_held() ||
        (
            self.contents_shown() &&
            BoundingRect::from_position_size(
                self.content_offset - vec2(error_margin, error_margin),
                self.content_window_rect.max + vec2(error_margin_2, error_margin_2)
            ).is_point_inside(cursor_pos - window_pos)
        )
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style,
        hover_style: &HoverStyle,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        _window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
    ) -> UpdateResult
    {
        if other_widget_active {
            self.flags &= !Self::HOVER_CONTENTS_SHOWN;
        }
        let mut cursor_in_widget = false;
        if self.contents_shown() {
            let (requires_triangulation, cursor_in_contents) = self.update_contents(
                    nox, hover_style, text_renderer, window_pos,
                    cursor_pos, delta_cursor_pos, window_moving
                );
            cursor_in_widget |= cursor_in_contents;
            if requires_triangulation {
                self.triangulate_contents();
            }
        }
        let widget_rect_width = style.animation_curve_size().x;
        let mut width = widget_rect_width;
        let title_width = self.title.get_text_width();
        if title_width != 0.0 {
            width += title_width * style.font_scale() + style.item_pad_outer().x;
        }
        let widget_rect = rect(
            Default::default(), vec2(widget_rect_width, style.calc_font_height(text_renderer)),
            style.rounding(),
        );
        let requires_triangulation = self.widget_rect != widget_rect;
        self.widget_rect = widget_rect;
        UpdateResult {
            min_widget_width: width,
            requires_triangulation,
            cursor_in_widget: cursor_in_widget,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    )
    {
        self.widget_rect.to_points(&mut |p| { points.push(p.into()); });
        self.widget_rect_vertex_range = tri(&points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        hover_style: &HoverStyle,
        vertices: &mut [Vertex],
    ) {
        if self.contents_shown() {
            self.set_contents_vertex_params(hover_style);
        }
        let title_width = self.title.get_text_width();
        let offset =
            if title_width != 0.0 {
                self.offset + vec2(title_width * style.font_scale() + style.item_pad_outer().x, 0.0)
            } else {
                self.offset
            };
        let target_color = hover_style.widget_bg_col();
        set_vertex_params(
            vertices, self.widget_rect_vertex_range,
            offset, target_color,
        );
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        _base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>
    {
        render_commands.bind_pipeline(text_pipeline_id)?;
        let font_scale = vec2(style.font_scale(), style.font_scale());
        self.title.render(
            render_commands, 
            window_pos + self.offset,
            style.text_col(),
            font_scale, inv_aspect_ratio, unit_scale,
            vertex_buffer, index_buffer,
        )?;
        if self.contents_shown() {
            return Ok(Some(self))
        }
        Ok(None)
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.widget_rect_vertex_range);
    }
}

impl<TitleText, I, FontHash, Style, HoverStyle> HoverContents<I, FontHash, HoverStyle> for
        AnimationCurve<TitleText, I, FontHash, Style, HoverStyle>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
        HoverStyle: WindowStyle<FontHash>
{

    fn render_commands(
        &self,
        _render_commands: &mut RenderCommands,
        _style: &HoverStyle,
        _base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        _vertex_buffer: &mut RingBuf,
        _index_buffer: &mut RingBuf,
        _window_pos: Vec2,
        _inv_aspect_ratio: f32,
        _unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
