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

#[derive(Clone, Copy)]
enum ActiveWidget {
    Slider(u32),
    Button(u32),
    Checkbox(u32),
}

pub(crate) struct Window<I, FontHash>
{
    main_rect: Rect,
    main_rect_draw_info: DrawInfo,
    title_bar_rect: Rect,
    title_bar_rect_draw_info: DrawInfo,
    position: Vec2,
    title: CompactString,
    title_text: Option<RenderedText>,
    vertices: Option<GlobalVec<Vertex>>,
    indices: GlobalVec<u32>,
    buttons: FxHashMap<u32, Button<I, FontHash>>,
    sliders: FxHashMap<u32, Slider<I, FontHash>>,
    checkboxs: FxHashMap<u32, Checkbox<I, FontHash>>,
    active_widgets: Option<GlobalVec<ActiveWidget>>,
    min_height: f32,
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
            vertices: Some(Default::default()),
            indices: Default::default(),
            buttons: FxHashMap::default(),
            sliders: FxHashMap::default(),
            checkboxs: FxHashMap::default(),
            active_widgets: Some(Default::default()),
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

    #[inline(always)]
    fn get_widget(&self, widget: ActiveWidget) -> &dyn Widget<I, FontHash> {
        match widget {
            ActiveWidget::Slider(id) => self.sliders.get(&id).unwrap(),
            ActiveWidget::Button(id) => self.buttons.get(&id).unwrap(),
            ActiveWidget::Checkbox(id) => self.checkboxs.get(&id).unwrap(),
        }
    }

    #[inline(always)]
    fn get_widget_mut(&mut self, widget: ActiveWidget) -> &mut dyn Widget<I, FontHash> {
        match widget {
            ActiveWidget::Slider(id) => self.sliders.get_mut(&id).unwrap(),
            ActiveWidget::Button(id) => self.buttons.get_mut(&id).unwrap(),
            ActiveWidget::Checkbox(id) => self.checkboxs.get_mut(&id).unwrap(),
        }
    }

    pub(crate) fn bounding_rect(&self, error_margin: f32) -> BoundingRect {
        BoundingRect::from_position_size(self.position - vec2(error_margin, error_margin), self.main_rect.size() + vec2(error_margin, error_margin))
    }

    pub(crate) fn update(
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
        let mut min_width: f32 = 0.0;
        let mut cursor_in_item = false;
        let window_width = self.main_rect.max.x;
        let active_widgets = self.active_widgets.take().unwrap();
        for &widget in &active_widgets {
            let widget = self.get_widget_mut(widget);
            let UpdateResult {
                min_widget_width,
                requires_triangulation,
                cursor_in_widget
            } = widget.update(
                nox,
                style,
                text_renderer,
                window_width,
                cursor_pos,
                cursor_in_this_window,
            );
            min_width = min_width.max(min_widget_width + style.item_pad_outer.x + style.item_pad_outer.x);
            if requires_triangulation {
                self.flags |= Self::REQUIRES_TRIANGULATION;
            }
            cursor_in_item |= cursor_in_widget;
        }
        self.active_widgets = Some(active_widgets);
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
            let mut vertices = self.vertices.take().unwrap();
            vertices.clear();
            self.indices.clear();
            let mut points = GlobalVec::new();
            let mut indices_usize = GlobalVec::new();
            self.main_rect.to_points(&mut |p| { points.push(p.into()); });
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE
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
            if !earcut::earcut(&points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                self.flags &= !Self::RENDERABLE;
            }
            title_bar_rect_draw_info.index_count = indices_usize.len() as u32 - title_bar_rect_draw_info.first_index;
            self.title_bar_rect_draw_info = title_bar_rect_draw_info;
            let active_widgets = self.active_widgets.take().unwrap();
            let mut flags = self.flags;
            for &widget in &active_widgets  {
                let widget = self.get_widget_mut(widget);
                widget.triangulate(&mut points, &mut |points: &[[f32; 2]]| {
                    let mut draw_info = DrawInfo {
                        first_index: indices_usize.len() as u32,
                        ..Default::default()
                    };
                    if !earcut::earcut(points, &[], false, &mut vertices, &mut indices_usize).unwrap() {
                        flags &= !Self::RENDERABLE;
                    }
                    draw_info.index_count = indices_usize.len() as u32 - draw_info.first_index;
                    draw_info
                });
            }
            self.active_widgets = Some(active_widgets);
            self.vertices = Some(vertices);
            self.flags = flags;
            self.indices.append_map(&indices_usize, |&i| i as u32);
            self.flags &= !Self::REQUIRES_TRIANGULATION;
        }
    }

    pub(crate) fn render_commands(
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
        let vert_total = unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked().len()
        };
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_total)?
        };
        unsafe {
            self.vertices
                .as_ref()
                .unwrap_unchecked()
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
                    pc_vertex.as_bytes()
                } else {
                    pc_fragment.as_bytes()
                }
            })?;
            render_commands.draw_indexed(
                self.main_rect_draw_info,
                [
                    DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
                    no_offset,
                ],
                DrawBufferInfo::new(index_buffer.id(), idx_mem.offset),
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
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.main_rect_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
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
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            self.title_bar_rect_draw_info,
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
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
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_text(self.title_text.as_ref().unwrap(), render_commands, vertex_buffer, index_buffer)?;
        for &widget in unsafe { self.active_widgets.as_ref().unwrap_unchecked() } {
            let slider = self.get_widget(widget);
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
        unsafe {
            self.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .clear();
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
            widget_y: style.calc_text_box_height(title_text.font_height) + style.item_pad_inner.y,
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
        let button = self.window.buttons
            .entry(id)
            .or_insert(Button::new(title));
        button.set_position(self.window.position + vec2(self.style.item_pad_outer.x, self.widget_y));
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
    )
    { 
        unsafe {
            self.window.active_widgets
                .as_mut()
                .unwrap_unchecked()
                .push(ActiveWidget::Slider(id));
        }
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
        let checkbox = self.window.checkboxs
            .entry(id)
            .or_insert(Checkbox::new(title, *value));
        if checkbox.pressed() {
            *value = !*value;
        }
        checkbox.set_checked(*value);
        checkbox.set_position(self.window.position + vec2(self.style.item_pad_outer.x, self.widget_y));
        self.widget_y += checkbox.calc_size(&self.style, self.text_renderer).y;
        *value
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
