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

use nox_font::{text_segment, RenderedText, VertexTextRenderer};

use crate::*;

pub struct InputText<TitleText, I, FontHash, Style, HoverStyle> {
    offset: Vec2,
    title: TitleText,
    input: CompactString,
    input_text: Option<RenderedText>,
    input_text_formatted: Option<RenderedText>,
    empty_input_prompt: CompactString,
    format_input: Box<fn(&mut dyn Write, &str) -> core::fmt::Result>,
    input_offsets: GlobalVec<Vec2>,
    input_text_offset_x: f32,
    selection: Option<(usize, usize)>,
    cursor_rect_vertex_range: VertexRange,
    text_cursor_pos: usize,
    input_rect: Rect,
    input_rect_vertex_range: VertexRange,
    input_rect_outline_vertex_range: VertexRange,
    selection_rect_vertices: [Vertex; 4],
    cursor_rect: Rect,
    flags: u32,
    cursor_timer: f32,
    double_click_timer: f32,
    outline_width: f32,
    width_override: f32,
    bg_col_override: ColorSRGBA,
    _marker: PhantomData<(I, FontHash, Style, HoverStyle)>,
}

impl<TitleText, I, FontHash, Style, HoverStyle> InputText<TitleText, I, FontHash, Style, HoverStyle>
    where
        TitleText: Text,
        Style: WindowStyle<FontHash>,
{

    const HOVERING: u32 = 0x1;
    const HELD: u32 = 0x2;
    const ACTIVE: u32 = 0x4;
    const CURSOR_VISIBLE: u32 = 0x8;
    const SELECTION_LEFT: u32 = 0x10;
    const MOUSE_VISIBLE: u32 = 0x20;
    const FORMAT_ERROR: u32 = 0x40;
    const SKIP_TITLE: u32 = 0x80;
    const CENTER_TEXT: u32 = 0x100;
    const CURSOR_ENABLE: u32 = 0x200;
    const SELECT_ALL: u32 = 0x400;
    const WIDTH_OVERRIDE: u32 = 0x800;
    const BG_COL_OVERRIDE: u32 = 0x1000;
    const CLICKED_LAST_FRAME: u32 = 0x2000;
    const SELECT_ALL_LAST_FRAME: u32 = 0x4000;
    const ACTIVATED_LAST_FRAME: u32 = 0x8000;

    const SELECTION_INDICES: [u32; 6] = [
        3, 1, 0,
        1, 3, 2,
    ];

    #[inline(always)]
    pub fn new(title: &str) -> Self {
        Self {
            offset: Default::default(),
            title: TitleText::new(title),
            input_text: None,
            input_text_formatted: None,
            format_input: Box::new(|fmt, input| -> core::fmt::Result {
                write!(fmt, "{}", input)
            }),
            empty_input_prompt: Default::default(),
            input: Default::default(),
            input_offsets: Default::default(),
            input_text_offset_x: 0.0,
            text_cursor_pos: Default::default(),
            selection: None,
            input_rect: Default::default(),
            input_rect_vertex_range: Default::default(),
            input_rect_outline_vertex_range: Default::default(),
            selection_rect_vertices: Default::default(),
            cursor_rect: Default::default(),
            cursor_rect_vertex_range: Default::default(),
            flags: Self::MOUSE_VISIBLE | Self::CURSOR_ENABLE,
            cursor_timer: 0.0,
            double_click_timer: 100.0,
            outline_width: 0.0,
            width_override: 0.0,
            bg_col_override: Default::default(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn input_offset(&self, style: &Style) -> Vec2 {
        self.offset + vec2(
            if self.skip_title() {
                Default::default()
            } else {
                let text_width = self.title.get_text_width();
                let title_width = text_width * style.font_scale();
                title_width + style.item_pad_outer().x
            },
            0.0,
        )
    }

    #[inline(always)]
    pub fn rel_bounding_rect(&self, style: &Style) -> BoundingRect {
        BoundingRect::from_position_size(
            self.input_offset(style),
            self.input_rect.max,
        )
    }

    #[inline(always)]
    pub fn outline_points(&self, width: f32, outline_points: &mut GlobalVec<[f32; 2]>) {
        let mut points = GlobalVec::new();
        self.input_rect.to_points(&mut |p| { points.push(p); });
        nox_geom::shapes::outline_points(
            &points,
            width, false,
            &mut |p| { outline_points.push(p.into()); }
        );
    }

    #[inline(always)]
    pub fn set_params(
        &mut self,
        width_override: Option<f32>,
        bg_col_override: Option<ColorSRGBA>,
        skip_title: bool,
        center_text: bool,
        empty_input_prompt: &str,
        format_input: Option<fn(&mut dyn Write, &str) -> core::fmt::Result>,
    )
    {
        self.flags &= !(
            Self::CENTER_TEXT |
            Self::SKIP_TITLE |
            Self::WIDTH_OVERRIDE |
            Self::BG_COL_OVERRIDE
        );
        self.flags |= Self::CENTER_TEXT * center_text as u32;
        if let Some(width_override) = width_override {
            self.width_override = width_override;
            self.flags |= Self::WIDTH_OVERRIDE;
        }
        if let Some(bg_col_override) = bg_col_override {
            self.bg_col_override = bg_col_override;
            self.flags |= Self::BG_COL_OVERRIDE;
        }
        self.flags |= Self::SKIP_TITLE * skip_title as u32;
        if let Some(format) = format_input {
            *self.format_input = format;
        }
        if empty_input_prompt != self.empty_input_prompt {
            self.empty_input_prompt = CompactString::new(empty_input_prompt);
        }
    }

    #[inline(always)]
    pub fn set_input_sliderable(&mut self, style: &Style, input: &impl Sliderable) {
        self.flags &= !Self::FORMAT_ERROR;
        let mut fmt = CompactString::default();
        if let Err(_) = input.display(style, &mut fmt) {
            self.flags |= Self::FORMAT_ERROR;
        }
        if fmt != self.input {
            self.input = fmt;
            self.input_offsets.clear();
            self.input_text = None;
        }
        self.text_cursor_pos = 0;
    }

    #[inline(always)]
    pub fn set_input(&mut self, input: &impl Display) {
        self.flags &= !Self::FORMAT_ERROR;
        let mut fmt = CompactString::default();
        if let Err(_) = write!(fmt, "{}", input) {
            self.flags |= Self::FORMAT_ERROR;
        }
        if fmt != self.input {
            self.input = fmt;
            self.input_offsets.clear();
            self.input_text = None;
        }
        self.text_cursor_pos = 0;
    }

    #[inline(always)]
    pub fn get_input<T: FromStr>(&mut self) -> Option<T> {
        T::from_str(&self.input).ok()
    }

    #[inline(always)]
    pub fn activate_and_select_all(&mut self) {
        self.flags |= Self::CURSOR_ENABLE;
        self.flags |= Self::ACTIVE;
        self.flags |= Self::ACTIVATED_LAST_FRAME;
        self.flags |= Self::CURSOR_VISIBLE;
        self.flags |= Self::SELECT_ALL;
        self.cursor_timer = 0.0;
    }

    #[inline(always)]
    pub fn set_cursor_enable(&mut self, value: bool) {
        self.flags &= !Self::CURSOR_ENABLE;
        self.flags |= Self::CURSOR_ENABLE * value as u32;
    }

    #[inline(always)]
    fn cursor_enabled(&self) -> bool {
        self.flags & Self::CURSOR_ENABLE == Self::CURSOR_ENABLE
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
    pub fn active(&self) -> bool {
        self.flags & Self::ACTIVE == Self::ACTIVE
    }

    #[inline(always)]
    fn cursor_visible(&self) -> bool {
        self.flags & Self::CURSOR_VISIBLE == Self::CURSOR_VISIBLE
    }

    #[inline(always)]
    fn mouse_visible(&self) -> bool {
        self.flags & Self::MOUSE_VISIBLE == Self::MOUSE_VISIBLE
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
    fn has_format_error(&self) -> bool {
        self.flags & Self::FORMAT_ERROR == Self::FORMAT_ERROR
    }

    #[inline(always)]
    fn skip_title(&self) -> bool {
        self.flags & Self::SKIP_TITLE == Self::SKIP_TITLE
    }

    #[inline(always)]
    fn has_width_override(&self) -> bool {
        self.flags & Self::WIDTH_OVERRIDE == Self::WIDTH_OVERRIDE
    }

    #[inline(always)]
    fn has_bg_col_override(&self) -> bool {
        self.flags & Self::BG_COL_OVERRIDE == Self::BG_COL_OVERRIDE
    }

    #[inline(always)]
    fn center_text(&self) -> bool {
        self.flags & Self::CENTER_TEXT == Self::CENTER_TEXT
    }

    #[inline(always)]
    fn select_all(&self) -> bool {
        self.flags & Self::SELECT_ALL == Self::SELECT_ALL
    }

    #[inline(always)]
    fn clicked_last_frame(&self) -> bool {
        self.flags & Self::CLICKED_LAST_FRAME == Self::CLICKED_LAST_FRAME
    }

    #[inline(always)]
    fn select_all_last_frame(&self) -> bool {
        self.flags & Self::SELECT_ALL_LAST_FRAME == Self::SELECT_ALL_LAST_FRAME
    }

    #[inline(always)]
    fn activated_last_frame(&self) -> bool {
        self.flags & Self::ACTIVATED_LAST_FRAME == Self::ACTIVATED_LAST_FRAME
    }

    #[inline(always)]
    fn calc_cursor_index(
        &self,
        rel_cursor_pos: f32,
        text_min_max: (f32, f32),
        text_box_min_max: (f32, f32),
        input_offset: f32,
    ) -> usize
    {
        if self.input_offsets.is_empty() { return 0 }
        if rel_cursor_pos < text_box_min_max.0 {
            for i in 1..self.input_offsets.len() {
                let offset = input_offset + self.input_offsets[i].x;
                if offset >= text_box_min_max.0 {
                    return i - 1
                }
            }
            return self.input_offsets.len() - 1
        }
        if rel_cursor_pos > text_box_min_max.1 {
            for (i, &offset) in self.input_offsets.iter().enumerate().rev() {
                let offset = offset.x;
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
            let offset = self.input_offsets[i].x;
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

impl<TitleText, I, FontHash, Style, HoverStyle> Widget<I, FontHash, Style, HoverStyle> for
        InputText<TitleText, I, FontHash, Style, HoverStyle>
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
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> f32
    {
        style.calc_text_box_height_from_text_height(style.calc_font_height(text_renderer))
    }

    fn is_active(
        &self,
        _nox: &Nox<I>,
        _style: &Style,
        _hover_style: &HoverStyle,
        _window_pos: Vec2,
        _cursor_pos: Vec2
    ) -> bool
    {
        self.active() || self.held()
    }

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        _hover_style: &HoverStyle,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        _cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult {
        enum CursorMove {
            None,
            Left,
            Right,
            Backspace,
        }
        let mut cursor_move = CursorMove::None;
        let font_scale = style.font_scale();
        let start_width = self.input_text
            .as_ref()
            .map(|v| style.calc_text_width(v))
            .unwrap_or_default();
        let active_this_frame = self.active();
        if active_this_frame {
            let mut cursor_timer = self.cursor_timer + nox.delta_time().as_secs_f32();
            if cursor_timer >= style.input_text_cursor_switch_speed() {
                self.toggle_cursor_visible();
                cursor_timer = 0.0;
            }
            self.cursor_timer = cursor_timer;
            if let Some(mut selection) = self.selection && selection.0 != selection.1 {
                self.selection = None;
                if self.selection_left() {
                    self.text_cursor_pos = selection.0;
                } else {
                    self.text_cursor_pos = selection.1;
                }
                let input = nox.get_input_text();
                if nox.was_key_pressed(KeyCode::Backspace) || !input.is_empty() {
                    let start_count = self.input.char_indices().count();
                    for i in (selection.0..selection.1).rev() {
                        let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                        self.input.remove(index);
                    }
                    let mut text_cursor_pos = selection.0;
                    for text in input {
                        if text.0 != KeyCode::Backspace && text.0 != KeyCode::Enter &&
                            text.0 != KeyCode::Escape
                        {
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
                    let end_count = self.input.char_indices().count();
                    if start_count > end_count {
                        cursor_move = CursorMove::Backspace;
                    }
                    self.input_text = None;
                } else if nox.was_key_pressed(KeyCode::ArrowLeft) {
                    if nox.is_key_held(KeyCode::ShiftLeft) {
                        if self.selection_left() {
                            if selection.0 != 0 {
                                selection.0 -= 1;
                                self.text_cursor_pos = selection.0;
                            }
                            self.text_cursor_pos = selection.0;
                        } else if selection.1 != selection.0 {
                            selection.1 -= 1;
                            self.text_cursor_pos = selection.1;
                        }
                        cursor_move = CursorMove::Left;
                        self.selection = Some(selection);
                    } else {
                        self.text_cursor_pos = selection.0;
                    }
                    self.flags |= Self::CURSOR_VISIBLE;
                    self.cursor_timer = 0.0;
                } else if nox.was_key_pressed(KeyCode::ArrowRight) {
                    if nox.is_key_held(KeyCode::ShiftLeft) {
                        if self.selection_left() {
                            if selection.0 != selection.1 {
                                selection.0 += 1;
                            }
                            self.text_cursor_pos = selection.0;
                        } else if selection.1 != self.input_offsets.len() {
                            selection.1 += 1;
                            self.text_cursor_pos = selection.1;
                        }
                        cursor_move = CursorMove::Right;
                        self.selection = Some(selection);
                    } else {
                        self.text_cursor_pos = selection.1;
                    }
                    self.flags |= Self::CURSOR_VISIBLE;
                    self.cursor_timer = 0.0;
                } else {
                    self.selection = Some(selection);
                }
            }
            else {
                self.flags &= !Self::SELECTION_LEFT;
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
                        if nox.is_key_held(KeyCode::ShiftLeft) {
                            self.selection = Some((text_cursor_pos - 1, text_cursor_pos));
                            self.flags |= Self::SELECTION_LEFT;
                        }
                        text_cursor_pos -= 1;
                        self.cursor_timer = 0.0;
                        self.flags |= Self::CURSOR_VISIBLE;
                    }
                }
                let input = nox.get_input_text();
                if !input.is_empty() {
                    for text in input {
                        if text.0 != KeyCode::Backspace &&
                            text.0 != KeyCode::Enter && text.0 != KeyCode::Escape
                        {
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
                    if text_cursor_pos != end_count && nox.is_key_held(KeyCode::ShiftLeft) {
                        self.selection  = Some((text_cursor_pos - 1, text_cursor_pos));
                    }
                    self.cursor_timer = 0.0;
                    self.flags |= Self::CURSOR_VISIBLE;
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
        let item_pad_outer = style.item_pad_outer();
        let item_pad_inner = style.item_pad_inner();
        let has_format_error = self.has_format_error();
        let skip_title = self.skip_title();
        let title = if !skip_title {
            let title = self.title.update(text_renderer, style.font_regular());
            let (min_bounds, max_bounds) = calc_bounds(window_pos, self.offset, window_size);
            if let Some(title) = &title {
                collect_text(title, self.offset + vec2(0.0, item_pad_inner.y), BoundedTextInstance {
                    add_scale: vec2(1.0, 1.0),
                    min_bounds,
                    max_bounds,
                    color: style.text_col(),
                });
            }
            title
        } else {
            None
        };
        self.input_text.get_or_insert_with(|| {
            if self.input.is_empty() {
                self.input_text_formatted = Some(text_renderer
                    .render(&[text_segment(&self.empty_input_prompt, style.font_regular())],
                    false, 0.0
                ).unwrap_or_default())
            } else {
                let mut fmt = CompactString::default();
                if has_format_error {
                    fmt = "Format error!".into();
                }
                else if let Err(e) = (self.format_input)(&mut fmt, &self.input) {
                    fmt.clear();
                    write!(fmt, "Format error: ! {}", e).ok();
                }
                self.input_text_formatted = Some(text_renderer
                    .render(&[text_segment(&fmt, style.font_regular())],
                    false, 0.0
                ).unwrap_or_default());
            }
            self.input_offsets.clear();
            text_renderer
                .render_and_collect_offsets(
                    &[text_segment(&self.input, style.font_regular())],
                    false, 0.0,
                    |_, [x, y]| { self.input_offsets.push(vec2(x, y) * font_scale); }
                )
                .unwrap_or_default()
        });
        let input_text = unsafe {
            self.input_text.as_ref().unwrap_unchecked()
        };
        let Vec2 { x: title_width, y: title_height } =
            if skip_title {
                vec2(0.0, style.calc_font_height(text_renderer))
            } else {
                style.calc_text_size(title.unwrap())
            };
        let offset = self.offset;
        let input_offset = if skip_title {
            offset
        } else {
            offset + vec2(title_width + item_pad_outer.x, 0.0)
        };
        let input_width = style.calc_text_width(input_text);
        let mut width = if self.has_width_override() {
            if self.center_text() {
                (input_width + item_pad_inner.x + item_pad_inner.x).max(self.width_override)
            } else {
                self.width_override
            }
        } else {
            let mut width = title_width +
                item_pad_outer.x + item_pad_outer.x + item_pad_outer.x;
            let min_window_width = width + style.min_input_text_width();
            if window_size.x < min_window_width {
                width = style.min_input_text_width();
            } else {
                width = (window_size.x - width).min(style.max_input_text_width());
            }
            width
        };
        let mut input_rect = rect(
            Default::default(),
            vec2(
                width,
                style.calc_text_box_height(input_text).max(
                    style.calc_text_box_height_from_text_height(style.calc_font_height(text_renderer))
                )
            ),
            style.rounding(),
        );
        let input_text_offset = input_offset + item_pad_inner;
        let text_cursor_pos_x = self.input_offsets
            .get(self.text_cursor_pos)
            .map(|v| v.x)
            .unwrap_or_else(|| input_width);
        let input_text_max_x = width - item_pad_inner.x - item_pad_inner.x;
        if !self.center_text() {
            match cursor_move {
                CursorMove::None => {},
                CursorMove::Left => {
                    if text_cursor_pos_x - self.input_text_offset_x < 0.0 {
                        self.input_text_offset_x = text_cursor_pos_x;
                    }
                    self.flags &= !Self::MOUSE_VISIBLE;
                },
                CursorMove::Right => {
                    if input_text_max_x -
                        self.calc_cursor_offset(font_scale, self.text_cursor_pos).x < 0.0
                    {
                        self.input_text_offset_x = text_cursor_pos_x - input_text_max_x;
                    }
                    self.flags &= !Self::MOUSE_VISIBLE;
                },
                CursorMove::Backspace => {
                    let pos = self.calc_cursor_offset(font_scale, self.text_cursor_pos).x;
                    let delta = start_width - input_width;
                    if input_text_max_x - pos > 0.0 &&
                        input_width - text_cursor_pos_x <
                        input_text_max_x - pos
                    {
                        self.input_text_offset_x =
                            (self.input_text_offset_x - delta).clamp(0.0, f32::INFINITY);
                    }
                    self.flags &= !Self::MOUSE_VISIBLE;
                }
            }
        }
        if delta_cursor_pos.x != 0.0 || delta_cursor_pos.y != 0.0 {
            self.flags |= Self::MOUSE_VISIBLE;
        }
        let mouse_visible = self.mouse_visible();
        let override_cursor = style.override_cursor();
        self.flags &= !Self::HOVERING;
        let mut cursor_in_widget = !mouse_visible;
        let mouse_released = nox.was_mouse_button_released(MouseButton::Left);
        let mouse_pressed = nox.was_mouse_button_pressed(MouseButton::Left);
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = style.cursor_error_margin();
        if !other_widget_active {
            if override_cursor {
                nox.set_cursor_hide(!mouse_visible);
            }
            if mouse_visible && self.cursor_enabled() {
                self.flags |= Self::HOVERING *
                    BoundingRect::from_position_size(
                        input_offset - vec2(error_margin, 0.0),
                        input_rect.max + vec2(error_margin + error_margin, 0.0)
                    ).is_point_inside(rel_cursor_pos) as u32;
                let input_min = input_offset.x + item_pad_inner.x - self.input_text_offset_x;
                let input_min_max = (input_min, input_min + input_width);
                let input_box_min_max =
                    (input_offset.x, input_offset.x + input_rect.max.x - item_pad_inner.x);
                let mut select_all = false;
                if self.hovering() {
                    cursor_in_widget = true;
                    if mouse_pressed {
                        if self.double_click_timer < style.double_click_secs() {
                            select_all = true;
                        } else {
                            self.text_cursor_pos =
                                self.calc_cursor_index(
                                    rel_cursor_pos.x,
                                    input_min_max,
                                    input_box_min_max,
                                    input_min,
                                );
                            self.flags |= Self::HELD;
                        }
                        self.double_click_timer = 0.0;
                    }
                    if override_cursor {
                        nox.set_cursor(CursorIcon::Text);
                    }
                }
                select_all &= self.selection.is_none();
                if select_all || self.select_all() {
                    self.flags |= Self::ACTIVE;
                    self.selection = Some((0, self.input_offsets.len()));
                    self.flags |= Self::CURSOR_VISIBLE;
                    self.cursor_timer = 0.0;
                }
                if !self.hovering() && !self.held() && !window_moving && mouse_released {
                    self.flags &= !Self::ACTIVE;
                    self.flags |= Self::MOUSE_VISIBLE;
                }
                if !select_all && !window_moving {
                    if self.clicked_last_frame() && !self.select_all_last_frame() && !self.activated_last_frame() {
                        self.selection = None;
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
                                            style.input_text_selection_scroll_speed() *
                                            nox.delta_time_secs_f32();
                                    } else if offset > input_text_max_x {
                                        self.input_text_offset_x +=
                                            style.input_text_selection_scroll_speed() *
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
                                            style.input_text_selection_scroll_speed() *
                                            nox.delta_time_secs_f32();
                                    } else if offset < 0.0 {
                                        self.input_text_offset_x -=
                                            style.input_text_selection_scroll_speed() *
                                            nox.delta_time_secs_f32();
                                    }
                                    if selection.1 < selection.0 {
                                        let tmp = selection.0;
                                        selection.0 = selection.1;
                                        selection.1 = tmp;
                                        self.flags |= Self::SELECTION_LEFT;
                                    }
                                }
                                selection.1 = selection.1.clamp(0, self.input_offsets.len());
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
                }
                self.flags &= !Self::SELECT_ALL_LAST_FRAME;
                self.flags |= Self::SELECT_ALL_LAST_FRAME * (select_all || self.select_all()) as u32;
                self.flags &= !Self::SELECT_ALL;
                self.flags &= !Self::CLICKED_LAST_FRAME;
                self.flags |= Self::CLICKED_LAST_FRAME * mouse_pressed as u32;
            }
            let deactivate =
                nox.was_key_pressed(KeyCode::Enter) |
                nox.was_key_pressed(KeyCode::Escape);
            if deactivate {
                self.flags &= !Self::ACTIVE;
                self.flags |= Self::MOUSE_VISIBLE;
            }
        } else {
            self.flags &= !Self::ACTIVE;
        }
        if self.center_text() {
            let text_width =  if self.active() {
                style.calc_text_width(self.input_text.as_ref().unwrap())
            } else {
                style.calc_text_width(self.input_text_formatted.as_ref().unwrap())
            };
            if text_width + item_pad_inner.x + item_pad_inner.x > self.width_override {
                self.input_text_offset_x = 0.0;
                input_rect.max.x = style.calc_text_box_width_from_text_width(text_width);
            } else {
                self.input_text_offset_x =
                    text_width * 0.5 - self.width_override * 0.5 + style.item_pad_inner().x;
            }
        }
        let cursor_rect_max = vec2(style.input_text_cursor_width(), title_height);
        let requires_triangulation =
            self.input_rect != input_rect ||
            self.cursor_rect.max != cursor_rect_max ||
            self.outline_width != style.outline_width();
        self.input_rect = input_rect;
        self.cursor_rect.max = cursor_rect_max;
        self.outline_width = style.outline_width();
        if !self.active() {
            self.selection = None;
        }
        if let Some(selection) = self.selection {
            let left = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.0).x.clamp(0.0, input_text_max_x);
            let right = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.1).x.clamp(0.0, input_text_max_x);
            let rect = BoundingRect::from_min_max(
                vec2(left, offset.y + item_pad_inner.y),
                vec2(right, offset.y + input_rect.max.y - item_pad_inner.y),
            );
            let col = style.input_text_selection_bg_col();
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
        self.double_click_timer += nox.delta_time_secs_f32();
        if self.has_width_override() && !self.skip_title() {
            width += self.title.get_text_width() * font_scale + item_pad_outer.x;
        }
        self.flags &= !Self::ACTIVATED_LAST_FRAME;
        self.flags |= Self::ACTIVATED_LAST_FRAME * (!active_this_frame && self.active()) as u32;
        let input_off = self.offset + 
            if !self.skip_title() {
                vec2(title_width + item_pad_outer.x, 0.0)
            } else {
                Default::default()
            };
        let input_bounding_rect = BoundingRect::from_position_size(
            window_pos + input_off + vec2(item_pad_inner.x, 0.0),
            self.input_rect.max - vec2(item_pad_inner.x + item_pad_inner.x, 0.0),
        );
        collect_text(
            if self.active() {
                self.input_text.as_ref().unwrap()
            } else {
                self.input_text_formatted.as_ref().unwrap()
            },
            input_off + item_pad_inner - vec2(self.input_text_offset_x, 0.0), 
            BoundedTextInstance {
                add_scale: vec2(1.0, 1.0),
                min_bounds: input_bounding_rect.min,
                max_bounds: input_bounding_rect.max,
                color:
                    if self.input.is_empty() {
                        style.input_text_empty_text_color()
                    } else {
                        style.text_col()
                    },
            }
        );
        UpdateResult {
            min_widget_width: width,
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
        let mut outline_points = GlobalVec::new();
        nox_geom::shapes::outline_points(
            points, self.outline_width, false,
            &mut |p| { outline_points.push(p.into()); }
        );
        self.input_rect_outline_vertex_range = tri(&outline_points);
        self.input_rect_vertex_range = tri(points);
        points.clear();
        self.cursor_rect.to_points(&mut |p| { points.push(p.into()); });
        self.cursor_rect_vertex_range = tri(points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        _hover_style: &HoverStyle,
        vertices: &mut [shaders::Vertex],
    )
    {
        let mut offset = if !self.skip_title() {
            let title_width = style.font_scale() * self.title.get_text_width();
            self.offset + vec2(title_width + style.item_pad_outer().x, 0.0)
        } else {
            self.offset
        };
        let mut target_color = if self.has_bg_col_override() {
            self.bg_col_override
        } else {
            style.input_text_bg_col()
        };
        set_vertex_params(vertices, self.input_rect_vertex_range, offset, target_color);
        target_color = style.input_text_active_outline_col();
        if self.active() {
            set_vertex_params(vertices, self.input_rect_outline_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.input_rect_outline_vertex_range);
        }
        if self.cursor_visible() {
            offset += style.item_pad_inner() +
                self.calc_cursor_offset(style.font_scale(), self.text_cursor_pos);
            target_color = style.text_col();
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
        _style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        _text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        _get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>
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
            let pc_vertex = push_constants_vertex(window_pos, vec2(1.0, 1.0), inv_aspect_ratio, unit_scale);
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
        Ok(None)
    }
}
