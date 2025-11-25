use std::rc::Rc;

use core::{
    cell::{RefCell, RefMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use nox_font::{text_segment_owned, RenderedText, TextOffset, TextSegmentOwned};

use nox_geom::{
    shapes::*,
    *
};

use nox::{
    alloc::arena_alloc::ArenaGuard,
    mem::{
        vec_types::{GlobalVec, Vector},
    },
    *
};

use crate::*;

#[derive(Default, Clone, Debug)]
pub struct RowOffsets {
    pub offsets: GlobalVec<TextOffset>,
    pub row_height: f32,
    pub max_x: f32,
    pub min_x: f32,
}

impl RowOffsets {

    pub fn new() -> Self {
        Self {
            offsets: Default::default(),
            row_height: 0.0,
            max_x: 0.0,
            min_x: 0.0,
        }
    }
}

#[derive(Default, Clone)]
pub struct Text {
    pub text: RenderedText,
    pub reset_offsets: GlobalVec<GlobalVec<font::VertexOffset>>,
    pub rows: GlobalVec<RowOffsets>,
    pub color: ColorSRGBA,
    pub offset: Vec2,
    pub scale: Vec2,
    pub selectable_index: Option<usize>,
    pub row_offset: u32,
    pub row_count: u32,
    pub bounds: BoundingRect,
    pub tool_tip: Option<Rc<CompactString>>,
}

impl Text {

    pub fn new(
        text: RenderedText,
        rows: GlobalVec<RowOffsets>,
        color: ColorSRGBA,
        offset: Vec2,
        scale: Vec2,
        selectable_index: Option<usize>,
        row_offset: u32,
        row_count: u32,
        bounds: Option<BoundingRect>,
        tool_tip: Option<Rc<CompactString>>,
    ) -> Self
    {
        let mut reset_offsets = GlobalVec::default();
        for (_, instance) in &text.text {
            let inst = reset_offsets.push(GlobalVec::default());
            for &offset in &instance.offsets {
                inst.push(offset);
            }
        }
        Self {
            text,
            reset_offsets,
            rows,
            color,
            offset,
            scale,
            selectable_index,
            row_offset,
            row_count,
            bounds: bounds.unwrap_or(BoundingRect::from_min_max(Vec2::MIN, Vec2::MAX)),
            tool_tip
        }
    }

    pub fn reset(&mut self) {
        for (i, (_, instance)) in self.text.text.iter_mut().enumerate() {
            instance.offsets.clone_from_slice(&self.reset_offsets[i]);
        }
    }
}

#[derive(Default, Clone)]
pub struct SharedText(pub Rc<RefCell<Text>>);

pub struct SharedTextMut<'a>(pub RefMut<'a, Text>);

pub struct SelectableTextData {}

impl SelectableTextData {

    #[inline(always)]
    pub fn as_text(&self) -> &[Text] {
        todo!()
    }

    #[inline(always)]
    pub fn as_text_mut(&mut self) -> &mut [Text] {
        todo!()
    }
}

pub struct SelectableText<Style>
{
    text: GlobalVec<Text>,
    selection: Option<(usize, usize)>,
    text_segment_builders: GlobalVec<TextSegmentBuilder>,
    tool_tip: Option<Rc<CompactString>>,
    selection_vertices: GlobalVec<Vertex>,
    selection_indices: GlobalVec<u32>,
    selection_rects: GlobalVec<Rect>,
    prev_selection: Option<(usize, usize)>,
    base_offset: Vec2,
    start_offset: Vec2,
    scroll_offset: Vec2,
    offset: Vec2,
    max_width: f32,
    char_count: u32,
    prev_char_count: u32,
    current_height: f32,
    current_row: u32,
    next_builder: u32,
    flags: u32,
    _marker: PhantomData<Style>,
}

impl<Style> SelectableText<Style>
    where 
        Style: UiStyle,
{

    const TRUNC_TO_WINDOW_WIDTH: u32 = 0x1;
    const HELD: u32 = 0x2;
    const HOVERED: u32 = 0x4;
    const SELECTION_LEFT: u32 = 0x8;

    #[inline(always)]
    pub fn new() -> Self
    {
        Self {
            text: Default::default(),
            selection: None,
            text_segment_builders: Default::default(),
            tool_tip: None,
            selection_vertices: Default::default(),
            selection_indices: Default::default(),
            selection_rects: Default::default(),
            prev_selection: None,
            base_offset: Default::default(),
            start_offset: Default::default(),
            offset: Default::default(),
            scroll_offset: vec2(0.0, 0.0),
            max_width: 0.0,
            char_count: 0,
            prev_char_count: 0,
            current_height: Default::default(),
            current_row: 0,
            next_builder: 0,
            flags: 0,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn set_trunc_to_window_width(&mut self, value: bool) {
        self.flags &= !Self::TRUNC_TO_WINDOW_WIDTH;
        or_flag!(self.flags, Self::TRUNC_TO_WINDOW_WIDTH, value);
    }

    #[inline(always)]
    pub fn current_offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    pub fn current_height(&self) -> f32 {
        self.current_height
    }

    #[inline(always)]
    pub fn as_text(&self) -> &[Text] {
        &self.text
    }

    #[inline(always)]
    pub fn as_text_mut(&mut self) -> &mut [Text] {
        &mut self.text
    }

    #[inline(always)]
    pub fn as_builder<'a, 'b>(
        &'a mut self,
        window_width: f32,
        style: &'a Style,
        text_renderer: &'a mut TextRenderer<'b>,
    ) -> SelectableTextBuilder<'a, 'b, Style>
    {
        SelectableTextBuilder::new(style, text_renderer, self, window_width)
    }

    #[inline(always)]
    pub fn set_base_and_start_offset(
        &mut self,
        base_offset: Vec2,
        start_offset: Vec2,
    ) {
        self.base_offset = base_offset;
        self.start_offset = start_offset;
        self.offset = start_offset;
    }

    #[inline(always)]
    pub fn set_current_height(&mut self, height: f32) {
        self.current_height = height;
    }

    #[inline(always)]
    pub fn get_selection(&self) -> CompactString {
        let mut result = CompactString::default();
        let mut i_off = 0;
        let selection = self.selection.unwrap_or_default();
        if selection.0 == selection.1 {
            return Default::default()
        }
        for text in &self.text {
            for row in &text.rows {
                for offset in &row.offsets {
                    if i_off >= selection.0 && i_off < selection.1 {
                        result.push(offset.char);
                    }
                    if i_off == selection.1 {
                        return result
                    }
                    i_off += 1;
                }
            }
        }
        result
    }

    #[inline(always)]
    fn trunc_to_window_width(&self) -> bool {
        self.flags & Self::TRUNC_TO_WINDOW_WIDTH == Self::TRUNC_TO_WINDOW_WIDTH
    }

    #[inline(always)]
    fn held(&self) -> bool {
        self.flags & Self::HELD == Self::HELD
    }

    #[inline(always)]
    fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
    }

    #[inline(always)]
    fn selection_left(&self) -> bool {
        self.flags & Self::SELECTION_LEFT == Self::SELECTION_LEFT
    }

    #[inline(always)]
    fn calc_cursor_index(
        &self,
        style: &Style,
        rel_cursor_pos: Vec2,
        text_min: Vec2,
    ) -> Option<usize>
    {
        if self.text.is_empty() { return None }
        let font_scale = style.font_scale();
        let mut k_off = 0;
        let mut cursor_in_row = None;
        let mut first_segment = true;
        let mut first_row = true;
        let last_row = self.current_row;
        for (i, text) in self.text.iter().enumerate() {
            let last_segment = i == self.text.len() - 1;
            let rows = &text.rows;
            for (j, RowOffsets { offsets, row_height, max_x, min_x }) in rows.iter().enumerate() {
                let row = j as u32 + text.row_offset;
                if row != 0 {
                    first_row = false;
                }
                let last_row = row == last_row;
                let row_height = row_height * font_scale;
                let max_x = text_min.x + max_x * font_scale;
                let min_x = text_min.x + min_x * font_scale;
                let cur_k_off = k_off;
                let n = offsets.len();
                for k in 0..n {
                    let text_offset = offsets[k];
                    let offset = vec2(
                        text_offset.offset[0] * font_scale,
                        text_offset.offset[1] * font_scale
                    );
                    let min_y = text_min.y + offset.y;
                    let max_y = min_y + row_height;
                    if (rel_cursor_pos.y >= min_y || first_row) &&
                        (rel_cursor_pos.y <= max_y || last_row)
                    {
                        cursor_in_row = Some(text.row_offset + j as u32);
                        if last_row && last_segment && rel_cursor_pos.x >= max_x {
                            return Some(cur_k_off + offsets.len());
                        }
                        if first_segment && first_row && rel_cursor_pos.x <= min_x {
                            return Some(cur_k_off)
                        }
                    }
                    if cursor_in_row.is_some() && text_min.x + offset.x >= rel_cursor_pos.x {
                        return Some(cur_k_off + k - 1)
                    }
                    k_off += 1;
                }
            }
            first_segment = false;
        }
        if cursor_in_row.is_some() {
            return Some(k_off - 1)
        }
        None
    }

    #[inline(always)]
    fn get_tooltip(&self, cursor_index: usize) -> Option<Rc<CompactString>> {
        let mut j_off = 0;
        for text in &self.text {
            let rows = &text.rows;
            for RowOffsets { offsets, row_height: _, max_x: _, min_x: _ } in rows {
                if j_off + offsets.len() > cursor_index {
                    return text.tool_tip.clone();
                }
                j_off += offsets.len();
            }
        }
        None
    }

    #[inline(always)]
    fn paint_selection(&mut self, style: &Style) {
        if self.prev_selection == self.selection {
            return
        }
        self.prev_selection = self.selection;
        self.selection_vertices.clear();
        self.selection_indices.clear();
        self.selection_rects.clear();
        let font_scale = style.font_scale();
        if let Some(mut selection) = self.selection {
            let mut j_off = 0;
            let mut cur_row: Option<usize> = None;
            let mut cur_rect = Default::default();
            for text in &self.text {
                let rows = &text.rows;
                if j_off >= selection.1 {
                    break
                }
                for (i, RowOffsets { offsets, row_height: _, max_x: _, min_x: _ })
                    in rows
                        .iter()
                        .enumerate()
                {
                    let row = i + text.row_offset as usize;
                    if j_off >= selection.1 {
                        break
                    }
                    if let Some(r) = cur_row && r != row {
                        self.selection_rects.push(cur_rect);
                        cur_rect = Default::default();
                        cur_row = None;
                        selection.0 = j_off;
                    }
                    for offset in offsets.iter() {
                        if j_off >= selection.1 {
                            break
                        }
                        if j_off == selection.0 {
                            cur_row = Some(row);
                            let mut off: Vec2 = offset.offset.into();
                            off *= font_scale;
                            let row_height = offset.row_height * font_scale;
                            cur_rect = rect(off, off + vec2(offset.x_advance * font_scale, row_height), 0.0);
                        } else {
                            cur_rect.max.x = (offset.offset[0] + offset.x_advance) * font_scale;
                        }
                        j_off += 1;
                    }
                }
            }
            if cur_rect != Default::default() {
                self.selection_rects.push(cur_rect);
            }
        }
        let mut points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        for rect in &mut self.selection_rects {
            rect.to_points(&mut |p| { points.push(p.into()); });
            earcut
                ::earcut(&points, &[], false, &mut self.selection_vertices, &mut indices_usize)
                .unwrap();
            points.clear();
        }
        self.selection_indices.append_map(&indices_usize, |&i| i as u32);
    }
}

pub struct SelectableTextBuilder<'a, 'b, Style>
    where 
        Style: UiStyle,
{
    style: &'a Style,
    text_renderer: &'a mut TextRenderer<'b>,
    text: &'a mut SelectableText<Style>,
    window_width: f32,
    color: ColorSRGBA,
    scale: Vec2,
}

impl<'a, 'b, Style> SelectableTextBuilder<'a, 'b, Style>
    where 
        Style: UiStyle,
{

    #[inline(always)]
    fn new(
        style: &'a Style,
        text_renderer: &'a mut TextRenderer<'b>,
        text: &'a mut SelectableText<Style>,
        window_width: f32,
    ) -> Self {
        Self {
            style,
            text_renderer,
            text,
            window_width,
            color: style.inactive_text_col(),
            scale: vec2(1.0, 1.0),
        }
    }

    #[inline(always)]
    pub fn color(&mut self, color: impl Color) -> &mut Self {
        self.color = color.to_srgba();
        self
    }

    #[inline(always)]
    pub fn default_color(&mut self) -> &mut Self {
        self.color = self.style.inactive_text_col();
        self
    }

    #[inline(always)]
    pub fn scale(&mut self, scale: Vec2) -> &mut Self {
        self.scale = scale;
        self
    }

    #[inline(always)]
    pub fn with_text(
        &mut self,
        tool_tip: Option<&str>,
        mut f: impl FnMut(&mut TextSegmentBuilder)
    ) -> &mut Self
    {
        let trunc_to_window_width = self.text.trunc_to_window_width();
        let builder =
            if self.text.next_builder >= self.text.text_segment_builders.len() as u32 {
                self.text.text_segment_builders.push(TextSegmentBuilder {
                    segments: GlobalVec::new(),
                    default_font: self.style.font_regular().clone(),
                })
            } else {
                let builder = &mut self.text.text_segment_builders[self.text.next_builder as usize];
                builder.segments.clear();
                builder
            };
        self.text.next_builder += 1;
        f(builder);
        let segments = builder.as_segments();
        let window_width = self.window_width;
        let font_scale = self.style.font_scale();
        let item_pad_outer = self.style.item_pad_outer();
        let base_to_end = window_width - item_pad_outer.x - self.text.base_offset.x;
        let offset = base_to_end - (window_width - item_pad_outer.x - self.text.offset.x);
        let mut rows: GlobalVec<RowOffsets> = GlobalVec::new();
        let mut current_row = self.text.current_row;
        let mut row_count = 0;
        let offset_y_scaled = (self.text.offset.y - self.text.base_offset.y) / font_scale;
        let mut first_word_truncated = false;
        if let Some(mut text) = self.text_renderer.render_and_collect_offsets(
            segments,
            false,
            if trunc_to_window_width {
                base_to_end / font_scale
            } else {
                0.0
            },
            offset / font_scale,
            |mut offset| {
                let row = offset.row;
                if row >= rows.len() as u32 {
                    rows.resize(row as usize + if !first_word_truncated { 1 } else { 0 },
                        RowOffsets {
                            offsets: GlobalVec::new(),
                            row_height:
                                if row == 0 {
                                    row_count += 1;
                                    offset.row_height
                                } else {
                                    if offset.first_word {
                                        first_word_truncated = true;
                                    }
                                    row_count += 1;
                                    current_row += 1;
                                    offset.row_height
                                },
                            max_x: 0.0,
                            min_x: 0.0,
                        }
                    );
                }
                if !offset.first_word {
                    first_word_truncated = false;
                }
                offset.offset[1] += offset_y_scaled;
                rows[row as usize].offsets.push(offset);
                self.text.char_count += 1;
            }
        ) {
            for (_, instanced) in &mut text.text {
                for instance in &mut instanced.offsets {
                    instance.offset[1] += offset_y_scaled;
                }
            }
            let font_height = text.row_height * font_scale;
            self.text.current_height = self.text.current_height.max(font_height);
            if text.text_rows > 1 {
                self.text.offset.y +=
                    self.text.current_height + (text.text_rows - 2) as f32 * font_height;
                self.text.current_height =
                    self.text.current_height.max(font_height);
            }
            for row in &mut rows {
                let (offset, x_advance) = row.offsets
                    .last()
                    .map(|v| (v.offset, v.x_advance))
                    .unwrap_or_default();
                row.max_x = offset[0] + x_advance;
                self.text.max_width = self.text.max_width.max(row.max_x * font_scale);
                row.min_x = row.offsets.first().map(|v| v.offset).unwrap_or_default()[0];
            }
            self.text.offset.x = rows.last().map(|v| self.text.base_offset.x +  v.max_x * font_scale).unwrap_or_default();
            self.text.text.push(Text::new(
                text, rows,
                self.color,
                self.text.base_offset,
                self.scale,
                Some(self.text.text.len()),
                self.text.current_row,
                row_count,
                None,
                tool_tip.map(|v| Rc::new(CompactString::new(v))),
            ));
        }
        self.text.current_row = current_row;
        self
    }
}

pub struct TextSegmentBuilder
{
    segments: GlobalVec<TextSegmentOwned<CompactString>>,
    default_font: CompactString,
}

impl TextSegmentBuilder
{

    #[inline(always)]
    fn as_segments(&self) -> &[TextSegmentOwned<CompactString>] {
        &self.segments
    }

    #[inline(always)]
    pub fn with_segment(
        &mut self,
        text: &str,
        font: Option<&str>,
    ) -> &mut Self {
        self.segments.push(text_segment_owned(text, font.map(|v| v.into()).unwrap_or(self.default_font.clone())));
        self
    }
}

impl<Style> Widget<Style> for SelectableText<Style>
    where 
        Style: UiStyle,
{

    #[inline(always)]
    fn get_offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2) {
        self.offset = offset;
    }

    #[inline(always)]
    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.scroll_offset = offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        _style: &Style,
        _text_renderer: &mut TextRenderer,
    ) -> Vec2 {
        Default::default()
    }

    fn status<'a>(
        &'a self,
        _ctx: &WindowCtx,
        _style: &Style,
        _window_pos: Vec2,
        _cursor_pos: Vec2,
    ) -> WidgetStatus<'a>
    {
        if self.held() {
            WidgetStatus::Active
        } else if self.hovered() {
            WidgetStatus::Hovered(self.tool_tip.as_ref().map(|v| v.as_str()))
        } else {
            WidgetStatus::Inactive
        }
    }

    fn update(
        &mut self,
        ctx: &mut WindowCtx,
        style: &Style,
        _text_renderer: &mut TextRenderer,
        _window_size: Vec2,
        window_pos: Vec2,
        _content_offset: Vec2,
        cursor_pos: Vec2,
        _delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        cursor_in_other_widget: bool,
        _window_moving: bool,
        hover_blocked: bool,
        _collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult
    {
        let mouse_left_state = ctx.mouse_button_state(MouseButton::Left);
        if mouse_left_state.pressed() || self.prev_char_count != self.char_count {
            self.selection = None;
        }
        let rel_cursor_pos = cursor_pos - window_pos - self.scroll_offset;
        if mouse_left_state.released() {
            self.flags &= !Self::HELD;
        }
        let mut cursor_index = self.calc_cursor_index(style, rel_cursor_pos, self.base_offset);
        let error_margin = style.cursor_error_margin();
        let cursor_in_text  =
            if let Some(cursor_index) = cursor_index {
                if rel_cursor_pos.y <= self.start_offset.y ||
                    rel_cursor_pos.y >= self.offset.y + self.current_height {
                    false
                }
                else if cursor_index == 0 {
                    rel_cursor_pos.x + error_margin >= self.start_offset.x
                } else if cursor_index as u32 == self.char_count {
                    rel_cursor_pos.x - error_margin <= self.offset.x
                } else {
                    true
                }
            } else {
                false
            };
        self.flags &= !Self::HOVERED;
        let mut cursor_in_widget = false;
        if cursor_in_this_window &&
            !other_widget_active &&
            !cursor_in_other_widget &&
            !hover_blocked
        {
            if cursor_in_text {
                if style.override_cursor() {
                    ctx.set_cursor(CursorIcon::Text);
                }
                if mouse_left_state.pressed() {
                    if let Some(index) = cursor_index {
                        self.selection = Some((index, index));
                        self.flags |= Self::HELD;
                    }
                }
            }
            if let Some(index) = cursor_index && cursor_in_text && !self.held() {
                self.tool_tip = self.get_tooltip(index);
                cursor_in_widget = true;
                self.flags |= Self::HOVERED;
            }
        }
        if let Some(mut selection) = self.selection {
            if !self.held() {
                cursor_index = None;
            }
            if ctx.key_state(KeyCode::ArrowLeft).pressed() {
                if ctx.key_state(KeyCode::ShiftLeft).held() {
                    if self.selection_left() {
                        if selection.0 != 0 {
                            selection.0 -= 1;
                        }
                        cursor_index = Some(selection.0);
                    } else {
                        if selection.0 != selection.1 {
                            selection.1 -= 1;
                        } else if selection.0 != 0 {
                            self.flags |= Self::SELECTION_LEFT;
                            selection.0 -= 1;
                        }
                        cursor_index = Some(selection.1);
                    }
                }
            } else if ctx.key_state(KeyCode::ArrowRight).pressed() {
                if ctx.key_state(KeyCode::ShiftLeft).held() {
                    if self.selection_left() {
                        selection.0 += 1;
                        cursor_index = Some(selection.0);
                    } else if selection.1 != self.char_count as usize {
                        selection.1 += 1;
                        cursor_index = Some(selection.1);
                    }
                }
            }
            if selection.0 > selection.1 {
                self.flags ^= Self::SELECTION_LEFT;
                let tmp = selection.0;
                selection.0 = selection.1;
                selection.1 = tmp;
            }
            if let Some(index) = cursor_index {
                let selection_left = self.selection_left();
                let prev_selection = selection;
                if selection_left || index < selection.0 {
                    selection.0 = index;
                    self.flags |= Self::SELECTION_LEFT;
                    if selection.1 < selection.0 {
                        let tmp = selection.0;
                        selection.0 = selection.1;
                        selection.1 = tmp;
                        self.flags &= !Self::SELECTION_LEFT;
                    }
                } else {
                    selection.1 = index;
                    if selection.1 < selection.0 {
                        let tmp = selection.0;
                        selection.0 = selection.1;
                        selection.1 = tmp;
                        self.flags |= Self::SELECTION_LEFT;
                    }
                }
                if !selection_left && self.selection_left() {
                    selection.1 = prev_selection.0;
                }
            }
            self.selection = Some(selection);
            if ctx.key_state(KeyCode::ControlLeft).held() && ctx.key_state(KeyCode::KeyC).pressed() {
                let input = self.get_selection();
                if !input.is_empty() {
                    ctx.set_clipboard(&input);
                }
                self.selection = None;
            }
        } else {
            self.flags &= !Self::SELECTION_LEFT;
        }
        if self.held() && style.override_cursor() {
            ctx.set_cursor(CursorIcon::Text);
        }
        self.prev_char_count = self.char_count;
        self.char_count = 0;
        self.paint_selection(style);
        for builder in &mut self.text_segment_builders {
            builder.segments.clear();
        }
        self.next_builder = 0;
        self.text.clear();
        self.current_height = 0.0;
        self.max_width = 0.0;
        self.current_row = 0;
        UpdateResult { requires_triangulation: false, requires_transfer_commands: false, cursor_in_widget }
    }

    fn triangulate(
        &mut self,
        _points: &mut GlobalVec<[f32; 2]>,
        _helper_points: &mut GlobalVec<[f32; 2]>,
        _tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {}

    fn set_vertex_params(
        &mut self,
        style: &Style,
        _vertices: &mut [Vertex],
    ) {
        if self.selection_vertices.is_empty() {
            return
        }
        let vertex_sample = self.selection_vertices[0];
        let offset = self.scroll_offset;
        if vertex_sample.offset != offset || vertex_sample.color != style.input_text_selection_bg_col() {
            let target_color = style.input_text_selection_bg_col();
            for vertex in &mut self.selection_vertices {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
    }

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        _style: &Style,
        _sampler: SamplerId,
        base_pipeline: GraphicsPipelineId,
        _text_pipeline: GraphicsPipelineId,
        _texture_pipeline: GraphicsPipelineId,
        _texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _tmp_alloc: &ArenaGuard,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<Style>>, Error> {
        if self.selection_vertices.is_empty() {
            return Ok(None)
        }
        let vertex_count = self.selection_vertices.len();
        let index_count = self.selection_indices.len();
        let vert_buf_id = vertex_buffer.id();
        let idx_buf_id = index_buffer.id();
        let vert_mem = unsafe { vertex_buffer
            .allocate(render_commands, vertex_count)?
        };
        let idx_mem = unsafe { index_buffer
            .allocate(render_commands, index_count)?
        };
        unsafe {
            self.selection_vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vertex_count);
            self.selection_indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), index_count);
        }
        render_commands.bind_pipeline(base_pipeline)?;
        let pc_vertex = push_constants_vertex(
            window_pos + self.base_offset,
            vec2(1.0, 1.0), inv_aspect_ratio, unit_scale
        );
        let pc_fragment = base_push_constants_fragment(
            content_area.min,
            content_area.max,
        );
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: index_count as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vert_buf_id, vert_mem.offset),
            ],
            DrawBufferInfo::new(idx_buf_id, idx_mem.offset)
        )?;
        Ok(None)
    }

    fn hide(
        &mut self,
        _vertices: &mut [Vertex],
        _window_semaphore: (TimelineSemaphoreId, u64),
        _global_resources: &mut GlobalResources,
        _tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> { Ok(()) }
}

impl SharedText {

    pub fn new(text: Text) -> Self {
        Self(Rc::new(RefCell::new(text)))
    }

    pub fn as_mut(&self) -> SharedTextMut<'_> {
        SharedTextMut(self.0.borrow_mut())
    }

    pub fn edit(self, mut f: impl FnMut(&mut SharedTextMut)) -> Self {
        f(&mut self.as_mut());
        self
    }
}

impl<'a> Deref for SharedTextMut<'a> {

    type Target = Text;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for SharedTextMut<'a> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
