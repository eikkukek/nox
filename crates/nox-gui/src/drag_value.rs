use core::{
    marker::PhantomData,
    fmt::Write,
};

use nox::{alloc::arena_alloc::ArenaGuard, mem::vec_types::Vector, *};

use nox_font::RenderedText;
use nox_geom::*;

use crate::*;

pub struct DragValue<Style> {
    input_text: InputText<Style>,
    delta_cursor_x: f32,
    amount: f32,
    flags: u32,
    focused_stroke_thickness: f32,
    active_stroke_thickness: f32,
    focused_stroke_vertex_range: Option<VertexRange>,
    active_stroke_vertex_range: Option<VertexRange>,
    _marker: PhantomData<Style>,
}

impl<Style> DragValue<Style>
    where
        Style: WindowStyle,
{

    const HOVERED: u32 = 0x1;
    const HELD: u32 = 0x2;
    const HELD_MOVED: u32 = 0x4;
    const TRANSPARENT_INACTIVE_BG: u32 = 0x8;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            input_text: InputText::new(),
            delta_cursor_x: 0.0,
            amount: 0.0,
            flags: 0,
            focused_stroke_thickness: 0.0,
            active_stroke_thickness: 0.0,
            focused_stroke_vertex_range: None,
            active_stroke_vertex_range: None,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_input_params(
        &mut self,
        style: &Style,
        width: f32,
        format_input: Option<fn(&mut dyn Write, &str) -> core::fmt::Result>,
        transparent_inactive_bg: bool,
    )
    {
        self.flags &= !Self::TRANSPARENT_INACTIVE_BG;
        self.input_text.set_params(
            width, Some(
                if transparent_inactive_bg && (!self.held() && !self.hovered() && !self.input_text.active()) {
                    self.flags |= Self::TRANSPARENT_INACTIVE_BG;
                    ColorSRGBA::black(0.0)
                } else {
                    style.widget_bg_col()
                }
            ),
            true, "", format_input, self.held()
        );
    }

    #[inline(always)]
    pub fn calc_value<T>(
        &mut self,
        style: &Style,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: f32,
    )
        where
            T: Sliderable
    {
        if !self.input_text.active() {
            let tmp = *value;
            let amount = self.delta_cursor_x * drag_speed;
            value.drag(min, max, amount + self.amount);
            if value == &tmp && self.held() {
                self.amount += amount;
            } else {
                self.amount = 0.0;
            }
            self.input_text.set_input_sliderable(style, value);
        } else {
            self.amount = 0.0;
            if let Some(v) = self.input_text.get_input() {
                *value = v;
            }
        }
    }

    #[inline(always)]
    pub fn calc_and_map_value<T, U>(
        &mut self,
        style: &Style,
        value: &mut T,
        min: T,
        max: T,
        drag_speed: f32,
        mut map_to: impl FnMut(&T) -> U,
        mut map_from: impl FnMut(U) -> T,
    )
        where
            T: Sliderable,
            U: Sliderable,
    {
        if !self.input_text.active() {
            let tmp = *value;
            let amount = self.delta_cursor_x * drag_speed;
            value.drag(min, max, self.delta_cursor_x * drag_speed);
            if value == &tmp {
                self.amount += amount;
            } else {
                self.amount = 0.0;
            }
            let mapped = map_to(value);
            self.input_text.set_input_sliderable(style, &mapped);
        } else {
            if let Some(v) = self.input_text.get_input() {
                *value = map_from(v);
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
    pub fn hide(&mut self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.focused_stroke_vertex_range);
        hide_vertices(vertices, self.active_stroke_vertex_range);
        self.input_text.hide(vertices);
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

impl<Style> Widget<Style> for DragValue<Style>
    where
        Style: WindowStyle,
{

    #[inline(always)]
    fn get_offset(&self) -> Vec2 {
        self.input_text.get_offset()
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2)
    {
        self.input_text.set_offset(offset);
    }

    #[inline(always)]
    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.input_text.set_scroll_offset(offset);
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut TextRenderer,
    ) -> Vec2
    {
        self.input_text.calc_size(style, text_renderer)
    }

    #[inline(always)]
    fn status<'a>(
        &'a self,
        ctx: &WindowCtx,
        style: &Style,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> WidgetStatus<'a>
    {
        if self.held() ||
            matches!(self.input_text.status(ctx, style, window_pos, cursor_pos), WidgetStatus::Active)
        {
            WidgetStatus::Active
        } else if self.hovered() {
            WidgetStatus::Hovered(None)
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        ctx: &mut WindowCtx,
        style: &Style,
        text_renderer: &mut TextRenderer,
        window_size: Vec2,
        window_pos: Vec2,
        content_offset: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        cursor_in_other_widget: bool,
        window_moving: bool,
        hover_blocked: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        self.input_text.set_cursor_enable(self.input_text.active());
        self.delta_cursor_x = Default::default();
        self.flags &= !Self::HOVERED;
        let rel_cursor_pos = cursor_pos - window_pos;
        let cursor_in_rect = self.input_text
            .rel_bounding_rect(style)
            .is_point_inside(rel_cursor_pos);
        let mouse_left_state = ctx.mouse_button_state(MouseButton::Left);
        if !other_widget_active && cursor_in_rect && !self.input_text.active() {
            self.flags |= Self::HOVERED;
            self.input_text.set_hovered(true);
            if self.held() {
                if mouse_left_state.released() {
                    if !self.held_moved() {
                        self.input_text.activate_and_select_all();
                    }
                    self.flags &= !Self::HELD;
                }
            } else {
                self.flags &= !Self::HELD;
                or_flag!(self.flags, Self::HELD, mouse_left_state.pressed());
            }
        }
        if self.held() {
            self.delta_cursor_x = delta_cursor_pos.x;
            self.input_text.set_hovered(true);
            if delta_cursor_pos.x.abs() > f32::EPSILON {
                self.flags |= Self::HELD_MOVED;
            }
            if mouse_left_state.released() {
                self.flags &= !Self::HELD;
            }
        } else {
            self.flags &= !Self::HELD_MOVED;
        }
        if style.override_cursor() && !other_widget_active && (cursor_in_rect || self.held())  {
            if style.override_cursor() {
                ctx.set_cursor(CursorIcon::ColResize);
            }
        }
        let mut update_results = self.input_text.update(
            ctx, style,
            text_renderer, window_size, window_pos, content_offset,
            cursor_pos, delta_cursor_pos, cursor_in_this_window,
            other_widget_active, cursor_in_other_widget, window_moving,
            hover_blocked, collect_text,
        );
        update_results.cursor_in_widget |= cursor_in_rect || self.held();
        update_results.requires_triangulation |=
            self.focused_stroke_thickness != style.focused_widget_stroke_thickness() ||
            self.active_stroke_thickness != style.active_widget_stroke_thickness();
        self.focused_stroke_thickness = style.focused_widget_stroke_thickness();
        self.active_stroke_thickness = style.active_widget_stroke_thickness();
        update_results
    }
    
    #[inline(always)]
    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        helper_points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {
        self.input_text.outline_points(self.focused_stroke_thickness, points);
        self.focused_stroke_vertex_range = tri(&points);
        points.clear();
        self.input_text.outline_points(self.active_stroke_thickness, points);
        self.active_stroke_vertex_range = tri(&points);
        points.clear();
        self.input_text.triangulate(points, helper_points, tri);
    }

    #[inline(always)]
    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [Vertex],
    ) {
        if !self.input_text.active() && (self.held() || self.hovered()) {
            let offset = self.input_text.offset();
            if self.held() {
                let target_color = style.active_widget_stroke_col();
                set_vertex_params(vertices, self.active_stroke_vertex_range, offset, target_color);
                hide_vertices(vertices, self.focused_stroke_vertex_range);
            } else {
                let target_color = style.focused_widget_stroke_col();
                set_vertex_params(vertices, self.focused_stroke_vertex_range, offset, target_color);
                hide_vertices(vertices, self.active_stroke_vertex_range);
            }
        } else {
            hide_vertices(vertices, self.focused_stroke_vertex_range);
            hide_vertices(vertices, self.active_stroke_vertex_range);
        }
        self.input_text.set_vertex_params(style, vertices);
    }

    #[inline(always)]
    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        sampler: SamplerId,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &ArenaGuard,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<Style>>, Error>
    {
        self.input_text.render_commands(
            render_commands, style, sampler, base_pipeline,
            text_pipeline, texture_pipeline, texture_pipeline_layout, vertex_buffer, index_buffer,
            window_pos, content_area, inv_aspect_ratio, unit_scale, tmp_alloc, get_custom_pipeline,
        )
    }

    #[inline(always)]
    fn hide(
        &mut self,
        vertices: &mut [Vertex],
        _window_semaphore: (TimelineSemaphoreId, u64),
        _global_resources: &mut GlobalResources,
        _tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> {
        self.hide(vertices);
        Ok(())
    }
}
