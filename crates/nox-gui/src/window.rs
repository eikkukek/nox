use core::{
    hash::Hash,
};

use nox::{
    mem::vec_types::{GlobalVec, Vector},
    *,
};

use rustc_hash::FxHashMap;

use compact_str::CompactString;

use nox_font::{VertexTextRenderer, text_segment, RenderedText};

use nox_geom::{
    shapes::*, *
};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveWidget {
    Slider(u32),
    Button(u32),
    Checkbox(u32),
    ColorPicker(u32),
}

struct HoverWindow {
    text: CompactString,
    rendered_text: RenderedText,
    rect: Rect,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    position: Vec2,
}

impl HoverWindow {

    fn new() -> Self {
        Self {
            text: Default::default(),
            rendered_text: Default::default(),
            rect: Default::default(),
            vertices: Default::default(),
            indices: Default::default(),
            position: Default::default(),
        }
    }

    fn update<FontHash>(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        cursor_pos: Vec2,
        text: &str,
    ) -> bool
        where
            FontHash: Clone + Eq + Hash,
    {
        if text != self.text {
            self.rendered_text = text_renderer.render(
                &[text_segment(text, &style.font_regular)], false, 0.0
            ).unwrap_or_default();
        }
        let rect = rect(
            Default::default(),
            style.calc_text_box_size(vec2(self.rendered_text.text_width, self.rendered_text.row_height)),
            style.rounding,
        );
        if rect != self.rect {
            self.rect = rect;
            return self.triangulate();
        }
        self.position = cursor_pos - vec2(self.rect.max.x, 0.0);
        return false
    }

    fn triangulate(&mut self) -> bool {
        self.vertices.clear();
        self.indices.clear();
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
            return false
        }
        self.indices.append_map(&indices_usize, |&v| v as u32);
        true
    }

    fn set_vertex_params<FontHash>(
        &mut self,
        style: &Style<FontHash>,
    ) {
        let vertex_sample = self.vertices[0];
        if vertex_sample.color != style.hover_window_bg_col {
            let target_color = style.hover_window_bg_col;
            for vertex in &mut self.vertices {
                vertex.color = target_color;
            }
        }
    }

    fn render_commands<FontHash>(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
    ) -> Result<(), Error>
    {
        let vert_count = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_count)?
        };
        let idx_count = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_count)?
        };
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(self.position, vec2(1.0, 1.0), inv_aspect_ratio);
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: self.indices.len() as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo::new(index_buffer.id(), idx_mem.offset)
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position + style.item_pad_inner,
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio
        );
        let pc_fragment = text_push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_text(&self.rendered_text, render_commands, vertex_buffer, index_buffer)?;
        Ok(())
    }
}

pub(crate) struct Window<I, FontHash>
{
    main_rect: Rect,
    title_bar_rect: Rect,
    separator_rect: Rect,
    main_rect_vertex_range: VertexRange,
    title_bar_vertex_range: VertexRange,
    separator_vertex_range: VertexRange,
    outline_vertex_range: VertexRange,
    outline_thin_vertex_range: VertexRange,
    main_draw_info: DrawInfo,
    position: Vec2,
    title: CompactString,
    title_text: Option<RenderedText>,
    vertices: Option<GlobalVec<Vertex>>,
    indices: GlobalVec<u32>,
    buttons: FxHashMap<u32, (u64, Button<I, FontHash>)>,
    sliders: FxHashMap<u32, (u64, Slider<I, FontHash>)>,
    checkboxs: FxHashMap<u32, (u64, Checkbox<I, FontHash>)>,
    color_pickers: FxHashMap<u32, (u64, ColorPicker<I, FontHash>)>,
    active_widgets: Option<GlobalVec<ActiveWidget>>,
    prev_active_widgets: Option<GlobalVec<ActiveWidget>>,
    hover_window: Option<HoverWindow>,
    last_triangulation: u64,
    last_frame: u64,
    min_height: f32,
    outline_width: f32,
    outline_thin_width: f32,
    flags: u32,
}

impl<I, FontHash> Window<I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    const RENDERABLE: u32 = 0x1;
    const REQUIRES_TRIANGULATION: u32 = 0x2;
    const CURSOR_IN_WINDOW: u32 = 0x4;
    const HELD: u32 = 0x8;
    const RESIZE_LEFT: u32 = 0x10;
    const RESIZE_RIGHT: u32 = 0x20;
    const RESIZE_TOP: u32 = 0x40;
    const RESIZE_BOTTOM: u32 = 0x80;
    const RESIZE_BLOCKED_COL: u32 = 0x100;
    const RESIZE_BLOCKED_ROW: u32 = 0x200;
    const HOVER_WINDOW_ACTIVE: u32 = 0x400;

    const CURSOR_ERROR_MARGIN: f32 = 0.01;

    pub(crate) fn new(
        title: &str,
        position: [f32; 2],
        size: [f32; 2],
        rounding: f32,
    ) -> Self
    {
        Self {
            main_rect: rect(Default::default(), size, 0.0),
            title_bar_rect: rect::<Vec2>(Default::default(), Default::default(), rounding),
            separator_rect: Default::default(),
            main_rect_vertex_range: Default::default(),
            title_bar_vertex_range: Default::default(),
            separator_vertex_range: Default::default(),
            outline_vertex_range: Default::default(),
            outline_thin_vertex_range: Default::default(),
            main_draw_info: Default::default(),
            position: position.into(),
            title: title.into(),
            title_text: None,
            vertices: Some(Default::default()),
            indices: Default::default(),
            buttons: FxHashMap::default(),
            sliders: FxHashMap::default(),
            checkboxs: FxHashMap::default(),
            color_pickers: FxHashMap::default(),
            active_widgets: Some(Default::default()),
            prev_active_widgets: Some(Default::default()),
            hover_window: Some(HoverWindow::new()),
            last_triangulation: 0,
            last_frame: 0,
            min_height: 0.0,
            outline_width: 0.0,
            outline_thin_width: 0.0,
            flags: Self::REQUIRES_TRIANGULATION,
        }
    }

    #[inline(always)]
    fn renderable(&self) -> bool {
        self.flags & Self::RENDERABLE == Self::RENDERABLE
    }

    #[inline(always)]
    fn requires_triangulation(&self) -> bool {
        self.flags & Self::REQUIRES_TRIANGULATION == Self::REQUIRES_TRIANGULATION
    }

    #[inline(always)]
    fn cursor_in_window(&self) -> bool {
        self.flags & Self::CURSOR_IN_WINDOW == Self::CURSOR_IN_WINDOW
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn resize_left(&self) -> bool {
        self.flags & Self::RESIZE_LEFT == Self::RESIZE_LEFT
    }

    #[inline(always)]
    fn resize_right(&self) -> bool {
        self.flags & Self::RESIZE_RIGHT == Self::RESIZE_RIGHT
    }

    #[inline(always)]
    fn resize_top(&self) -> bool {
        self.flags & Self::RESIZE_TOP == Self::RESIZE_TOP
    }

    #[inline(always)]
    fn resize_bottom(&self) -> bool {
        self.flags & Self::RESIZE_BOTTOM == Self::RESIZE_BOTTOM
    }

    #[inline(always)]
    fn resize_nw(&self) -> bool {
        self.resize_top() && self.resize_left()
    }

    #[inline(always)]
    fn resize_ne(&self) -> bool {
        self.resize_top() && self.resize_right()
    }

    #[inline(always)]
    fn resize_sw(&self) -> bool {
        self.resize_bottom() && self.resize_left()
    }

    #[inline(always)]
    fn resize_se(&self) -> bool {
        self.resize_bottom() && self.resize_right()
    }

    #[inline(always)]
    fn resize_blocked_col(&self) -> bool {
        self.flags & Self::RESIZE_BLOCKED_COL == Self::RESIZE_BLOCKED_COL
    }

    #[inline(always)]
    fn resize_blocked_row(&self) -> bool {
        self.flags & Self::RESIZE_BLOCKED_ROW == Self::RESIZE_BLOCKED_ROW
    }

    #[inline(always)]
    fn any_resize(&self) -> bool {
        self.resize_left() ||
        self.resize_right() ||
        self.resize_top() ||
        self.resize_bottom()
    }

    #[inline(always)]
    fn hover_window_active(&self) -> bool {
        self.flags & Self::HOVER_WINDOW_ACTIVE == Self::HOVER_WINDOW_ACTIVE
    }

    #[inline(always)]
    fn get_widget(&self, widget: ActiveWidget) -> (u64, &dyn Widget<I, FontHash>) {
        match widget {
            ActiveWidget::Slider(id) => self.sliders.get(&id).map(|(l, w)| (*l, w as &dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::Button(id) => self.buttons.get(&id).map(|(l, w)| (*l, w as &dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::Checkbox(id) => self.checkboxs.get(&id).map(|(l, w)| (*l, w as &dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::ColorPicker(id) => self.color_pickers.get(&id).map(|(l, w)| (*l, w as &dyn Widget<I, FontHash>)).unwrap(),
        }
    }

    #[inline(always)]
    fn get_widget_mut(&mut self, widget: ActiveWidget) -> (&mut u64, &mut dyn Widget<I, FontHash>) {
        match widget {
            ActiveWidget::Slider(id) => self.sliders.get_mut(&id).map(|(l, w)| (l, w as &mut dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::Button(id) => self.buttons.get_mut(&id).map(|(l, w)| (l, w as &mut dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::Checkbox(id) => self.checkboxs.get_mut(&id).map(|(l, w)| (l, w as &mut dyn Widget<I, FontHash>)).unwrap(),
            ActiveWidget::ColorPicker(id) => self.color_pickers.get_mut(&id).map(|(l, w)| (l, w as &mut dyn Widget<I, FontHash>)).unwrap(),
        }
    }

    #[inline(always)]
    pub fn set_last_frame(&mut self, frame: u64) {
        self.last_frame = frame;
    }

    #[inline(always)]
    pub fn last_frame(&self) -> u64 {
        self.last_frame
    }

    #[inline(always)]
    pub fn bounding_rect(&self, error_margin: f32) -> BoundingRect {
        BoundingRect::from_position_size(self.position - vec2(error_margin, error_margin) * 0.5, self.main_rect.size() + vec2(error_margin, error_margin))
    }

    pub fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_other_window: bool,
    ) -> bool
        where 
            I: Interface,
            FontHash: Clone + Eq + Hash,
    {
        let cursor_in_this_window =
            !cursor_in_other_window &&
            self.bounding_rect(Self::CURSOR_ERROR_MARGIN).is_point_inside(cursor_pos);
        self.flags &= !(Self::CURSOR_IN_WINDOW | Self::HOVER_WINDOW_ACTIVE);
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_this_window as u32;
        let mut min_width: f32 = 0.0;
        let min_height = self.min_height;
        let mut cursor_in_some_widget = false;
        let window_width = self.main_rect.max.x;
        let window_pos = self.position;
        let (active_widgets, mut prev_active_widgets, mut vertices) = unsafe {(
            self.active_widgets.take().unwrap_unchecked(),
            self.prev_active_widgets.take().unwrap_unchecked(),
            self.vertices.take().unwrap_unchecked(),
        )};
        prev_active_widgets.retain(|v| !active_widgets.contains(v));
        for &widget in &prev_active_widgets {
            let (_, widget) = self.get_widget_mut(widget);
            widget.hide(&mut vertices);
        }
        self.prev_active_widgets = Some(prev_active_widgets);
        self.vertices = Some(vertices);
        let mut hover_window = self.hover_window.take().unwrap();
        for &widget in &active_widgets {
            let (_, widget) = self.get_widget_mut(widget);
            let UpdateResult {
                min_widget_width,
                requires_triangulation,
                cursor_in_widget
            } = widget.update(
                nox,
                style,
                text_renderer,
                window_width,
                window_pos,
                cursor_pos,
                cursor_in_this_window,
            );
            min_width = min_width.max(min_widget_width + style.item_pad_outer.x + style.item_pad_outer.x);
            if cursor_in_widget && let Some(hover_text) = widget.hover_text() {
                hover_window.update(style, text_renderer, cursor_pos, hover_text);
                self.flags |= Self::HOVER_WINDOW_ACTIVE;
            }
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_some_widget |= cursor_in_widget;
        }
        self.active_widgets = Some(active_widgets);
        self.hover_window = Some(hover_window);
        let title_text = self.title_text.as_ref().unwrap();
        min_width = min_width.max(style.calc_text_box_width(title_text.text_width) + style.item_pad_outer.x);
        if self.main_rect.max.x < min_width {
            self.main_rect.max.x = min_width;
        }
        let mut main_rect_max = self.main_rect.max;
        let override_cursor = style.override_cursor;
        if self.held() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::HELD;
            } else {
                self.position += delta_cursor_pos;
            }
        }
        if self.resize_left() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_LEFT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cursor_pos.x <= self.position.x {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let delta_width = cursor_pos.x - self.position.x;
                    let new_width = main_rect_max.x - delta_width;
                    if new_width < min_width {
                        self.position.x += main_rect_max.x - min_width;
                        main_rect_max.x = min_width;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                        self.position.x += delta_width;
                    }
                }
            }
        }
        if self.resize_right() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_RIGHT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_col() {
                    if cursor_pos.x - self.position.x >= min_width {
                        self.flags &= !Self::RESIZE_BLOCKED_COL;
                    }
                } else {
                    let new_width = cursor_pos.x - self.position.x;
                    if new_width < min_width {
                        main_rect_max.x = min_width;
                        self.flags |= Self::RESIZE_BLOCKED_COL;
                    } else {
                        main_rect_max.x = new_width;
                    }
                }
            }
        }
        if self.resize_top() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_TOP;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cursor_pos.y <= self.position.y {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                }
                else {
                    let delta_height = cursor_pos.y - self.position.y;
                    let new_height = main_rect_max.y - delta_height;
                    if new_height < min_height {
                        self.position.y += main_rect_max.y - min_height;
                        main_rect_max.y = min_height;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                        self.position.y = cursor_pos.y;
                    }
                } 
            }
        }
        if self.resize_bottom() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_BOTTOM;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                if self.resize_blocked_row() {
                    if cursor_pos.y - self.position.y >= min_height {
                        self.flags &= !Self::RESIZE_BLOCKED_ROW;
                    }
                } else {
                    let new_height = cursor_pos.y - self.position.y;
                    if new_height < self.min_height {
                        main_rect_max.y = min_height;
                        self.flags |= Self::RESIZE_BLOCKED_ROW;
                    } else {
                        main_rect_max.y = new_height;
                    }
                }
            }
        }
        if !self.held() && !self.any_resize() && cursor_in_this_window && !cursor_in_some_widget {
            let mut flags = self.flags;
            flags &= !Self::RESIZE_BLOCKED_COL;
            flags &= !Self::RESIZE_BLOCKED_ROW;
            let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
            if cursor_pos.x >= self.position.x - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.x <= self.position.x + Self::CURSOR_ERROR_MARGIN 
            {
                flags |= Self::RESIZE_LEFT;
            }
            if cursor_pos.x >= self.position.x + self.main_rect.max.x - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.x <= self.position.x + self.main_rect.max.x + Self::CURSOR_ERROR_MARGIN
            {
                flags |= Self::RESIZE_RIGHT;
            }
            if cursor_pos.y >= self.position.y - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.y <= self.position.y + Self::CURSOR_ERROR_MARGIN
            {
                flags |= Self::RESIZE_TOP;
            }
            if cursor_pos.y >= self.position.y + self.main_rect.max.y - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.y <= self.position.y + self.main_rect.max.y + Self::CURSOR_ERROR_MARGIN
            {
                flags |= Self::RESIZE_BOTTOM;
            }
            self.flags = flags;
            if !self.any_resize()
            {
                if BoundingRect
                    ::from_position_size(self.position, self.title_bar_rect.max)
                    .is_point_inside(cursor_pos)
                {
                    self.flags |= Self::HELD * mouse_pressed as u32;
                }
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            }
            else if override_cursor {
                if self.resize_nw() {
                    nox.set_cursor(CursorIcon::NwResize);
                }
                else if self.resize_ne() {
                    nox.set_cursor(CursorIcon::NeResize);
                }
                else if self.resize_sw() {
                    nox.set_cursor(CursorIcon::SwResize);
                }
                else if self.resize_se() {
                    nox.set_cursor(CursorIcon::SeResize);
                }
                else {
                    if self.resize_left() || self.resize_right() {
                        nox.set_cursor(CursorIcon::ColResize);
                    }
                    if self.resize_top() || self.resize_bottom() {
                        nox.set_cursor(CursorIcon::RowResize);
                    }
                }
            }
            self.flags &= !((Self::RESIZE_LEFT | Self::RESIZE_RIGHT | Self::RESIZE_TOP | Self::RESIZE_BOTTOM) * !mouse_pressed as u32);
        }
        if main_rect_max.y < self.min_height {
            main_rect_max.y = self.min_height;
        }
        let mut title_bar_rect = self.title_bar_rect;
        title_bar_rect.max.x = self.main_rect.max.x;
        title_bar_rect.max.y = style.calc_text_box_height(title_text.row_height);
        let mut separator_rect = self.separator_rect;
        separator_rect.max.x = self.main_rect.max.x;
        separator_rect.max.y = style.separator_height;
        let requires_triangulation =
            (style.rounding != self.main_rect.rounding ||
            self.outline_width != style.outline_width ||
            self.outline_thin_width != style.outline_thin_width ||
            main_rect_max != self.main_rect.max ||
            self.title_bar_rect != title_bar_rect ||
            self.separator_rect != separator_rect
        ) as u32;
        self.flags |= Self::REQUIRES_TRIANGULATION * requires_triangulation;
        self.main_rect.rounding = style.rounding;
        self.outline_width = style.outline_width;
        self.outline_thin_width = style.outline_thin_width;
        self.main_rect.max = main_rect_max;
        self.title_bar_rect = title_bar_rect;
        self.separator_rect = separator_rect;
        cursor_in_this_window || self.any_resize()
    }

    #[inline(always)]
    pub fn triangulate(&mut self) {
        if self.requires_triangulation() {
            let new_triangulation = self.last_triangulation + 1;
            self.flags |= Self::RENDERABLE;
            let mut vertices = self.vertices.take().unwrap();
            vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            let mut outline_points = GlobalVec::new();
            nox_geom::shapes::outline_points(&points, self.outline_width, false, &mut |p| { outline_points.push(p.into()); });
            if !earcut::earcut(&outline_points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.outline_vertex_range = 0..vertices.len();
            outline_points.clear();
            nox_geom::shapes::outline_points(&points, self.outline_thin_width, false, &mut |p| { outline_points.push(p.into()); });
            let mut vertex_begin = vertices.len();
            if !earcut::earcut(&outline_points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.outline_thin_vertex_range = vertex_begin..vertices.len();
            vertex_begin = vertices.len();
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_vertex_range = vertex_begin..vertices.len();
            points.clear();
            self.title_bar_rect.to_points_partial_round(true, true, false, false,
                &mut |p| { points.push(p.into()); }
            );
            vertex_begin = vertices.len();
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.title_bar_vertex_range = vertex_begin..vertices.len();
            points.clear();
            self.separator_rect.to_points_no_round(
                &mut |p| { points.push(p.into()); }
            );
            vertex_begin = vertices.len();
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.separator_vertex_range = vertex_begin..vertices.len();
            let active_widgets = self.active_widgets.take().unwrap();
            let mut flags = self.flags;
            for &widget in &active_widgets  {
                let (last_triangulation, widget) = self.get_widget_mut(widget);
                *last_triangulation = new_triangulation;
                points.clear();
                widget.triangulate(&mut points,
                    &mut |points: &[[f32; 2]]| {
                        let vertex_begin = vertices.len();
                        if !earcut::earcut(points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                            flags &= !Self::RENDERABLE;
                        }
                        vertex_begin..vertices.len()
                    }
                );
            }
            self.main_draw_info = DrawInfo {
                first_index: 0,
                index_count: indices_usize.len() as u32,
                ..Default::default()
            };
            self.active_widgets = Some(active_widgets);
            self.vertices = Some(vertices);
            self.flags = flags;
            self.flags &= !Self::REQUIRES_TRIANGULATION;
            self.indices.append_map(&indices_usize, |&i| i as u32);
            self.last_triangulation = new_triangulation;
        }
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
    ) -> Result<(), Error>
    {
        if !self.renderable() {
            return Ok(())
        }
        let vert_total = unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked().len()
        };
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        let idx_total = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_total)?
        };
        let active_widgets = unsafe {
            self.active_widgets.take().unwrap_unchecked()
        };
        let mut vertices = unsafe {
            self.vertices.take().unwrap_unchecked()
        };
        for &widget in &active_widgets {
            let (_, widget) = self.get_widget_mut(widget);
            widget.set_vertex_params(style, &mut vertices);
        }
        self.active_widgets = Some(active_widgets);
        let vertex_sample = vertices[self.main_rect_vertex_range.start];
        if vertex_sample.color != style.window_bg_col {
            let target_color = style.window_bg_col;
            for vertex in &mut vertices[self.main_rect_vertex_range.clone()] {
                vertex.color = target_color;
            }
        }
        let vertex_sample = vertices[self.title_bar_vertex_range.start];
        if vertex_sample.color != style.window_title_bar_col {
            let target_color = style.window_title_bar_col;
            for vertex in &mut vertices[self.title_bar_vertex_range.clone()] {
                vertex.color = target_color;
            }
        }
        let vertex_sample = vertices[self.separator_vertex_range.start];
        let offset = vec2(0.0, self.title_bar_rect.max.y - self.separator_rect.max.y * 0.5);
        if vertex_sample.offset != offset || vertex_sample.color != style.separator_col {
            let target_color = style.separator_col;
            for vertex in &mut vertices[self.separator_vertex_range.clone()] {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
        let any_resize = self.any_resize();
        if self.cursor_in_window() || any_resize {
            let vertex_sample = vertices[self.outline_vertex_range.start];
            let offset = vec2(0.0, 0.0);
            let target_color = if any_resize || self.held() {
                style.outline_col_hl
            } else {
                style.outline_col
            };
            if vertex_sample.offset != offset || vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_vertex_range.clone()] {
                    vertex.offset = offset;
                    vertex.color = target_color;
                }
            }
            let vertex_sample = vertices[self.outline_thin_vertex_range.start];
            if vertex_sample.color.a != 0.0 {
                for vertex in &mut vertices[self.outline_thin_vertex_range.clone()] {
                    vertex.color = ColorRGBA::transparent_black();
                }
            }
        } else {
            let vertex_sample = vertices[self.outline_vertex_range.start];
            let target_color = ColorRGBA::transparent_black();
            if vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_vertex_range.clone()] {
                    vertex.color = target_color;
                }
            }
            let vertex_sample = vertices[self.outline_thin_vertex_range.start];
            let offset = vec2(0.0, 0.0);
            let target_color = style.outline_thin_col;
            if vertex_sample.offset != offset || vertex_sample.color != target_color {
                for vertex in &mut vertices[self.outline_thin_vertex_range.clone()] {
                    vertex.offset = offset;
                    vertex.color = target_color;
                }
            }
        }
        self.vertices = Some(vertices);
        unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked()
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_total);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_total);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position,
            vec2(1.0, 1.0),
            inv_aspect_ratio,
        );
        render_commands.push_constants(|_| unsafe {
            pc_vertex.as_bytes()
        })?;
        render_commands.draw_indexed(
            self.main_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position + vec2(style.item_pad_outer.x, style.item_pad_inner.y),
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio,
        );
        let pc_fragment = text_push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_text(self.title_text.as_ref().unwrap(), render_commands, vertex_buffer, index_buffer)?;
        for &widget in unsafe { self.active_widgets.as_ref().unwrap_unchecked() } {
            let (_, widget) = self.get_widget(widget);
            widget.render_commands(
                render_commands,
                style,
                base_pipeline_id,
                text_pipeline_id,
                vertex_buffer,
                index_buffer,
                self.position,
                inv_aspect_ratio,
            )?;
        }
        if self.hover_window_active() {
            let hover_window = unsafe { self.hover_window
                    .as_mut()
                    .unwrap_unchecked()
            };
            hover_window.set_vertex_params(style);
            hover_window.render_commands(
                render_commands,
                style,
                base_pipeline_id,
                text_pipeline_id,
                vertex_buffer,
                index_buffer,
                inv_aspect_ratio
            )?;
        }
        unsafe {
            self.prev_active_widgets
                .as_mut()
                .unwrap_unchecked()
                .move_from_vec(self.active_widgets.as_mut().unwrap_unchecked()).unwrap();
        }
        Ok(())
    }
}

pub struct WindowContext<'a, 'b, I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash, 
{
    style: &'a Style<FontHash>,
    window: &'a mut Window<I, FontHash>,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    widget_y: f32,
}

impl<'a, 'b, I, FontHash> WindowContext<'a, 'b, I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    pub(crate) fn new(
        window: &'a mut Window<I, FontHash>,
        style: &'a Style<FontHash>,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    ) -> Self {
        let title_text = window.title_text.get_or_insert(text_renderer.render(
            &[text_segment(window.title.as_str(), &style.font_regular)],
            false,
            0.0,
        ).unwrap_or_default());
        Self {
            widget_y: style.calc_text_box_height(title_text.row_height) + style.item_pad_inner.y,
            window,
            style,
            text_renderer,
        }
    }

    pub fn update_button(
        &mut self,
        id: u32,
        title: &str,
    ) -> bool
    {
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Button(id));
        }
        let (last_triangulation, button) = self.window.buttons
            .entry(id)
            .or_insert((0, Button::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash>::REQUIRES_TRIANGULATION;
        }
        button.set_offset(vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += button.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
        button.pressed()
    }

    pub fn update_slider<T: Sliderable>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        min: T,
        max: T,
    ) -> Result<(), Error>
    { 
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Slider(id));
        }
        let (last_triangulation, slider) = self.window.sliders
            .entry(id)
            .or_insert((0, Slider::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash>::REQUIRES_TRIANGULATION;
        }
        if slider.held() {
            slider.quantized_t = value.slide_and_quantize_t(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
            slider.quantized_t = slider.t;
        }
        slider.hover_text.clear();
        value
            .display(self.style, &mut slider.hover_text)
            .map_err(|e| {
                Error::UserError(format!("nox_gui: failed to format slider value: {}", e))
            })?;
        slider.set_offset(vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += slider.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
        Ok(())
    }

    pub fn update_checkbox(
        &mut self,
        id: u32,
        title: &str,
        value: &mut bool,
    ) -> bool
    {
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Checkbox(id));
        }
        let (last_triangulation, checkbox) = self.window.checkboxs
            .entry(id)
            .or_insert((0, Checkbox::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash>::REQUIRES_TRIANGULATION;
        }
        if checkbox.pressed() {
            *value = !*value;
        }
        checkbox.set_checked(*value);
        checkbox.set_offset(vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += checkbox.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
        *value
    }

    pub fn update_color_picker(
        &mut self,
        id: u32,
        title: &str,
        _value: &mut ColorRGBA,
    )
    {
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::ColorPicker(id));
        }
        let (last_triangulation, color_picker) = self.window.color_pickers
            .entry(id)
            .or_insert((0, ColorPicker::new(title)));
        if *last_triangulation != self.window.last_triangulation {
            self.window.flags |= Window::<I, FontHash>::REQUIRES_TRIANGULATION;
        }
        color_picker.set_offset(vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += color_picker.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
    }
}

impl<'a, 'b, I, FontHash> Drop for WindowContext<'a, 'b, I, FontHash>
    where 
        I: Interface,
        FontHash: Clone + Eq + Hash,
{
    fn drop(&mut self) {
        self.window.min_height = self.widget_y + self.style.item_pad_outer.y
    }
}
