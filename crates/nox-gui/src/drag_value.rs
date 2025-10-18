use core::{
    marker::PhantomData,
    hash::Hash,
    fmt::Write,
};

use nox::{mem::vec_types::Vector, *};

use nox_geom::*;

use crate::*;

pub struct DragValue<I, FontHash, Style, HoverStyle> {
    input_text: InputText<I, FontHash, Style, HoverStyle>,
    delta_cursor_x: f32,
    flags: u32,
    focused_outline_vertex_range: VertexRange,
    focused_outline_width: f32,
    _marker: PhantomData<(FontHash, Style, HoverStyle)>,
}

impl<I, FontHash, Style, HoverStyle> DragValue<I, FontHash, Style, HoverStyle>
    where 
        Style: WindowStyle<FontHash>,
{

    const HOVERED: u32 = 0x1;
    const HELD: u32 = 0x2;
    const HELD_MOVED: u32 = 0x4;

    #[inline(always)]
    pub fn new(title: &str) -> Self {
        Self {
            input_text: InputText::new(title),
            delta_cursor_x: 0.0,
            flags: 0,
            focused_outline_vertex_range: Default::default(),
            focused_outline_width: 0.0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_input_params(
        &mut self,
        style: &Style,
        min_width: f32,
        skip_title: bool,
        format_input: Option<fn(&mut dyn Write, &str) -> core::fmt::Result>
    )
    {
        self.input_text.set_params(
            Some(min_width), Some(style.widget_bg_col()),
            skip_title, true, "", format_input
        );
    }

    #[inline(always)]
    pub fn calc_value<T>(&mut self, style: &Style, value: &mut T, min: T, max: T, drag_speed: f32, )
        where
            T: Sliderable
    {
        if !self.input_text.active() {
            value.drag(min, max, self.delta_cursor_x * drag_speed);
            self.input_text.set_input_sliderable(style, value);
        } else {
            if let Some(v) = self.input_text.get_input() {
                *value = v;
            }
        }
    }

    #[inline(always)]
    pub fn set_value<T>(&mut self, style: &Style, value: &T)
        where
            T: Sliderable
    {
        self.input_text.set_input_sliderable(style, value);
    }

    #[inline(always)]
    pub fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn held_moved(&self) -> bool {
        self.flags & Self::HELD_MOVED == Self::HELD_MOVED
    }
}

impl<I, FontHash, Style, HoverStyle> Widget<I, FontHash, Style, HoverStyle> for 
        DragValue<I, FontHash, Style, HoverStyle>
    where
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
        self.input_text.set_offset(offset);
    }

    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
    ) -> f32
    {
        style.calc_font_height(text_renderer)
    }

    fn is_active(
        &self,
        nox: &Nox<I>,
        style: &Style,
        hover_style: &HoverStyle,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> bool
    {
        self.held() ||
        self.input_text.is_active(nox, style, hover_style, window_pos, cursor_pos)
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style,
        hover_style: &HoverStyle,
        text_renderer: &mut nox_font::VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
    ) -> UpdateResult
    {
        self.input_text.set_cursor_enable(self.input_text.active());
        self.delta_cursor_x = Default::default();
        self.flags &= !Self::HOVERED;
        let rel_cursor_pos = cursor_pos - window_pos;
        let cursor_in_rect = self.input_text
            .rel_bounding_rect(style)
            .is_point_inside(rel_cursor_pos);
        if cursor_in_rect && !self.input_text.active() {
            self.flags |= Self::HOVERED;
            if self.held() {
                if nox.was_mouse_button_released(MouseButton::Left) {
                    if !self.held_moved() {
                        self.input_text.activate_and_select_all();
                    }
                    self.flags &= !Self::HELD;
                }
            } else {
                self.flags &= !Self::HELD;
                self.flags |=
                    Self::HELD * nox.was_mouse_button_pressed(MouseButton::Left) as u32;
            }
        }
        if self.held() {
            self.delta_cursor_x = delta_cursor_pos.x;
            if delta_cursor_pos.x.abs() > f32::EPSILON {
                self.flags |= Self::HELD_MOVED;
            }
            if nox.was_mouse_button_released(MouseButton::Left) {
                self.flags &= !Self::HELD;
            }
        } else {
            self.flags &= !Self::HELD_MOVED;
        }
        if style.override_cursor() && (cursor_in_rect || self.held())  {
            if style.override_cursor() {
                nox.set_cursor(CursorIcon::ColResize);
            }
        }
        let mut update_results = self.input_text.update(
            nox, style, hover_style,
            text_renderer, window_width, window_pos,
            cursor_pos, delta_cursor_pos, cursor_in_this_window,
            other_widget_active, window_moving
        );
        update_results.cursor_in_widget |= cursor_in_rect || self.held();
        update_results.requires_triangulation |=
            self.focused_outline_width != style.focused_outline_width();
        self.focused_outline_width = style.focused_outline_width();
        update_results
    }

    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    ) {
        self.input_text.outline_points(self.focused_outline_width, points);
        self.focused_outline_vertex_range = tri(&points);
        points.clear();
        self.input_text.triangulate(points, tri);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        hover_style: &HoverStyle,
        vertices: &mut [Vertex],
    ) {
        if !self.input_text.active() && (self.held() || self.hovered()) {
            let offset = self.input_text.input_offset(style);
            let target_color = if self.held() {
                style.active_widget_outline_col()
            } else {
                style.focused_widget_outline_col()
            };
            set_vertex_params(vertices, self.focused_outline_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.focused_outline_vertex_range);
        }
        self.input_text.set_vertex_params(style, hover_style, vertices);
    }

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
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>
    {
        self.input_text.render_commands(
            render_commands, style, base_pipeline_id,
            text_pipeline_id, vertex_buffer, index_buffer,
            window_pos, inv_aspect_ratio, get_custom_pipeline
        )
    }

    fn hide(
        &self,
        vertices: &mut [Vertex],
    ) {
        hide_vertices(vertices, self.focused_outline_vertex_range);
        self.input_text.hide(vertices);
    }
}
