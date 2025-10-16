use core::{
    hash::Hash,
    marker::PhantomData,
    fmt::{Display, Write},
    str::FromStr,
};

use compact_str::CompactString;

use nox::{mem::vec_types::{GlobalVec, Vector}, *};

use nox_geom::{
    shapes::*,
    *,
};

use nox_font::{RenderedText, VertexTextRenderer, text_segment};

use crate::*;

pub(crate) struct InputText<I, FontHash> {
    offset: Vec2,
    title: CompactString,
    title_text: Option<RenderedText>,
    input: CompactString,
    input_text: Option<RenderedText>,
    input_offsets: GlobalVec<Vec2>,
    input_text_offset_x: f32,
    selection: Option<(usize, usize)>,
    cursor_rect_vertex_range: VertexRange,
    text_cursor_pos: usize,
    input_rect: Rect,
    input_rect_vertex_range: VertexRange,
    selection_rect_vertices: [Vertex; 4],
    cursor_rect: Rect,
    flags: u32,
    cursor_timer: f32,
    _marker: PhantomData<(I, FontHash)>,
}

impl<I, FontHash> InputText<I, FontHash>
{

    const HOVERING: u32 = 0x1;
    const HELD: u32 = 0x2;
    const ACTIVATED: u32 = 0x4;
    const ACTIVE: u32 = 0x8;
    const CURSOR_VISIBLE: u32 = 0x10;
    const SELECTION_LEFT: u32 = 0x20;

    const SELECTION_INDICES: [u32; 6] = [
        3, 1, 0,
        1, 3, 2,
    ];

    #[inline(always)]
    pub fn new(title: &str) -> Self {
        Self {
            offset: Default::default(),
            title: CompactString::new(title),
            title_text: None,
            input: Default::default(),
            input_offsets: Default::default(),
            input_text_offset_x: 0.0,
            text_cursor_pos: Default::default(),
            selection: None,
            input_text: None,
            input_rect: Default::default(),
            input_rect_vertex_range: Default::default(),
            selection_rect_vertices: Default::default(),
            cursor_rect: Default::default(),
            cursor_rect_vertex_range: Default::default(),
            flags: 0,
            cursor_timer: 0.0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn hovering(&self) -> bool {
        self.flags & Self::HOVERING == Self::HOVERING
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn activated(&self) -> bool {
        self.flags & Self::ACTIVATED == Self::ACTIVATED
    }

    #[inline(always)]
    pub fn active(&self) -> bool {
        self.flags & Self::ACTIVE == Self::ACTIVE
    }

    #[inline(always)]
    fn cursor_visible(&self) -> bool {
        self.flags & Self::CURSOR_VISIBLE == Self::CURSOR_VISIBLE
    }

    #[inline(always)]
    fn selection_left(&self) -> bool {
        self.flags & Self::SELECTION_LEFT == Self::SELECTION_LEFT
    }

    #[inline(always)]
    fn toggle_cursor_visible(&mut self) {
        self.flags ^= Self::CURSOR_VISIBLE;
    }

    #[inline(always)]
    pub fn set_input(&mut self, input: &impl Display) {
        self.input.clear();
        write!(self.input, "{}", input).ok();
        self.input_text = None;
        self.text_cursor_pos = 0;
    }

    #[inline(always)]
    pub fn get_input<T: FromStr>(&mut self) -> Option<T> {
        T::from_str(&self.input).ok()
    }

    #[inline(always)]
    fn calc_cursor_index(
        &self,
        rel_cursor_pos: f32,
        text_min_max: (f32, f32),
        text_box_min_max: (f32, f32),
        text_offset: f32,
        input_offset: f32,
    ) -> usize
    {
        if rel_cursor_pos < text_box_min_max.0 {
            for i in 1..self.input_offsets.len() {
                let offset = input_offset + self.input_offsets[i].x + text_offset;
                if offset >= text_box_min_max.0 {
                    return i - 1
                }
            }
            return self.input_offsets.len() - 1
        }
        if rel_cursor_pos > text_box_min_max.1 {
            for (i, &offset) in self.input_offsets.iter().enumerate().rev() {
                let offset = offset.x + text_offset;
                if input_offset + offset < text_box_min_max.1 {
                    return i + 1
                }
            }
            return 0
        }
        if rel_cursor_pos > text_min_max.1 {
            return self.input_offsets.len()
        }
        for i in 1..self.input_offsets.len() {
            let offset = self.input_offsets[i].x + text_offset;
            if text_min_max.0 + offset >= rel_cursor_pos
            {
                return i - 1
            }
        }
        self.input_offsets.len() - 1
    }

    #[inline(always)]
    fn calc_cursor_offset(&self, font_scale: f32, pos: usize) -> Vec2 {
        self.input_offsets
            .get(pos)
            .cloned()
            .unwrap_or_else(|| {
                self.input_text
                    .as_ref()
                    .map(|t| { vec2(t.text_width * font_scale, 0.0) })
                    .unwrap_or_default()
        }) - vec2(self.input_text_offset_x, 0.0)
    }
}

impl<I, FontHash> Widget<I, FontHash> for InputText<I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    fn hover_text(&self) -> Option<&str> {
        None
    }

    #[inline(always)]
    fn set_offset(
        &mut self,
        offset: Vec2,
    )
    {
        self.offset = offset;
    }

    #[inline(always)]
    fn calc_height(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> f32
    {
        style.calc_text_box_height(text_renderer.font_height(&style.font_regular).unwrap_or_default())
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        _style: &Style<FontHash>,
        _window_pos: Vec2,
        _cursor_pos: Vec2
    ) -> bool
    {
        self.held()
    }

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
    ) -> UpdateResult {
        enum CursorMove {
            None,
            Left,
            Right,
            Backspace,
        }
        let mut cursor_move = CursorMove::None;
        let font_scale = style.font_scale;
        let start_width = self.input_text
            .as_ref()
            .map(|v| style.calc_text_width(v.text_width))
            .unwrap_or_default();
        if self.active() {
            let mut cursor_timer = self.cursor_timer + nox.delta_time().as_secs_f32();
            if cursor_timer >= style.input_text_cursor_switch_speed {
                self.toggle_cursor_visible();
                cursor_timer = 0.0;
            }
            self.cursor_timer = cursor_timer;
            if let Some(selection) = self.selection && selection.0 != selection.1 {
                if self.selection_left() {
                    self.text_cursor_pos = selection.0;
                } else {
                    self.text_cursor_pos = selection.1;
                }
                let input = nox.get_input_text();
                self.selection = None;
                if nox.was_key_pressed(KeyCode::Backspace) || !input.is_empty() {
                    let start_count = self.input.char_indices().count();
                    for i in (selection.0..selection.1).rev() {
                        let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                        self.input.remove(index);
                    }
                    let mut text_cursor_pos = selection.0;
                    for text in input {
                        if text.0 != KeyCode::Backspace && text.0 != KeyCode::Enter {
                            self.input.insert_str(
                                self.input
                                    .char_indices()
                                    .skip(text_cursor_pos)
                                    .next()
                                    .map(|(i, _)| i)
                                    .unwrap_or_else(|| self.input.len()),
                                &text.1
                            );
                            text_cursor_pos += text.1.char_indices().count();
                        }
                    }
                    self.text_cursor_pos = text_cursor_pos;
                    if selection.1 == self.text_cursor_pos {
                        self.text_cursor_pos = selection.0 - selection.1;
                    }
                    let end_count = self.input.char_indices().count();
                    if start_count > end_count {
                        cursor_move = CursorMove::Backspace;
                    }
                    self.input_text = None;
                } else if nox.was_key_pressed(KeyCode::ArrowLeft) {
                    self.text_cursor_pos = selection.0;
                } else if nox.was_key_pressed(KeyCode::ArrowRight) {
                    self.text_cursor_pos = selection.1;
                } else {
                    self.selection = Some(selection);
                }
            }
            else {
                let mut text_cursor_pos = self.text_cursor_pos;
                let start_pos = text_cursor_pos;
                let start_count = self.input.char_indices().count();
                if text_cursor_pos != 0 {
                    if nox.was_key_pressed(KeyCode::Backspace) {
                        let remove = text_cursor_pos - 1;
                        let (index, _) = self.input.char_indices().skip(remove).next().unwrap();
                        self.input.remove(index);
                        self.input_text = None;
                        text_cursor_pos = remove;
                    } else if nox.was_key_pressed(KeyCode::ArrowLeft) {
                        text_cursor_pos -= 1;
                    }
                }
                let input = nox.get_input_text();
                if !input.is_empty() {
                    for text in input {
                        if text.0 != KeyCode::Backspace && text.0 != KeyCode::Enter {
                            self.input.insert_str(
                                self.input
                                    .char_indices()
                                    .skip(text_cursor_pos)
                                    .next()
                                    .map(|(i, _)| i)
                                    .unwrap_or_else(|| self.input.len()),
                                &text.1
                            );
                            text_cursor_pos += text.1.char_indices().count();
                        }
                    }
                    self.input_text = None;
                }
                let end_count = self.input.char_indices().count();
                if nox.was_key_pressed(KeyCode::ArrowRight) {
                    text_cursor_pos = (text_cursor_pos + 1).clamp(0, end_count);
                    cursor_move = CursorMove::Right;
                }
                if start_count > end_count {
                    cursor_move = CursorMove::Backspace;
                } else if start_pos < text_cursor_pos {
                    cursor_move = CursorMove::Right;
                } else if start_pos > text_cursor_pos {
                    cursor_move = CursorMove::Left;
                }
                self.text_cursor_pos = text_cursor_pos;
            }
        } else {
            self.flags &= !Self::CURSOR_VISIBLE;
            self.input_text_offset_x = 0.0;
        }
        let title_text = self.title_text.get_or_insert(text_renderer
            .render(
                &[text_segment(&self.title, &style.font_regular)],
                false, 0.0
            )
            .unwrap_or_default()
        );
        self.input_text.get_or_insert_with(|| {
            self.input_offsets.clear();
            text_renderer
                .render_and_collect_offsets(
                    &[text_segment(&self.input, &style.font_regular)],
                    false, 0.0,
                    |_, [x, y]| { self.input_offsets.push(vec2(x, y) * font_scale); }
                )
                .unwrap_or_default()
        });
        let input_text = unsafe {
            self.input_text.as_ref().unwrap_unchecked()
        };
        let title_width = style.calc_text_width(title_text.text_width);
        let text_height = style.calc_text_height(title_text.row_height);
        let item_pad_outer = style.item_pad_outer;
        let item_pad_inner = style.item_pad_inner;
        let offset = self.offset;
        let input_offset = offset + vec2(title_width + item_pad_outer.x, 0.0);
        let input_text_offset = input_offset + item_pad_inner;
        let mut width = title_width +
            item_pad_outer.x + item_pad_outer.x + item_pad_outer.x;
        let min_window_width = width + style.input_text_min_width;
        if window_width < min_window_width {
            width = style.input_text_min_width;
        } else {
            width = (window_width - width).min(style.input_text_max_width);
        }
        let input_rect = rect(
            Default::default(),
            vec2(width, style.calc_text_box_height(title_text.row_height)),
            style.rounding,
        );
        let input_width = style.calc_text_width(input_text.text_width);
        let text_cursor_pos_x = self.input_offsets
            .get(self.text_cursor_pos)
            .map(|v| v.x)
            .unwrap_or_else(|| input_width);
        let input_text_max_x = width - item_pad_inner.x - item_pad_inner.x;
        match cursor_move {
            CursorMove::None => {},
            CursorMove::Left => {
                if text_cursor_pos_x - self.input_text_offset_x < 0.0 {
                    self.input_text_offset_x = text_cursor_pos_x;
                }
            },
            CursorMove::Right => {
                if input_text_max_x - self.calc_cursor_offset(font_scale, self.text_cursor_pos).x < 0.0
                {
                    self.input_text_offset_x = text_cursor_pos_x - input_text_max_x;
                }
            },
            CursorMove::Backspace => {
                let pos = self.calc_cursor_offset(font_scale, self.text_cursor_pos).x;
                let delta = start_width - input_width;
                if input_text_max_x - pos > 0.0 &&
                    input_width - text_cursor_pos_x <
                    input_text_max_x - pos
                {
                    self.input_text_offset_x = (self.input_text_offset_x - delta).clamp(0.0, f32::INFINITY);
                }
            }
        }
        self.flags &= !Self::HOVERING;
        let mut cursor_in_widget = false;
        let override_cursor = style.override_cursor;
        let mouse_released = nox.was_mouse_button_released(MouseButton::Left);
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = style.cursor_error_margin;
        if !other_widget_active {
            self.flags |= Self::HOVERING *
                BoundingRect::from_position_size(
                    input_offset - vec2(error_margin, 0.0),
                    input_rect.max + vec2(error_margin + error_margin, 0.0)
                ).is_point_inside(rel_cursor_pos) as u32;
            let input_min = input_offset.x + item_pad_inner.x;
            let input_min_max = (input_min, input_min + input_width);
            let input_box_min_max = (input_offset.x, input_offset.x + input_rect.max.x - item_pad_inner.x);
            let text_offset = -self.input_text_offset_x;
            let enter_pressed = nox.was_key_pressed(KeyCode::Enter);
            if self.hovering() && !enter_pressed {
                cursor_in_widget = true;
                if mouse_pressed {
                    self.text_cursor_pos =
                        self.calc_cursor_index(
                            rel_cursor_pos.x,
                            input_min_max,
                            input_box_min_max,
                            text_offset,
                            input_min,
                        );
                    self.flags |= Self::HELD;
                }
                if override_cursor {
                    nox.set_cursor(CursorIcon::Text);
                }
            } else if enter_pressed || (!window_moving && mouse_released && !self.held()) {
                self.flags &= !Self::ACTIVE;
            }
            if mouse_pressed {
                self.selection = None;
                self.flags &= !Self::SELECTION_LEFT;
            }
            if self.held() {
                if mouse_released {
                    self.flags &= !Self::HELD;
                    if let Some(selection) = self.selection && selection.0 == selection.1 {
                        self.selection = None;
                    }
                } else {
                    self.flags |= Self::ACTIVE;
                    self.flags |= Self::CURSOR_VISIBLE;
                    let text_cursor_pos = self.calc_cursor_index(
                        rel_cursor_pos.x,
                        input_min_max,
                        input_box_min_max,
                        text_offset,
                        input_min,
                    );
                    let selection_left = self.selection_left();
                    if let Some(mut selection) = self.selection {
                        if selection_left || text_cursor_pos < selection.0 {
                            selection.0 = text_cursor_pos;
                            self.flags |= Self::SELECTION_LEFT;
                            let offset = self.calc_cursor_offset(font_scale, selection.0).x;
                            if offset < 0.0 {
                                self.input_text_offset_x -=
                                    style.input_text_scroll_speed *
                                    nox.delta_time_secs_f32();
                            } else if offset > input_text_max_x {
                                self.input_text_offset_x +=
                                    style.input_text_scroll_speed *
                                    nox.delta_time_secs_f32();
                            }
                            if selection.1 < selection.0 {
                                let tmp = selection.0;
                                selection.0 = selection.1;
                                selection.1 = tmp;
                                self.flags &= !Self::SELECTION_LEFT;
                            }
                        } else {
                            selection.1 = text_cursor_pos;
                            let offset = self.calc_cursor_offset(font_scale, selection.1).x;
                            if offset > input_text_max_x {
                                self.input_text_offset_x +=
                                    style.input_text_scroll_speed *
                                    nox.delta_time_secs_f32();
                            } else if offset < 0.0 {
                                self.input_text_offset_x -=
                                    style.input_text_scroll_speed *
                                    nox.delta_time_secs_f32();
                            }
                            if selection.1 < selection.0 {
                                let tmp = selection.0;
                                selection.0 = selection.1;
                                selection.1 = tmp;
                                self.flags |= Self::SELECTION_LEFT;
                            }
                        }
                        self.selection = Some(selection);
                        self.text_cursor_pos = if self.selection_left() {
                            selection.0
                        } else {
                            selection.1
                        };
                    } else {
                        self.selection = Some((text_cursor_pos, text_cursor_pos));
                        self.text_cursor_pos = text_cursor_pos;
                    }
                    cursor_in_widget = true;
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Text);
                    }
                }
            }
        } else {
            self.flags &= !Self::ACTIVE;
        }
        if self.activated() {
            self.flags &= !Self::ACTIVATED;
            self.flags |= Self::ACTIVE;
            self.flags |= Self::CURSOR_VISIBLE;
            self.cursor_timer = 0.0;
        }
        let cursor_rect_max = vec2(style.input_text_cursor_width, text_height);
        let requires_triangulation =
            self.input_rect != input_rect ||
            self.cursor_rect.max != cursor_rect_max;
        self.input_rect = input_rect;
        self.cursor_rect.max = cursor_rect_max;
        if let Some(selection) = self.selection {
            let left = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.0).x.clamp(0.0, input_text_max_x);
            let right = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.1).x.clamp(0.0, input_text_max_x);
            let rect = BoundingRect::from_min_max(
                vec2(left, offset.y + item_pad_inner.y),
                vec2(right, offset.y + input_rect.max.y - item_pad_inner.y),
            );
            let col = style.input_text_selection_col;
            self.selection_rect_vertices[0] = Vertex {
                pos: rect.min,
                offset: Default::default(),
                color: col,
            };
            self.selection_rect_vertices[1] = Vertex {
                pos: vec2(rect.min.x, rect.max.y),
                offset: Default::default(),
                color: col,
            };
            self.selection_rect_vertices[2] = Vertex {
                pos: rect.max,
                offset: Default::default(),
                color: col,
            };
            self.selection_rect_vertices[3] = Vertex {
                pos: vec2(rect.max.x, rect.min.y),
                offset: Default::default(),
                color: col,
            };
        } else {
            self.flags &= !Self::SELECTION_LEFT;
        }
        UpdateResult {
            min_widget_width: style.input_text_min_width,
            requires_triangulation,
            cursor_in_widget,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut mem::vec_types::GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    ) {
        self.input_rect.to_points(&mut |p| { points.push(p.into()); });
        self.input_rect_vertex_range = tri(points);
        points.clear();
        self.cursor_rect.to_points(&mut |p| { points.push(p.into()); });
        self.cursor_rect_vertex_range = tri(points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style<FontHash>,
        vertices: &mut [shaders::Vertex],
    )
    {
        let title_width = style.calc_text_width(
            self.title_text.as_ref().unwrap().text_width
        );
        let mut offset = self.offset + vec2(title_width + style.item_pad_outer.x, 0.0);
        let mut target_color = style.input_text_bg_col;
        set_vertex_params(vertices, self.input_rect_vertex_range, offset, target_color);
        if self.cursor_visible() {
            offset += style.item_pad_inner + self.calc_cursor_offset(style.font_scale, self.text_cursor_pos);
            target_color = style.text_col;
            set_vertex_params(vertices, self.cursor_rect_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.cursor_rect_vertex_range);
        }
    }

    fn hide(
        &self,
        vertices: &mut [shaders::Vertex],
    )
    {
        hide_vertices(vertices, self.input_rect_vertex_range);
    }

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
    ) -> Result<Option<&dyn OnTopContents<I, FontHash>>, Error>
    {
        if let Some(selection) = self.selection && selection.0 != selection.1 {
            let vert_mem = unsafe {
                vertex_buffer.allocate(render_commands, 4)?
            };
            let idx_mem = unsafe {
                index_buffer.allocate(render_commands, 6)?
            };
            unsafe {
                self.selection_rect_vertices
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), 4);
                Self::SELECTION_INDICES
                    .as_ptr()
                    .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), 6);
            }
            render_commands.bind_pipeline(base_pipeline_id)?;
            let pc_vertex = push_constants_vertex(window_pos, vec2(1.0, 1.0), inv_aspect_ratio);
            render_commands.push_constants(|_| unsafe {
                pc_vertex.as_bytes()
            })?;
            render_commands.draw_indexed(
                DrawInfo {
                    index_count: 6,
                    ..Default::default()
                },
                [
                    DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset)
                ],
                    DrawBufferInfo::new(index_buffer.id(), idx_mem.offset)
            )?;
        }
        let item_pad_outer = style.item_pad_outer;
        let item_pad_inner = style.item_pad_inner;
        let font_scale = vec2(style.font_scale, style.font_scale);
        let text_col = style.text_col;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.offset + vec2(0.0, item_pad_inner.y),
            font_scale, inv_aspect_ratio
        );
        let pc_fragment = text_push_constants_fragment(text_col);
        render_text(render_commands, &self.title_text.as_ref().unwrap(),
            pc_vertex, pc_fragment, vertex_buffer, index_buffer
        )?;
        let title_width = style.calc_text_width(
            self.title_text.as_ref().unwrap().text_width
        );
        let pos = window_pos + self.offset + vec2(title_width + item_pad_outer.x, 0.0);
        render_commands.bind_pipeline(get_custom_pipeline(INPUT_TEXT_PIPELINE_HASH).unwrap())?;
        let pc_vertex = push_constants_vertex(
            pos + item_pad_inner - vec2(self.input_text_offset_x, 0.0),
            font_scale, inv_aspect_ratio
        );
        let pc_fragment = input_text_push_constants_fragment(
            text_col,
            BoundingRect::from_position_size(
                pos + vec2(item_pad_inner.x, 0.0),
                self.input_rect.max - vec2(item_pad_inner.x + item_pad_inner.x, 0.0),
            )
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        let vertex_buffer_id = vertex_buffer.id();
        let index_buffer_id = index_buffer.id();
        for text in &self.input_text.as_ref().unwrap().text {
            let vert_mem = unsafe {
                vertex_buffer.allocate(render_commands, text.trigs.vertices.len())?
            };
            let vert_off_mem = unsafe {
                vertex_buffer.allocate(render_commands, text.offsets.len())?
            };
            let idx_mem = unsafe {
                index_buffer.allocate(render_commands, text.trigs.indices.len())?
            }; 
            unsafe {
                text.trigs.vertices
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), text.trigs.vertices.len());
                text.offsets
                    .as_ptr()
                    .copy_to_nonoverlapping(vert_off_mem.ptr.as_ptr(), text.offsets.len());
                text.trigs.indices
                    .as_ptr()
                    .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), text.trigs.indices.len());
            }
            render_commands.draw_indexed(
                DrawInfo {
                    index_count: text.trigs.indices.len() as u32,
                    instance_count: text.offsets.len() as u32,
                    ..Default::default()
                },
                [
                    DrawBufferInfo::new(vertex_buffer_id, vert_mem.offset),
                    DrawBufferInfo::new(vertex_buffer_id, vert_off_mem.offset),
                ],
                DrawBufferInfo::new(index_buffer_id, idx_mem.offset),
            )?;
        }
        Ok(None)
    }
}
