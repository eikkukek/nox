use std::rc::Rc;

use core::{
    marker::PhantomData,
    fmt::{Display, Write},
    str::FromStr,
    cell::RefCell,
};

use compact_str::CompactString;

use nox::{alloc::arena_alloc::ArenaGuard, mem::vec_types::{GlobalVec, Vector}, *};

use nox_geom::{
    shapes::*,
    *,
};

use nox_font::{text_segment, RenderedText};

use crate::{
    surface::*,
    *
};

pub struct InputTextData {
    input: CompactString,
    input_text: Option<RenderedText>,
    input_text_formatted: Option<RenderedText>,
    empty_input_prompt: CompactString,
    format_input: Box<fn(&mut dyn Write, &str) -> core::fmt::Result>,
    input_offsets: GlobalVec<Vec2>,
    row_offsets: RowOffsets,
    formatted_row_offsets: RowOffsets,
    input_text_offset_x: f32,
    selection: Option<(usize, usize)>,
    text_cursor_pos: usize,
    cursor_timer: f32,
    double_click_timer: f32,
    width: f32,
    bg_col_override: ColorSRGBA,
    flags: u32,
}

impl InputTextData {

    const ACTIVE: u32 = 0x1;
    const CURSOR_VISIBLE: u32 = 0x2;
    const SELECTION_LEFT: u32 = 0x4;
    const MOUSE_VISIBLE: u32 = 0x8;
    const FORMAT_ERROR: u32 = 0x10;
    const CENTER_TEXT: u32 = 0x20;
    const CURSOR_ENABLE: u32 = 0x40;
    const SELECT_ALL: u32 = 0x80;
    const BG_COL_OVERRIDE: u32 = 0x100;
    const CLICKED_LAST_FRAME: u32 = 0x200;
    const SELECT_ALL_LAST_FRAME: u32 = 0x400;
    const ACTIVATED_LAST_FRAME: u32 = 0x800;
    const PARENT_ACTIVE: u32 = 0x1000;
    const ACTIVE_LAST_FRAME: u32 = 0x2000;
    
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            input: Default::default(),
            input_text: None,
            input_text_formatted: None,
            empty_input_prompt: Default::default(),
            format_input: Box::new(|fmt, input| -> core::fmt::Result {
                write!(fmt, "{}", input)
            }),
            input_offsets: Default::default(),
            row_offsets: Default::default(),
            formatted_row_offsets: Default::default(),
            input_text_offset_x: 0.0,
            selection: None,
            text_cursor_pos: 0,
            cursor_timer: 0.0,
            double_click_timer: f32::MAX,
            width: 0.0,
            bg_col_override: Default::default(),
            flags: Self::MOUSE_VISIBLE | Self::CURSOR_ENABLE,
        }
    }

    #[inline(always)]
    pub fn set_params(
        &mut self,
        width: f32,
        bg_col_override: Option<ColorSRGBA>,
        center_text: bool,
        empty_input_prompt: &str,
        format_input: Option<fn(&mut dyn Write, &str) -> core::fmt::Result>,
        parent_active: bool,
    )
    {
        self.flags &= !(
            Self::CENTER_TEXT |
            Self::BG_COL_OVERRIDE |
            Self::PARENT_ACTIVE
        );
        or_flag!(self.flags, Self::CENTER_TEXT, center_text);
        or_flag!(self.flags, Self::PARENT_ACTIVE, parent_active);
        self.width = width;
        if let Some(bg_col_override) = bg_col_override {
            self.bg_col_override = bg_col_override;
            self.flags |= Self::BG_COL_OVERRIDE;
        }
        if let Some(format) = format_input {
            *self.format_input = format;
        }
        if empty_input_prompt != self.empty_input_prompt {
            self.empty_input_prompt = CompactString::new(empty_input_prompt);
        }
    }

    #[inline(always)]
    pub fn set_input_sliderable<T>(
        &mut self,
        style: &impl UiStyle,
        input: &T
    )
        where 
            T: Sliderable,
    {
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

    #[inline(always)]
    pub fn set_cursor_enable(&mut self, value: bool) {
        self.flags &= !Self::CURSOR_ENABLE;
        or_flag!(self.flags, Self::CURSOR_ENABLE, value);
    }

    pub fn update<Surface: UiReactSurface, Style: UiStyle>(
        &mut self,
        ui: &mut UiReactCtx<Surface, Style>,
        reaction: &mut Reaction,
    )
    {
        enum CursorMove {
            None,
            Left,
            Right,
            Backspace,
        }
        let mut cursor_move = CursorMove::None;
        let font_scale = ui.style().font_scale();
        let start_width = self.input_text
            .as_ref()
            .map(|v| ui.style().calc_text_width(v))
            .unwrap_or_default();
        let active_this_frame = self.active();
        if active_this_frame {
            let mut cursor_timer = self.cursor_timer + ui.win_ctx().delta_time_secs_f32();
            if cursor_timer >= ui.style().input_text_cursor_switch_speed() {
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
                let input_empty = ui.win_ctx().get_input_text().0 == 0;
                if ui.win_ctx().key_state(KeyCode::ControlLeft).held() {
                    if ui.win_ctx().key_state(KeyCode::KeyV).pressed() &&
                        let Some(text) = ui.win_ctx().get_clipboard()
                    {
                        let start_count = self.input.char_indices().count();
                        for i in (selection.0..selection.1).rev() {
                            let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                            self.input.remove(index); 
                        }
                        let mut text_cursor_pos = selection.0;
                        self.input.insert_str(
                            self.input
                                .char_indices()
                                .skip(text_cursor_pos)
                                .next()
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| self.input.len()),
                            &text,
                        );
                        text_cursor_pos += text.char_indices().count();
                        self.text_cursor_pos = text_cursor_pos;
                        self.input_text = None;
                        let end_count = self.input.char_indices().count();
                        if start_count > end_count {
                            cursor_move = CursorMove::Backspace;
                        } else {
                            cursor_move = CursorMove::Right;
                        }
                    } else if ui.win_ctx().key_state(KeyCode::KeyC).pressed() {
                        let mut text = CompactString::default();
                        let mut iter = self.input.char_indices().skip(selection.0);
                        for _ in selection.0..selection.1  {
                            text.push(iter.next().unwrap().1);
                        }
                        ui.win_ctx().set_clipboard(&text);
                        self.selection = Some(selection);
                    } else if ui.win_ctx().key_state(KeyCode::KeyX).pressed() {
                        let mut text = CompactString::default();
                        let mut iter = self.input.char_indices().skip(selection.0);
                        for _ in selection.0..selection.1  {
                            text.push(iter.next().unwrap().1);
                        }
                        ui.win_ctx().set_clipboard(&text);
                        for i in (selection.0..selection.1).rev() {
                            let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                            self.input.remove(index);
                        }
                        self.text_cursor_pos = selection.0;
                        self.input_text = None;
                        cursor_move = CursorMove::Backspace;
                    } else {
                        self.selection = Some(selection);
                    }
                }
                else if ui.win_ctx().key_state(KeyCode::Backspace).pressed() || !input_empty {
                    let start_count = self.input.char_indices().count();
                    for i in (selection.0..selection.1).rev() {
                        let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                        self.input.remove(index);
                    }
                    let mut text_cursor_pos = selection.0;
                    for text in ui.win_ctx().get_input_text().1 {
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
                                text.1
                            );
                            text_cursor_pos += text.1.char_indices().count();
                        }
                    }
                    self.text_cursor_pos = text_cursor_pos;
                    let end_count = self.input.char_indices().count();
                    if start_count > end_count {
                        cursor_move = CursorMove::Backspace;
                    } else {
                        cursor_move = CursorMove::Right;
                    }
                    self.input_text = None;
                } else if ui.win_ctx().key_state(KeyCode::ArrowLeft).pressed() {
                    if ui.win_ctx().key_state(KeyCode::ShiftLeft).held() {
                        if self.selection_left() {
                            if selection.0 != 0 {
                                selection.0 -= 1;
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
                } else if ui.win_ctx().key_state(KeyCode::ArrowRight).pressed() {
                    if ui.win_ctx().key_state(KeyCode::ShiftLeft).held() {
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
                    if ui.win_ctx().key_state(KeyCode::Backspace).pressed() {
                        let remove = text_cursor_pos - 1;
                        let (index, _) = self.input.char_indices().skip(remove).next().unwrap();
                        self.input.remove(index);
                        self.input_text = None;
                        text_cursor_pos = remove;
                    } else if ui.win_ctx().key_state(KeyCode::ArrowLeft).pressed() {
                        if ui.win_ctx().key_state(KeyCode::ShiftLeft).held() {
                            self.selection = Some((text_cursor_pos - 1, text_cursor_pos));
                            self.flags |= Self::SELECTION_LEFT;
                        }
                        text_cursor_pos -= 1;
                        self.cursor_timer = 0.0;
                        self.flags |= Self::CURSOR_VISIBLE;
                    }
                }
                if ui.win_ctx().key_state(KeyCode::ControlLeft).held()
                {
                    if ui.win_ctx().key_state(KeyCode::KeyV).pressed() && let Some(text) = ui.win_ctx().get_clipboard() {
                        self.input.insert_str(
                            self.input
                                .char_indices()
                                .skip(text_cursor_pos)
                                .next()
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| self.input.len()),
                            &text,
                        );
                        text_cursor_pos += text.char_indices().count();
                        self.input_text = None;
                    }
                } else {
                    let input = ui.win_ctx().get_input_text();
                    if input.0 != 0 {
                        for text in input.1 {
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
                }
                let end_count = self.input.char_indices().count();
                if ui.win_ctx().key_state(KeyCode::ArrowRight).pressed() {
                    text_cursor_pos = (text_cursor_pos + 1).clamp(0, end_count);
                    if text_cursor_pos != end_count && ui.win_ctx().key_state(KeyCode::ShiftLeft).held() {
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
        let item_pad_inner = ui.style().item_pad_inner();
        let has_format_error = self.has_format_error();
        let text_col =
            if self.active() || self.parent_active() {
                ui.style().active_text_col()
            } else if reaction.hovered() {
                ui.style().focused_text_col()
            } else {
                ui.style().inactive_text_col()
            };
        self.input_text.get_or_insert_with(|| {
            self.formatted_row_offsets.offsets.clear();
            let mut input_text = Default::default();
            ui.render_text(|style, renderer| {
                if self.input.is_empty() {
                    let text = self.input_text_formatted.insert(renderer
                        .render_and_collect_offsets(&[text_segment(&self.empty_input_prompt, style.font_regular())],
                        false, 0.0,
                        0.0,
                        |offset| {
                            self.formatted_row_offsets.offsets.push(offset);
                        },
                    ).unwrap_or_default());
                    self.formatted_row_offsets.row_height = text.row_height;
                } else {
                    let mut fmt = CompactString::default();
                    if has_format_error {
                        fmt = "Format error!".into();
                    }
                    else if let Err(e) = (self.format_input)(&mut fmt, &self.input) {
                        fmt.clear();
                        write!(fmt, "Format error: ! {}", e).ok();
                    }
                    let text = self.input_text_formatted.insert(renderer
                        .render_and_collect_offsets(&[text_segment(&fmt, style.font_regular())],
                        false, 0.0,
                        0.0,
                        |offset| {
                            self.formatted_row_offsets.offsets.push(offset);
                        },
                    ).unwrap_or_default());
                    self.formatted_row_offsets.row_height = text.row_height;
                }
                self.input_offsets.clear();
                self.row_offsets.offsets.clear();
                let text = renderer
                    .render_and_collect_offsets(
                        &[text_segment(&self.input, style.font_regular())],
                        false, 0.0,
                        0.0,
                        |offset| {
                            self.row_offsets.offsets.push(offset);
                            self.input_offsets.push(vec2(offset.offset[0], offset.offset[1]) * font_scale);
                        },
                    )
                    .unwrap_or_default();
                self.row_offsets.row_height = text.row_height;
                input_text = text
            });
            input_text
        });
        let input_text = unsafe {
            self.input_text
                .as_ref()
                .unwrap_unchecked()
        };
        let text_height = ui.font_height();
        let offset = reaction.offset;
        let input_width = ui.style().calc_text_width(input_text);
        let width =
            if self.center_text() {
                (input_width + item_pad_inner.x + item_pad_inner.x).max(self.width)
            } else {
                self.width
            };
        let mut input_rect = rect(
            Default::default(),
            vec2(
                width,
                ui.style().calc_text_box_height(input_text).max(
                    ui.style().calc_text_box_height_from_text_height(text_height)
                )
            ),
            ui.style().rounding(),
        );
        let input_text_offset = offset + item_pad_inner;
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
        if ui.win_ctx().cursor_moved() {
            self.flags |= Self::MOUSE_VISIBLE;
        }
        let mouse_visible = self.mouse_visible();
        let rel_cursor_pos = reaction.rel_cursor_pos();
        let override_cursor = ui.style().override_cursor();
        let mouse_left_state = ui.win_ctx().mouse_button_state(MouseButton::Left);
        if override_cursor {
            if self.active() {
                ui.win_ctx().set_cursor_hide(!mouse_visible);
            } else if self.active_last_frame() {
                ui.win_ctx().set_cursor_hide(false);
            }
        }
        self.flags &= !Self::ACTIVE_LAST_FRAME;
        or_flag!(self.flags, Self::ACTIVE_LAST_FRAME, self.active());
        let surface_moving = ui.surface_moving();
        if mouse_visible && self.cursor_enabled() {
            let input_min = offset.x + item_pad_inner.x - self.input_text_offset_x;
            let input_min_max = (input_min, input_min + input_width);
            let input_box_min_max =
                (offset.x, offset.x + input_rect.max.x - item_pad_inner.x);
            let mut select_all = false;
            if reaction.clicked() {
                if self.double_click_timer < ui.style().double_click_secs() {
                    select_all = true;
                } else {
                    self.text_cursor_pos =
                        self.calc_cursor_index(
                            rel_cursor_pos.x,
                            input_min_max,
                            input_box_min_max,
                            input_min,
                        );
                }
                self.double_click_timer = 0.0;
            }
            if reaction.held() || reaction.hovered() {
                reaction.cursor(CursorIcon::Text);
            }
            select_all &= self.selection.is_none();
            if select_all || self.select_all() {
                self.flags |= Self::ACTIVE;
                self.selection = Some((0, self.input_offsets.len()));
                self.flags |= Self::CURSOR_VISIBLE;
                self.cursor_timer = 0.0;
            }
            if !reaction.hovered() && !reaction.held() && !surface_moving && mouse_left_state.released() {
                self.flags &= !Self::ACTIVE;
                self.flags |= Self::MOUSE_VISIBLE;
            }
            if !select_all && !surface_moving {
                if self.clicked_last_frame() &&
                    !self.select_all_last_frame() &&
                    !self.activated_last_frame()
                {
                    self.selection = None;
                }
                if reaction.held() {
                    if mouse_left_state.released() {
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
                                        ui.style().input_text_selection_scroll_speed() *
                                        ui.win_ctx().delta_time_secs_f32();
                                } else if offset > input_text_max_x {
                                    self.input_text_offset_x +=
                                        ui.style().input_text_selection_scroll_speed() *
                                        ui.win_ctx().delta_time_secs_f32();
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
                                        ui.style().input_text_selection_scroll_speed() *
                                        ui.win_ctx().delta_time_secs_f32();
                                } else if offset < 0.0 {
                                    self.input_text_offset_x -=
                                        ui.style().input_text_selection_scroll_speed() *
                                        ui.win_ctx().delta_time_secs_f32();
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
                        if override_cursor {
                            ui.win_ctx().set_cursor(CursorIcon::Text);
                        }
                    }
                }
            }
            self.flags &= !Self::SELECT_ALL_LAST_FRAME;
            or_flag!(self.flags, Self::SELECT_ALL_LAST_FRAME, select_all || self.select_all());
            self.flags &= !Self::SELECT_ALL;
            self.flags &= !Self::CLICKED_LAST_FRAME;
            or_flag!(self.flags, Self::CLICKED_LAST_FRAME, mouse_left_state.pressed());
        }
        let deactivate =
            ui.win_ctx().key_state(KeyCode::Enter).pressed() ||
            ui.win_ctx().key_state(KeyCode::Escape).pressed() ||
            (mouse_left_state.pressed() && !reaction.held() && !reaction.hovered());
        if deactivate {
            self.flags &= !Self::ACTIVE;
            self.flags |= Self::MOUSE_VISIBLE;
        }
        if self.center_text() {
            let text_width =  if self.active() {
                ui.style().calc_text_width(self.input_text.as_ref().unwrap())
            } else {
                ui.style().calc_text_width(self.input_text_formatted.as_ref().unwrap())
            };
            if text_width + item_pad_inner.x + item_pad_inner.x > self.width {
                self.input_text_offset_x = 0.0;
                input_rect.max.x = ui.style().calc_text_box_width_from_text_width(text_width);
            } else {
                self.input_text_offset_x =
                    text_width * 0.5 - self.width * 0.5 + ui.style().item_pad_inner().x;
            }
        }
        if !self.active() {
            self.selection = None;
        }
        let mut selection_rect = Default::default();
        if let Some(selection) = self.selection {
            let left = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.0).x.clamp(0.0, input_text_max_x);
            let right = input_text_offset.x +
                self.calc_cursor_offset(font_scale, selection.1).x.clamp(0.0, input_text_max_x);
            let rect = BoundingRect::from_min_max(
                vec2(left, offset.y + item_pad_inner.y),
                vec2(right, offset.y + input_rect.max.y - item_pad_inner.y),
            );
            selection_rect = (rect.min, rect.max);
        } else {
            self.flags &= !Self::SELECTION_LEFT;
        }
        let reaction_id = reaction.id();
        let mut visuals = ui.style().interact_visuals(reaction);
        if self.active() {
            visuals.bg_strokes[0].col = ui.style().input_text_active_stroke_col();
            visuals.bg_stroke_idx = 0;
        }
        let fill_col =
            if self.has_bg_col_override() {
                self.bg_col_override
            } else {
                ui.style().input_text_bg_col()
            };
        let selection_bg_col = ui.style().input_text_selection_bg_col();
        let cursor_off = offset + item_pad_inner + self.calc_cursor_offset(font_scale, self.text_cursor_pos);
        let cursor_size =
            if self.cursor_visible() {
                vec2(ui.style().input_text_cursor_width(), text_height)
            } else {
                Default::default()
            };
        ui.paint(move |painter, row| {
            let off = vec2(0.0, row.height_halved - input_rect.max.y * 0.5);
            painter
                .rect(
                    reaction_id,
                    input_rect,
                    offset + off,
                    fill_col,
                    visuals.bg_strokes.clone(),
                    visuals.bg_stroke_idx
                )
                .flat_rect(
                    reaction_id,
                    selection_rect.0, selection_rect.1,
                    off,
                    selection_bg_col
                )
                .flat_rect(
                    reaction_id,
                    cursor_off,
                    cursor_off + cursor_size,
                    off,
                    text_col,
                );
        });
        self.double_click_timer += ui.win_ctx().delta_time_secs_f32();
        self.flags &= !Self::ACTIVATED_LAST_FRAME;
        or_flag!(self.flags, Self::ACTIVATED_LAST_FRAME, !active_this_frame && self.active());
        let text = Rc::new(RefCell::new(Text::new(
            if self.active() {
                self.input_text.clone().unwrap()
            } else {
                self.input_text_formatted.clone().unwrap()
            },
            GlobalVec::with_len(1, if self.active() { self.row_offsets.clone() } else { self.formatted_row_offsets.clone() }),
            if self.input.is_empty() {
                ui.style().input_text_empty_text_color()
            } else {
                text_col
            },
            offset + vec2(item_pad_inner.x - self.input_text_offset_x, 0.0),
            vec2(1.0, 1.0),
            None,
            0,
            1,
            Some(BoundingRect::from_min_max(offset + item_pad_inner, offset + input_rect.max - item_pad_inner)),
            None,
        )));
        ui.render_text(text);
        reaction.size = input_rect.max;
    }

    #[inline(always)]
    fn cursor_enabled(&self) -> bool {
        self.flags & Self::CURSOR_ENABLE == Self::CURSOR_ENABLE
    }

    #[inline(always)]
    pub fn active(&self) -> bool {
        self.flags & Self::ACTIVE == Self::ACTIVE
    }

    #[inline(always)]
    fn cursor_visible(&self) -> bool { self.flags & Self::CURSOR_VISIBLE == Self::CURSOR_VISIBLE }

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
    fn parent_active(&self) -> bool {
        self.flags & Self::PARENT_ACTIVE == Self::PARENT_ACTIVE
    }

    #[inline(always)]
    fn active_last_frame(&self) -> bool {
        self.flags & Self::ACTIVE_LAST_FRAME == Self::ACTIVE_LAST_FRAME
    }
}

pub struct InputText<Style> {
    offset: Vec2,
    input: CompactString,
    input_text: Option<RenderedText>,
    input_text_formatted: Option<RenderedText>,
    empty_input_prompt: CompactString,
    format_input: Box<fn(&mut dyn Write, &str) -> core::fmt::Result>,
    input_offsets: GlobalVec<Vec2>,
    input_text_offset_x: f32,
    selection: Option<(usize, usize)>,
    cursor_rect_vertex_range: Option<VertexRange>,
    text_cursor_pos: usize,
    input_rect: Rect,
    input_rect_vertex_range: Option<VertexRange>,
    input_rect_stroke_vertex_range: Option<VertexRange>,
    selection_rect_vertices: [Vertex; 4],
    cursor_rect: Rect,
    cursor_timer: f32,
    double_click_timer: f32,
    focused_stroke_thickness: f32,
    width: f32,
    bg_col_override: ColorSRGBA,
    flags: u32,
    _marker: PhantomData<Style>,
}

impl<Style> InputText<Style>
    where
        Style: UiStyle,
{

    const HOVERED: u32 = 0x1;
    const HELD: u32 = 0x2;
    const ACTIVE: u32 = 0x4;
    const CURSOR_VISIBLE: u32 = 0x8;
    const SELECTION_LEFT: u32 = 0x10;
    const MOUSE_VISIBLE: u32 = 0x20;
    const FORMAT_ERROR: u32 = 0x40;
    const CENTER_TEXT: u32 = 0x80;
    const CURSOR_ENABLE: u32 = 0x100;
    const SELECT_ALL: u32 = 0x200;
    const BG_COL_OVERRIDE: u32 = 0x400;
    const CLICKED_LAST_FRAME: u32 = 0x800;
    const SELECT_ALL_LAST_FRAME: u32 = 0x1000;
    const ACTIVATED_LAST_FRAME: u32 = 0x2000;
    const PARENT_ACTIVE: u32 = 0x4000;
    const ACTIVE_LAST_FRAME: u32 = 0x8000;

    const SELECTION_INDICES: [u32; 6] = [
        3, 1, 0,
        1, 3, 2,
    ];

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            offset: Default::default(),
            input_text: None,
            input_text_formatted: None,
            format_input: Box::new(|fmt, input| -> core::fmt::Result {
                write!(fmt, "{}", input)
            }),
            empty_input_prompt: Default::default(),
            input: Default::default(),
            input_offsets: Default::default(),
            input_text_offset_x: 0.0,
            text_cursor_pos: 0, 
            selection: None,
            input_rect: Default::default(),
            input_rect_vertex_range: None,
            input_rect_stroke_vertex_range: None,
            selection_rect_vertices: Default::default(),
            cursor_rect: Default::default(),
            cursor_rect_vertex_range: None,
            cursor_timer: 0.0,
            double_click_timer: 100.0,
            focused_stroke_thickness: 0.0,
            width: 0.0,
            bg_col_override: Default::default(),
            flags: Self::MOUSE_VISIBLE | Self::CURSOR_ENABLE,
            _marker: PhantomData,
        }
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
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    pub fn set_params(
        &mut self,
        width: f32,
        bg_col_override: Option<ColorSRGBA>,
        center_text: bool,
        empty_input_prompt: &str,
        format_input: Option<fn(&mut dyn Write, &str) -> core::fmt::Result>,
        parent_active: bool,
    )
    {
        self.flags &= !(
            Self::CENTER_TEXT |
            Self::BG_COL_OVERRIDE |
            Self::PARENT_ACTIVE
        );
        or_flag!(self.flags, Self::CENTER_TEXT, center_text);
        or_flag!(self.flags, Self::PARENT_ACTIVE, parent_active);
        self.width = width;
        if let Some(bg_col_override) = bg_col_override {
            self.bg_col_override = bg_col_override;
            self.flags |= Self::BG_COL_OVERRIDE;
        }
        if let Some(format) = format_input {
            *self.format_input = format;
        }
        if empty_input_prompt != self.empty_input_prompt {
            self.empty_input_prompt = CompactString::new(empty_input_prompt);
        }
    }

    #[inline(always)]
    pub fn set_input_sliderable<T>(
        &mut self,
        style: &Style,
        input: &T
    )
        where 
            T: Sliderable,
    {
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
    pub fn rel_bounding_rect(&self, style: &Style) -> BoundingRect {
        let item_pad_inner = style.item_pad_inner();
        BoundingRect::from_position_size(
            self.offset + vec2(item_pad_inner.x, 0.0),
            self.input_rect.max - vec2(item_pad_inner.x + item_pad_inner.x, 0.0),
        )
    }

    #[inline(always)]
    pub fn set_hovered(&mut self, value: bool) {
        self.flags &= !Self::HOVERED;
        or_flag!(self.flags, Self::HOVERED, value);
    }

    #[inline(always)]
    pub fn set_cursor_enable(&mut self, value: bool) {
        self.flags &= !Self::CURSOR_ENABLE;
        or_flag!(self.flags, Self::CURSOR_ENABLE, value);
    }

    #[inline(always)]
    pub fn hide(&mut self, vertices: &mut [Vertex]) {
        hide_vertices(vertices, self.input_rect_vertex_range);
    }

    #[inline(always)]
    fn cursor_enabled(&self) -> bool {
        self.flags & Self::CURSOR_ENABLE == Self::CURSOR_ENABLE
    }

    #[inline(always)]
    fn hovered(&self) -> bool {
        self.flags & Self::HOVERED == Self::HOVERED
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
    fn parent_active(&self) -> bool {
        self.flags & Self::PARENT_ACTIVE == Self::PARENT_ACTIVE
    }

    #[inline(always)]
    fn active_last_frame(&self) -> bool {
        self.flags & Self::ACTIVE_LAST_FRAME == Self::ACTIVE_LAST_FRAME
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

impl<Style> Widget<Style> for InputText<Style>
    where
        Style: UiStyle,
{

    fn get_offset(&self) -> Vec2 {
        self.offset
    }

    #[inline(always)]
    fn set_offset(&mut self, offset: Vec2)
    {
        self.offset = offset;
    }

    fn set_scroll_offset(&mut self, offset: Vec2) {
        self.offset += offset;
    }

    #[inline(always)]
    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut TextRenderer,
    ) -> Vec2
    {
        let item_pad_inner = style.item_pad_inner();
        let width =
            if self.center_text() {
                let input_width =
                    if let Some(input_text) = self.input_text.as_ref() {
                        style.calc_text_width(input_text)
                    } else {
                        0.0
                    };
                (input_width + item_pad_inner.x + item_pad_inner.x).max(self.width)
            } else {
                self.width
            };
        vec2(
            width,
            style.calc_text_box_height_from_text_height(style.calc_font_height(text_renderer)),
        )
    }

    fn status<'a>(
        &'a self,
        _ctx: &WindowCtx,
        _style: &Style,
        _window_pos: Vec2,
        _cursor_pos: Vec2
    ) -> WidgetStatus<'a>
    {
        if !self.mouse_visible() || self.held() {
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
        _cursor_in_this_window: bool,
        other_widget_active: bool,
        _cursor_in_other_widget: bool,
        window_moving: bool,
        hover_blocked: bool,
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
            let mut cursor_timer = self.cursor_timer + ctx.delta_time().as_secs_f32();
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
                let input_empty = ctx.get_input_text().0 == 0;
                if ctx.key_state(KeyCode::ControlLeft).held() {
                    if ctx.key_state(KeyCode::KeyV).pressed() && let Some(text) = ctx.get_clipboard() {
                        let start_count = self.input.char_indices().count();
                        for i in (selection.0..selection.1).rev() {
                            let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                            self.input.remove(index); 
                        }
                        let mut text_cursor_pos = selection.0;
                        self.input.insert_str(
                            self.input
                                .char_indices()
                                .skip(text_cursor_pos)
                                .next()
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| self.input.len()),
                            &text,
                        );
                        text_cursor_pos += text.char_indices().count();
                        self.text_cursor_pos = text_cursor_pos;
                        self.input_text = None;
                        let end_count = self.input.char_indices().count();
                        if start_count > end_count {
                            cursor_move = CursorMove::Backspace;
                        } else {
                            cursor_move = CursorMove::Right;
                        }
                    } else if ctx.key_state(KeyCode::KeyC).pressed() {
                        let mut text = CompactString::default();
                        let mut iter = self.input.char_indices().skip(selection.0);
                        for _ in selection.0..selection.1  {
                            text.push(iter.next().unwrap().1);
                        }
                        ctx.set_clipboard(&text);
                        self.selection = Some(selection);
                    } else if ctx.key_state(KeyCode::KeyX).pressed() {
                        let mut text = CompactString::default();
                        let mut iter = self.input.char_indices().skip(selection.0);
                        for _ in selection.0..selection.1  {
                            text.push(iter.next().unwrap().1);
                        }
                        ctx.set_clipboard(&text);
                        for i in (selection.0..selection.1).rev() {
                            let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                            self.input.remove(index);
                        }
                        self.text_cursor_pos = selection.0;
                        self.input_text = None;
                        cursor_move = CursorMove::Backspace;
                    } else {
                        self.selection = Some(selection);
                    }
                }
                else if ctx.key_state(KeyCode::Backspace).pressed() || !input_empty {
                    let start_count = self.input.char_indices().count();
                    for i in (selection.0..selection.1).rev() {
                        let (index, _) = self.input.char_indices().skip(i).next().unwrap();
                        self.input.remove(index);
                    }
                    let mut text_cursor_pos = selection.0;
                    for text in ctx.get_input_text().1 {
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
                    } else {
                        cursor_move = CursorMove::Right;
                    }
                    self.input_text = None;
                } else if ctx.key_state(KeyCode::ArrowLeft).pressed() {
                    if ctx.key_state(KeyCode::ShiftLeft).held() {
                        if self.selection_left() {
                            if selection.0 != 0 {
                                selection.0 -= 1;
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
                } else if ctx.key_state(KeyCode::ArrowRight).pressed() {
                    if ctx.key_state(KeyCode::ShiftLeft).held() {
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
                    if ctx.key_state(KeyCode::Backspace).pressed() {
                        let remove = text_cursor_pos - 1;
                        let (index, _) = self.input.char_indices().skip(remove).next().unwrap();
                        self.input.remove(index);
                        self.input_text = None;
                        text_cursor_pos = remove;
                    } else if ctx.key_state(KeyCode::ArrowLeft).pressed() {
                        if ctx.key_state(KeyCode::ShiftLeft).held() {
                            self.selection = Some((text_cursor_pos - 1, text_cursor_pos));
                            self.flags |= Self::SELECTION_LEFT;
                        }
                        text_cursor_pos -= 1;
                        self.cursor_timer = 0.0;
                        self.flags |= Self::CURSOR_VISIBLE;
                    }
                }
                if ctx.key_state(KeyCode::ControlLeft).held()
                {
                    if ctx.key_state(KeyCode::KeyV).pressed() && let Some(text) = ctx.get_clipboard() {
                        self.input.insert_str(
                            self.input
                                .char_indices()
                                .skip(text_cursor_pos)
                                .next()
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| self.input.len()),
                            &text,
                        );
                        text_cursor_pos += text.char_indices().count();
                        self.input_text = None;
                    }
                } else {
                    let input = ctx.get_input_text();
                    if input.0 != 0 {
                        for text in input.1 {
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
                }
                let end_count = self.input.char_indices().count();
                if ctx.key_state(KeyCode::ArrowRight).pressed() {
                    text_cursor_pos = (text_cursor_pos + 1).clamp(0, end_count);
                    if text_cursor_pos != end_count && ctx.key_state(KeyCode::ShiftLeft).held() {
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
        let item_pad_inner = style.item_pad_inner();
        let has_format_error = self.has_format_error();
        let text_col =
            if self.active() || self.parent_active() {
                style.active_text_col()
            } else if self.hovered() {
                style.focused_text_col()
            } else {
                style.inactive_text_col()
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
                    0.0,
                    |offset| {
                        self.input_offsets.push(vec2(offset.offset[0], offset.offset[1]) * font_scale);
                    },
                )
                .unwrap_or_default()
        });
        let input_text = unsafe {
            self.input_text
                .as_ref()
                .unwrap_unchecked()
        };
        let text_height = style.calc_font_height(text_renderer);
        let offset = self.offset;
        let input_width = style.calc_text_width(input_text);
        let width =
            if self.center_text() {
                (input_width + item_pad_inner.x + item_pad_inner.x).max(self.width)
            } else {
                self.width
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
        let input_text_offset = offset + item_pad_inner;
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
        self.flags &= !Self::HOVERED;
        let mut cursor_in_widget = !mouse_visible;
        let mouse_left_state = ctx.mouse_button_state(MouseButton::Left);
        let rel_cursor_pos = cursor_pos - window_pos;
        let error_margin = style.cursor_error_margin();
        if override_cursor {
            if self.active() {
                ctx.set_cursor_hide(!mouse_visible);
            } else if self.active_last_frame() {
                ctx.set_cursor_hide(false);
            }
        }
        self.flags &= !Self::ACTIVE_LAST_FRAME;
        or_flag!(self.flags, Self::ACTIVE_LAST_FRAME, self.active());
        if !other_widget_active {
            if mouse_visible && self.cursor_enabled() {
                let cursor_in_input = BoundingRect::from_position_size(
                    offset - vec2(error_margin, 0.0),
                    input_rect.max + vec2(error_margin + error_margin, 0.0)
                ).is_point_inside(rel_cursor_pos);
                or_flag!(self.flags, Self::HOVERED, cursor_in_input && !hover_blocked);
                let input_min = offset.x + item_pad_inner.x - self.input_text_offset_x;
                let input_min_max = (input_min, input_min + input_width);
                let input_box_min_max =
                    (offset.x, offset.x + input_rect.max.x - item_pad_inner.x);
                let mut select_all = false;
                if self.hovered() {
                    cursor_in_widget = true;
                    if mouse_left_state.pressed() {
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
                        ctx.set_cursor(CursorIcon::Text);
                    }
                }
                select_all &= self.selection.is_none();
                if select_all || self.select_all() {
                    self.flags |= Self::ACTIVE;
                    self.selection = Some((0, self.input_offsets.len()));
                    self.flags |= Self::CURSOR_VISIBLE;
                    self.cursor_timer = 0.0;
                }
                if !self.hovered() && !self.held() && !window_moving && mouse_left_state.released() {
                    self.flags &= !Self::ACTIVE;
                    self.flags |= Self::MOUSE_VISIBLE;
                }
                if !select_all && !window_moving {
                    if self.clicked_last_frame() &&
                        !self.select_all_last_frame() &&
                        !self.activated_last_frame()
                    {
                        self.selection = None;
                    }
                    if self.held() {
                        if mouse_left_state.released() {
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
                                            ctx.delta_time_secs_f32();
                                    } else if offset > input_text_max_x {
                                        self.input_text_offset_x +=
                                            style.input_text_selection_scroll_speed() *
                                            ctx.delta_time_secs_f32();
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
                                            ctx.delta_time_secs_f32();
                                    } else if offset < 0.0 {
                                        self.input_text_offset_x -=
                                            style.input_text_selection_scroll_speed() *
                                            ctx.delta_time_secs_f32();
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
                                ctx.set_cursor(CursorIcon::Text);
                            }
                        }
                    }
                }
                self.flags &= !Self::SELECT_ALL_LAST_FRAME;
                or_flag!(self.flags, Self::SELECT_ALL_LAST_FRAME, select_all || self.select_all());
                self.flags &= !Self::SELECT_ALL;
                self.flags &= !Self::CLICKED_LAST_FRAME;
                or_flag!(self.flags, Self::CLICKED_LAST_FRAME, mouse_left_state.pressed());
            }
            let deactivate =
                ctx.key_state(KeyCode::Enter).pressed() |
                ctx.key_state(KeyCode::Escape).pressed();
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
            if text_width + item_pad_inner.x + item_pad_inner.x > self.width {
                self.input_text_offset_x = 0.0;
                input_rect.max.x = style.calc_text_box_width_from_text_width(text_width);
            } else {
                self.input_text_offset_x =
                    text_width * 0.5 - self.width * 0.5 + style.item_pad_inner().x;
            }
        }
        let cursor_rect_max = vec2(style.input_text_cursor_width(), text_height);
        let requires_triangulation =
            self.input_rect != input_rect ||
            self.cursor_rect.max != cursor_rect_max ||
            self.focused_stroke_thickness != style.focused_widget_stroke_thickness();
        self.input_rect = input_rect;
        self.cursor_rect.max = cursor_rect_max;
        self.focused_stroke_thickness = style.focused_widget_stroke_thickness();
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
        self.double_click_timer += ctx.delta_time_secs_f32();
        self.flags &= !Self::ACTIVATED_LAST_FRAME;
        or_flag!(self.flags, Self::ACTIVATED_LAST_FRAME, !active_this_frame && self.active());
        let input_bounding_rect = BoundingRect::from_position_size(
            window_pos + self.offset + vec2(item_pad_inner.x, 0.0),
            self.input_rect.max - vec2(item_pad_inner.x + item_pad_inner.x, 0.0),
        );
        collect_text(
            if self.active() {
                self.input_text.as_ref().unwrap()
            } else {
                self.input_text_formatted.as_ref().unwrap()
            },
            self.offset + item_pad_inner - vec2(self.input_text_offset_x, 0.0), 
            BoundedTextInstance {
                add_scale: vec2(1.0, 1.0),
                min_bounds: input_bounding_rect.min.max(window_pos + content_offset),
                max_bounds: input_bounding_rect.max.min(window_pos + window_size),
                color:
                    if self.input.is_empty() {
                        style.input_text_empty_text_color()
                    } else {
                        text_col
                    },
            }
        );
        UpdateResult {
            requires_triangulation,
            requires_transfer_commands: false,
            cursor_in_widget,
        }
    }

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> Option<VertexRange>,
    ) {
        self.input_rect.to_points(&mut |p| { points.push(p.into()); });
        outline_points(
            points, self.focused_stroke_thickness, false,
            &mut |p| { helper_points.push(p.into()); }
        );
        self.input_rect_stroke_vertex_range = tri(&helper_points);
        self.input_rect_vertex_range = tri(points);
        points.clear();
        self.cursor_rect.to_points(&mut |p| { points.push(p.into()); });
        self.cursor_rect_vertex_range = tri(points);
    }

    fn set_vertex_params(
        &mut self,
        style: &Style,
        vertices: &mut [shaders::Vertex],
    )
    {
        let mut offset = self.offset;
        let target_color = if self.has_bg_col_override() {
            self.bg_col_override
        } else {
            style.input_text_bg_col()
        };
        set_vertex_params(vertices, self.input_rect_vertex_range, offset, target_color);
        if self.active() {
            let target_color = style.input_text_active_stroke_col();
            set_vertex_params(vertices, self.input_rect_stroke_vertex_range, offset, target_color);
        } else if self.hovered() {
            let target_color = style.focused_widget_stroke_col();
            set_vertex_params(vertices, self.input_rect_stroke_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.input_rect_stroke_vertex_range);
        }
        if self.cursor_visible() {
            offset += style.item_pad_inner() +
                self.calc_cursor_offset(style.font_scale(), self.text_cursor_pos);
            let target_color = style.focused_text_col();
            set_vertex_params(vertices, self.cursor_rect_vertex_range, offset, target_color);
        } else {
            hide_vertices(vertices, self.cursor_rect_vertex_range);
        }
    }

    fn hide(
        &mut self,
        vertices: &mut [shaders::Vertex],
        _window_semaphore: (TimelineSemaphoreId, u64),
        _global_resources: &mut GlobalResources,
        _tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error>
    {
        self.hide(vertices);
        Ok(())
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
    ) -> Result<Option<&dyn HoverContents<Style>>, Error>
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
            render_commands.bind_pipeline(base_pipeline)?;
            let pc_vertex = push_constants_vertex(window_pos, vec2(1.0, 1.0), inv_aspect_ratio, unit_scale);
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
