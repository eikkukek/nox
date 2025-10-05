use core::{
    hash::Hash,
};

use nox::{
    mem::{
        value_as_bytes, vec_types::{GlobalVec, Vector}
    },
    *,
};

use rustc_hash::FxHashMap;

use compact_str::CompactString;

use nox_font::{VertexTextRenderer, text_segment, RenderedText};

use nox_geom::{
    shapes::*, *
};

use crate::*;

pub(crate) struct Window {
    main_rect: Rect,
    main_rect_draw_info: DrawInfo,
    title_bar_rect: Rect,
    title_bar_rect_draw_info: DrawInfo,
    position: Vec2,
    title: CompactString,
    title_text: Option<RenderedText>,
    vertices: GlobalVec<Vertex>,
    indices: GlobalVec<u32>,
    sliders: FxHashMap<u32, Slider>,
    buttons: FxHashMap<u32, Button>,
    active_sliders: GlobalVec<u32>,
    active_buttons: GlobalVec<u32>,
    min_height: f32,
    flags: u32,
}

impl Window {

    const RENDERABLE: u32 = 0x1;
    const REQUIRES_TRIANGULATION: u32 = 0x2;
    const CURSOR_IN_WINDOW: u32 = 0x4;
    const HELD: u32 = 0x8;
    const RESIZE_LEFT: u32 = 0x10;
    const RESIZE_RIGHT: u32 = 0x20;
    const RESIZE_TOP: u32 = 0x40;
    const RESIZE_BOTTOM: u32 = 0x80;

    const CURSOR_ERROR_MARGIN: f32 = 0.01;

    pub(crate) fn new(
        title: &str,
        size: [f32; 2],
        position: [f32; 2],
        rounding: f32,
    ) -> Self
    {
        Self {
            main_rect: rect(Default::default(), size, rounding),
            main_rect_draw_info: Default::default(),
            title_bar_rect: rect::<Vec2>(Default::default(), Default::default(), rounding),
            title_bar_rect_draw_info: Default::default(),
            position: position.into(),
            title: title.into(),
            title_text: None,
            vertices: Default::default(),
            indices: Default::default(),
            sliders: FxHashMap::default(),
            buttons: FxHashMap::default(),
            active_sliders: Default::default(),
            active_buttons: Default::default(),
            min_height: 0.0,
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
    fn any_resize(&self) -> bool {
        self.resize_left() ||
        self.resize_right() ||
        self.resize_top() ||
        self.resize_bottom()
    }

    pub(crate) fn bounding_rect(&self, error_margin: f32) -> BoundingRect {
        BoundingRect::from_position_size(self.position - vec2(error_margin, error_margin), self.main_rect.size() + vec2(error_margin, error_margin))
    }

    pub(crate) fn update<I, FontHash>(
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
        self.flags &= !Self::CURSOR_IN_WINDOW;
        self.flags |= Self::CURSOR_IN_WINDOW * cursor_in_this_window as u32;
        let title_text = self.title_text.as_ref().unwrap();
        let mut min_width = style.calc_text_width(title_text.text_width) + style.item_pad_outer.x * 2.0;
        if self.main_rect.max.x < min_width {
            self.main_rect.max.x = min_width;
        }
        let mut cursor_in_item = false;
        for id in &self.active_sliders {
            let slider = self.sliders.get_mut(id).unwrap();
            let (requires_triangulation, cursor_in_slider, min_win_width) = slider.update(
                nox,
                style,
                text_renderer,
                self.main_rect.max.x,
                cursor_pos,
                cursor_in_this_window,
            );
            min_width = min_width.max(min_win_width);
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_item |= cursor_in_slider;
        }
        for id in &self.active_buttons {
            let button = self.buttons.get_mut(id).unwrap();
            let (requires_triangulation, cursor_in_button, min_win_width) = button.update(
                nox,
                style,
                text_renderer,
                self.main_rect.max.x,
                cursor_pos,
                cursor_in_this_window,
            );
            min_width = min_width.max(min_win_width);
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_item |= cursor_in_button;
        }
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
        else if self.resize_left() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_LEFT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                main_rect_max.x -= delta_cursor_pos.x;
                if main_rect_max.x < min_width {
                    main_rect_max.x = min_width;
                    self.flags &= !Self::RESIZE_LEFT;
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Default);
                    }
                } else {
                    self.position.x += delta_cursor_pos.x;
                }
            }
        }
        else if self.resize_right() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_RIGHT;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                main_rect_max.x += delta_cursor_pos.x;
                if main_rect_max.x < min_width {
                    main_rect_max.x = min_width;
                    self.flags &= !Self::RESIZE_RIGHT;
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Default);
                    }
                }
            }
        }
        else if self.resize_top() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_TOP;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                main_rect_max.y -= delta_cursor_pos.y;
                if main_rect_max.y < self.min_height {
                    main_rect_max.y = self.min_height;
                    self.flags &= !Self::RESIZE_TOP;
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Default);
                    }
                } else {
                    self.position.y += delta_cursor_pos.y;
                }
            }
        }
        else if self.resize_bottom() {
            if !nox.is_mouse_button_held(MouseButton::Left) {
                self.flags &= !Self::RESIZE_BOTTOM;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            } else {
                main_rect_max.y += delta_cursor_pos.y;
                if main_rect_max.y < self.min_height {
                    main_rect_max.y = self.min_height;
                    self.flags &= !Self::RESIZE_BOTTOM;
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Default);
                    }
                }
            }
        }
        else if cursor_in_this_window && !cursor_in_item {
            let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left) as u32;
            if cursor_pos.x >= self.position.x - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.x <= self.position.x + Self::CURSOR_ERROR_MARGIN 
            {
                self.flags |= Self::RESIZE_LEFT * mouse_pressed;
                if override_cursor {
                    nox.set_cursor(CursorIcon::ColResize);
                }
            }
            else if cursor_pos.x >= self.position.x + self.main_rect.max.x - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.x <= self.position.x + self.main_rect.max.x + Self::CURSOR_ERROR_MARGIN
            {
                self.flags |= Self::RESIZE_RIGHT * mouse_pressed;
                if override_cursor {
                    nox.set_cursor(CursorIcon::ColResize);
                }
            }
            else if cursor_pos.y >= self.position.y - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.y <= self.position.y + Self::CURSOR_ERROR_MARGIN
            {
                self.flags |= Self::RESIZE_TOP * mouse_pressed;
                if override_cursor {
                    nox.set_cursor(CursorIcon::RowResize);
                }
            }
            else if cursor_pos.y >= self.position.y + self.main_rect.max.y - Self::CURSOR_ERROR_MARGIN &&
                cursor_pos.y <= self.position.y + self.main_rect.max.y + Self::CURSOR_ERROR_MARGIN
            {
                self.flags |= Self::RESIZE_BOTTOM * mouse_pressed;
                if override_cursor {
                    nox.set_cursor(CursorIcon::RowResize);
                }
            }
            else if BoundingRect
                    ::from_position_size(self.position, self.title_bar_rect.max)
                    .is_point_inside(cursor_pos)
            {
                self.flags |= Self::HELD * mouse_pressed;
                if override_cursor {
                    nox.set_cursor(CursorIcon::Default);
                }
            }
            else if override_cursor {
                nox.set_cursor(CursorIcon::Default);
            }
        }
        if main_rect_max != self.main_rect.max {
            self.main_rect.max = main_rect_max;
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        let mut title_bar_rect = self.title_bar_rect;
        title_bar_rect.max.x = self.main_rect.max.x;
        title_bar_rect.max.y = style.calc_text_box_height(title_text.font_height);
        if self.title_bar_rect != title_bar_rect {
            self.title_bar_rect = title_bar_rect;
            self.flags |= Self::REQUIRES_TRIANGULATION;
        }
        cursor_in_this_window
    }

    #[inline(always)]
    pub(crate) fn triangulate(&mut self) {
        if self.requires_triangulation() {
            self.flags |= Self::RENDERABLE;
            self.vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            self.main_rect_draw_info = DrawInfo {
                first_index: 0,
                index_count: indices_usize.len() as u32,
                ..Default::default()
            };
            points.clear();
            let mut title_bar_rect_draw_info = DrawInfo {
                first_index: indices_usize.len() as u32,
                ..Default::default()
            };
            self.title_bar_rect.to_points_partial_round(true, true, false, false,
                &mut |p| { points.push(p.into()); }
            );
            if !earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            title_bar_rect_draw_info.index_count = indices_usize.len() as u32 - title_bar_rect_draw_info.first_index;
            self.title_bar_rect_draw_info = title_bar_rect_draw_info;
            let mut tri = |points: &[[f32; 2]]| {
                let mut draw_info = DrawInfo {
                    first_index: indices_usize.len() as u32,
                    ..Default::default()
                };
                if !earcut::earcut(points, &[], false, &mut self.vertices, &mut indices_usize).unwrap() {
                    self.flags &= !Self::RENDERABLE;
                }
                draw_info.index_count = indices_usize.len() as u32 - draw_info.first_index;
                draw_info
            };
            for id in &self.active_sliders {
                let slider = self.sliders.get_mut(id).unwrap();
                slider.triangulate(&mut points, &mut tri);
            }
            for id in &self.active_buttons {
                let button = self.buttons.get_mut(id).unwrap();
                button.triangulate(&mut points, &mut tri);
            }
            self.indices.append_map(&indices_usize, |&i| i as u32);
            self.flags &= !Self::REQUIRES_TRIANGULATION;
        }
    }

    pub(crate) fn render_commands<FontHash>(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        inv_aspect_ratio: f32,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        base_pipeline: GraphicsPipelineId,
        no_offset: DrawBufferInfo,
    ) -> Result<(), Error>
    {
        if !self.renderable() {
            return Ok(())
        }
        let vert_total = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_total);
        }
        let idx_total = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_total)?
        };
        unsafe {
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_total);
        }
        render_commands.bind_pipeline(base_pipeline)?;
        let any_resize = self.any_resize();
        if self.cursor_in_window() || any_resize {
            let pc_vertex = style.calc_outline_push_constant(
                self.position,
                self.main_rect.max,
                inv_aspect_ratio
            );
            let pc_fragment = push_constants_fragment(
                if self.held() || any_resize {
                    style.outline_col_hl
                } else {
                    style.outline_col
                }
            );
            render_commands.push_constants(|pc| unsafe {
                if pc.stage == ShaderStage::Vertex {
                    value_as_bytes(&pc_vertex).unwrap()
                } else {
                    value_as_bytes(&pc_fragment).unwrap()
                }
            })?;
            render_commands.draw_indexed(
                self.main_rect_draw_info,
                [
                    DrawBufferInfo {
                        id: vertex_buffer.id(),
                        offset: vert_mem.offset,
                    },
                    no_offset,
                ],
                DrawBufferInfo {
                    id: index_buffer.id(),
                    offset: idx_mem.offset,
                },
            )?;
        }
        let pc_vertex = push_constants_vertex(
            self.position,
            vec2(1.0, 1.0),
            inv_aspect_ratio,
        );
        let pc_fragment = push_constants_fragment(style.window_bg_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            self.main_rect_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer.id(),
                    offset: vert_mem.offset,
                },
                no_offset,
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        let pc_fragment = push_constants_fragment(style.window_title_bar_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_commands.draw_indexed(
            self.title_bar_rect_draw_info,
            [
                DrawBufferInfo {
                    id: vertex_buffer.id(),
                    offset: vert_mem.offset,
                },
                no_offset,
            ],
            DrawBufferInfo {
                id: index_buffer.id(),
                offset: idx_mem.offset,
            },
        )?;
        let pc_vertex = push_constants_vertex(
            self.position + vec2(style.item_pad_outer.x, style.item_pad_inner.y),
            vec2(style.font_scale, style.font_scale),
            inv_aspect_ratio,
        );
        let pc_fragment = push_constants_fragment(style.text_col);
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                value_as_bytes(&pc_vertex).unwrap()
            } else {
                value_as_bytes(&pc_fragment).unwrap()
            }
        })?;
        render_text(self.title_text.as_ref().unwrap(), render_commands, vertex_buffer, index_buffer)?;
        for id in &self.active_sliders {
            let slider = self.sliders.get(id).unwrap();
            slider.render_commands(
                render_commands,
                style,
                inv_aspect_ratio,
                vertex_buffer,
                index_buffer,
                vert_mem.offset,
                idx_mem.offset,
                no_offset,
            )?;
        }
        for id in &self.active_buttons {
            let button = self.buttons.get(id).unwrap();
            button.render_commands(
                render_commands,
                style,
                inv_aspect_ratio,
                vertex_buffer,
                index_buffer,
                vert_mem.offset,
                idx_mem.offset,
                no_offset
            )?;
        }
        self.active_sliders.clear();
        self.active_buttons.clear();
        Ok(())
    }
}

pub struct WindowContext<'a, 'b, FontHash>
    where
        FontHash: Clone + Eq + Hash, 
{
    style: &'a Style<FontHash>,
    window: &'a mut Window,
    text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    widget_y: f32,
}

impl<'a, 'b, FontHash> WindowContext<'a, 'b, FontHash>
    where
        FontHash: Clone + Eq + Hash,
{

    pub(crate) fn new(
        window: &'a mut Window,
        style: &'a Style<FontHash>,
        text_renderer: &'a mut VertexTextRenderer<'b, FontHash>,
    ) -> Self {
        let title_text = window.title_text.get_or_insert(text_renderer.render(
            &[text_segment(window.title.as_str(), &style.font_regular)],
            false,
            0.0,
        ).unwrap_or_default());
        Self {
            widget_y: style.calc_text_box_height(title_text.font_height) + style.item_pad_inner.y,
            window,
            style,
            text_renderer,
        }
    }

    pub fn update_slider<T: Sliderable>(
        &mut self,
        id: u32,
        title: &str,
        value: &mut T,
        min: T,
        max: T,
    )
    { 
        self.window.active_sliders.push(id);
        let slider = self.window.sliders
            .entry(id)
            .or_insert(Slider::new(value.calc_t(min, max), title.into()));
        if slider.held() {
            value.slide(min, max, slider.t);
        } else {
            slider.t = value.calc_t(min, max);
        }
        slider.set_position(self.window.position + vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += slider.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
    }

    pub fn update_button(
        &mut self,
        id: u32,
        title: &str,
    ) -> bool
    {
        self.window.active_buttons.push(id);
        let button = self.window.buttons
            .entry(id)
            .or_insert(Button::new(title));
        button.set_position(self.window.position + vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += button.calc_size(&self.style, self.text_renderer).y + self.style.item_pad_outer.y;
        button.pressed()
    }
}

impl<'a, 'b, FontHash> Drop for WindowContext<'a, 'b, FontHash>
    where 
        FontHash: Clone + Eq + Hash,
{
    fn drop(&mut self) {
        self.window.min_height = self.widget_y + self.style.item_pad_outer.y
    }
}
