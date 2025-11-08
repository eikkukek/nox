use nox::{mem::vec_types::{GlobalVec, Vector}, *};

use nox_geom::{
    shapes::*,
    *
};

use crate::*;

pub struct ScrollBarState {
    pub new_t: f32,
    pub requires_triangulation: bool,
}

pub struct VerScrollBar {
    t: f32,
    width_t: f32,
    width: f32,
    offset: Vec2,
    bar_rect_max: Vec2,
    handle_rect_max: Vec2,
    max_t: f32,
    rounding: f32,
    bar_vertex_range: Option<VertexRange>,
    handle_vertex_range: Option<VertexRange>,
    opacity: f32,
    flags: u32,
}

impl VerScrollBar {

    const HELD: u32 = 0x1;
    const HOVERING: u32 = 0x2;
    const HIDING: u32 = 0x4;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            t: 0.0,
            width_t: 0.0,
            width: 0.0,
            offset: Default::default(),
            bar_rect_max: Default::default(),
            handle_rect_max: Default::default(),
            rounding: 0.0,
            max_t: 0.0,
            bar_vertex_range: None,
            handle_vertex_range: None,
            opacity: 0.0,
            flags: 0,
        }
    }

    #[inline(always)]
    fn handle_off(
        &self,
        offset: Vec2
    ) -> Vec2 {
        // min_t = 0.0
        // max_t = view_pcent
        //
        // solve for t
        // cur_t = (1.0 - t) * min_t + t * max_t
        // cur_t = t * max_t | / max_t
        // t = cur_t / max_t
        let t = self.t / self.max_t;
        let mut pos = offset;
        pos.y += (self.bar_rect_max.y - self.handle_rect_max.y) * t;
        pos
    }

    #[inline(always)]
    fn calc_t(
        &self,
        mut cursor_position: Vec2,
        bar_pos: Vec2,
    ) -> f32
    {
        // t = cur_t / max_t | * max_t
        // cur_t = t * max_t
        //
        // pos = (bar_max - handle_max) * t
        // t = pos / (bar_max - handle_max)
        //
        // cur_t = pos / (bar_max - handle_max) * max_t
        cursor_position.y -= self.handle_rect_max.y * 0.5;
        let t = (cursor_position.y - bar_pos.y) / (self.bar_rect_max.y - self.handle_rect_max.y) * self.max_t;
        t.clamp(0.0, self.max_t)
    }

    #[inline(always)]
    pub fn deactivate(&mut self) {
        self.flags &= !Self::HELD;
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn hovering(&self) -> bool {
        self.flags & Self::HOVERING == Self::HOVERING
    }

    #[inline(always)]
    fn hiding(&self) -> bool {
        self.flags & Self::HIDING == Self::HIDING
    }

    #[inline(always)]
    pub fn calc_width(&mut self, style: &impl WindowStyle) -> f32 {
        self.width = lerp(style.scroll_bar_width(), style.scroll_bar_fat_width(), self.width_t);
        self.width
    }

    pub fn update<I: Interface>(
        &mut self,
        nox: &Nox<I>,
        style: &impl WindowStyle,
        current_t: f32,
        offset: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        bar_height: f32,
        content_height: f32,
        window_height: f32,
        widget_active: bool,
        hover_blocked: bool,
    ) -> ScrollBarState {
        let width = self.width;
        self.max_t = 1.0 - window_height / content_height;
        let handle_height = window_height / content_height * bar_height;
        let handle_rect_max = vec2(width, handle_height);
        let bar_rect_max = vec2(width, bar_height);
        let requires_triangulation =
            self.handle_rect_max != handle_rect_max ||
            self.bar_rect_max != bar_rect_max ||
            self.rounding != style.rounding();
        self.handle_rect_max = handle_rect_max;
        self.bar_rect_max = bar_rect_max;
        self.rounding = style.rounding();
        self.offset = offset;
        self.flags &= !Self::HOVERING;
        if self.held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HELD;
            } else {
                self.t = self.calc_t(cursor_pos, window_pos + offset);
            }
        } else {
            self.t = current_t;
            let error_margin = style.cursor_error_margin();
            let bounding_rect = BoundingRect::from_position_size(
                window_pos + offset - vec2(error_margin, 0.0), bar_rect_max + vec2(error_margin + error_margin, 0.0)
            );
            if !hover_blocked && !widget_active && bounding_rect.is_point_inside(cursor_pos) {
                self.flags |= Self::HOVERING;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                    self.t = self.calc_t(cursor_pos, window_pos + offset);
                }
            }
        }
        self.flags &= !Self::HIDING;
        or_flag!(self.flags, Self::HIDING, hover_blocked && !self.held());
        let anim_delta = nox.delta_time_secs_f32() * style.animation_speed();
        if self.hiding() {
            self.opacity = (self.opacity - anim_delta).clamp(0.0, 1.0);
        } else {
            self.opacity = (self.opacity + anim_delta).clamp(0.0, 1.0);
        }
        if self.held() || self.hovering() {
            self.width_t = (self.width_t + anim_delta).clamp(0.0, 1.0);
        } else {
            self.width_t = (self.width_t - anim_delta).clamp(0.0, 1.0);
        }
        ScrollBarState {
            new_t: self.t,
            requires_triangulation,
        }
    }

    pub fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        mut tri: impl FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {
        rect(
            Default::default(),
            self.bar_rect_max,
            self.rounding,
        ).to_points(&mut |p| { points.push(p.into()); });
        self.bar_vertex_range = tri(points);
        points.clear();
        rect(
            Default::default(),
            self.handle_rect_max,
            self.rounding,
        ).to_points(&mut |p| { points.push(p.into()); });
        self.handle_vertex_range = tri(points);
    }

    pub fn set_vertex_params(
        &self,
        style: &impl WindowStyle,
        vertices: &mut [Vertex],
    ) {
        let (mut bar_col, mut handle_col) =
            if self.held() {
                (style.scroll_bar_col().scale_alpha(self.opacity), style.scroll_bar_handle_col())
            } else if self.hovering() {
                (style.scroll_bar_col(), style.scroll_bar_handle_col().with_alpha(0.6))
            } else {
                (style.scroll_bar_col().with_alpha(0.7), style.scroll_bar_handle_col().with_alpha(0.2))
            };
        bar_col = bar_col.scale_alpha(self.opacity);
        handle_col = handle_col.scale_alpha(self.opacity);
        let mut offset = self.offset;
        set_vertex_params(vertices, self.bar_vertex_range, offset, bar_col);
        offset = self.handle_off(offset);
        set_vertex_params(vertices, self.handle_vertex_range, offset, handle_col);
    }

    pub fn hide(&self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.bar_vertex_range);
        hide_vertices(vertices, self.handle_vertex_range);
    }
}

pub struct HorScrollBar {
    t: f32,
    height_t: f32,
    height: f32,
    offset: Vec2,
    bar_rect_max: Vec2,
    handle_rect_max: Vec2,
    max_t: f32,
    rounding: f32,
    bar_vertex_range: Option<VertexRange>,
    handle_vertex_range: Option<VertexRange>,
    opacity: f32,
    flags: u32,
}

impl HorScrollBar {

    const HELD: u32 = 0x1;
    const HOVERING: u32 = 0x2;
    const HIDING: u32 = 0x4;

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            t: 0.0,
            height_t: 0.0,
            height: 0.0,
            offset: Default::default(),
            bar_rect_max: Default::default(),
            handle_rect_max: Default::default(),
            rounding: 0.0,
            max_t: 0.0,
            bar_vertex_range: Default::default(),
            handle_vertex_range: Default::default(),
            opacity: 0.0,
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn deactivate(&mut self) {
        self.flags &= !Self::HELD;
    }

    #[inline(always)]
    pub fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    pub fn hovering(&self) -> bool {
        self.flags & Self::HOVERING == Self::HOVERING
    }

    #[inline(always)]
    fn hiding(&self) -> bool {
        self.flags & Self::HIDING == Self::HIDING
    }

    #[inline(always)]
    pub fn calc_height(&mut self, style: &impl WindowStyle) -> f32 {
        self.height = lerp(style.scroll_bar_width(), style.scroll_bar_fat_width(), self.height_t);
        self.height
    }

    #[inline(always)]
    fn handle_off(
        &self,
        offset: Vec2
    ) -> Vec2 {
        // min_t = 0.0
        // max_t = view_pcent
        //
        // solve for t
        // cur_t = (1.0 - t) * min_t + t * max_t
        // cur_t = t * max_t | / max_t
        // t = cur_t / max_t
        let t = self.t / self.max_t;
        let mut pos = offset;
        pos.x += (self.bar_rect_max.x - self.handle_rect_max.x) * t;
        pos
    }

    #[inline(always)]
    fn calc_t(
        &self,
        mut cursor_position: Vec2,
        bar_pos: Vec2,
    ) -> f32
    {
        // t = cur_t / max_t | * max_t
        // cur_t = t * max_t
        //
        // pos = (bar_max - handle_max) * t
        // t = pos / (bar_max - handle_max)
        //
        // cur_t = pos / (bar_max - handle_max) * max_t
        cursor_position.x -= self.handle_rect_max.x * 0.5;
        let t = (cursor_position.x - bar_pos.x) / (self.bar_rect_max.x - self.handle_rect_max.x) * self.max_t;
        t.clamp(0.0, self.max_t)
    }

    pub fn update<I: Interface>(
        &mut self,
        nox: &Nox<I>,
        style: &impl WindowStyle,
        current_t: f32,
        offset: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        bar_width: f32,
        content_width: f32,
        window_width: f32,
        widget_active: bool,
        hover_blocked: bool,
    ) -> ScrollBarState {
        let height = self.height;
        self.max_t = 1.0 - window_width / content_width;
        let handle_width = window_width / content_width * bar_width;
        let handle_rect_max = vec2(handle_width, height);
        let bar_rect_max = vec2(bar_width, height);
        let requires_triangulation =
            self.handle_rect_max != handle_rect_max ||
            self.bar_rect_max != bar_rect_max ||
            self.rounding != style.rounding();
        self.handle_rect_max = handle_rect_max;
        self.bar_rect_max = bar_rect_max;
        self.rounding = style.rounding();
        self.offset = offset;
        self.flags &= !Self::HOVERING;
        if self.held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HELD;
            } else {
                self.t = self.calc_t(cursor_pos, window_pos + offset);
            }
        } else {
            self.t = current_t;
            let error_margin = style.cursor_error_margin();
            let bounding_rect = BoundingRect::from_position_size(
                window_pos + offset - vec2(0.0, error_margin), bar_rect_max + vec2(0.0, error_margin + error_margin)
            );
            if !hover_blocked && !widget_active && bounding_rect.is_point_inside(cursor_pos) {
                self.flags |= Self::HOVERING;
                if nox.was_mouse_button_pressed(MouseButton::Left) {
                    self.flags |= Self::HELD;
                    self.t = self.calc_t(cursor_pos, window_pos + offset);
                }
            }
        }
        self.flags &= !Self::HIDING;
        or_flag!(self.flags, Self::HIDING, hover_blocked && !self.held());
        let anim_delta = nox.delta_time_secs_f32() * style.animation_speed();
        if self.hiding() {
            self.opacity = (self.opacity - anim_delta).clamp(0.0, 1.0);
        } else {
            self.opacity = (self.opacity + anim_delta).clamp(0.0, 1.0);
        }
        if self.held() || self.hovering() {
            self.height_t = (self.height_t + anim_delta).clamp(0.0, 1.0);
        } else {
            self.height_t = (self.height_t - anim_delta).clamp(0.0, 1.0);
        }
        ScrollBarState {
            new_t: self.t,
            requires_triangulation,
        }
    }

    pub fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        mut tri: impl FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {
        rect(
            Default::default(),
            self.bar_rect_max,
            self.rounding,
        ).to_points(&mut |p| { points.push(p.into()); });
        self.bar_vertex_range = tri(points);
        points.clear();
        rect(
            Default::default(),
            self.handle_rect_max,
            self.rounding,
        ).to_points(&mut |p| { points.push(p.into()); });
        self.handle_vertex_range = tri(points);
    }

    pub fn set_vertex_params(
        &self,
        style: &impl WindowStyle,
        vertices: &mut [Vertex],
    ) {
        let (mut bar_col, mut handle_col) =
            if self.held() {
                (style.scroll_bar_col().scale_alpha(self.opacity), style.scroll_bar_handle_col())
            } else if self.hovering() {
                (style.scroll_bar_col(), style.scroll_bar_handle_col().with_alpha(0.6))
            } else {
                (style.scroll_bar_col().with_alpha(0.7), style.scroll_bar_handle_col().with_alpha(0.2))
            };
        bar_col = bar_col.scale_alpha(self.opacity);
        handle_col = handle_col.scale_alpha(self.opacity);
        let mut offset = self.offset;
        set_vertex_params(vertices, self.bar_vertex_range, offset, bar_col);
        offset = self.handle_off(offset);
        set_vertex_params(vertices, self.handle_vertex_range, offset, handle_col);
    }

    pub fn hide(&self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.bar_vertex_range);
        hide_vertices(vertices, self.handle_vertex_range);
    }
}
