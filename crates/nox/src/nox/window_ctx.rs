use std::{
    sync::Arc,
    time,
};


use winit::{
    window::Window,
    keyboard::{PhysicalKey, Key},
};

use rustc_hash::FxHashMap;

use compact_str::CompactString;

use mem::vec_types::GlobalVec;

use crate::{
    clipboard::Clipboard,
    *
};

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub(super) pressed: bool,
    pub(super) released: bool,
    pub(super) held: bool,
    pub(super) repeat: bool,
}

impl InputState {

    #[inline(always)]
    pub fn pressed(self) -> bool {
        self.pressed
    }

    #[inline(always)]
    pub fn released(self) -> bool {
        self.released
    }

    #[inline(always)]
    pub fn held(self) -> bool {
        self.held
    }

    #[inline(always)]
    pub fn repeat(self) -> bool {
        self.repeat
    }
}

pub struct WindowCtx {
    pub(super) window: Option<Arc<Window>>,
    pub(super) physical_keys: FxHashMap<PhysicalKey, InputState>,
    pub(super) logical_keys: FxHashMap<Key, InputState>,
    pub(super) mouse_buttons: FxHashMap<MouseButton, InputState>,
    pub(super) input_text: GlobalVec<(KeyCode, CompactString)>,
    pub(super) clipboard: Clipboard,
    pub(super) window_size: (u32, u32),
    pub(super) cursor_position: (f64, f64),
    pub(super) mouse_scroll_pixel_delta: (f64, f64),
    pub(super) mouse_scroll_line_delta: (f32, f32),
    pub(super) current_cursor: CursorIcon,
    pub(super) delta_counter: time::Instant,
    pub(super) delta_time: time::Duration,
    pub(super) flags: u32,
}

impl WindowCtx {

    pub(super) const CURSOR_MOVED: u32 = 0x1;
    pub(super) const CURSOR_SET: u32 = 0x2;

    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            physical_keys: FxHashMap::default(),
            logical_keys: FxHashMap::default(),
            mouse_buttons: FxHashMap::default(),
            input_text: Default::default(),
            window_size: (0, 0),
            cursor_position: (0.0, 0.0),
            clipboard: Clipboard::None,
            mouse_scroll_pixel_delta: (0.0, 0.0),
            mouse_scroll_line_delta: (0.0, 0.0),
            current_cursor: CursorIcon::Default,
            delta_counter: time::Instant::now(),
            delta_time: Default::default(),
            flags: 0,
        }
    }

    #[inline(always)]
    pub fn delta_time(&self) -> time::Duration {
        self.delta_time
    }

    #[inline(always)]
    pub fn delta_time_secs_f32(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    #[inline(always)]
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    #[inline(always)]
    pub fn window_size_f32(&self) -> (f32, f32) {
        (self.window_size.0 as f32, self.window_size.1 as f32)
    }

    #[inline(always)]
    pub fn aspect_ratio(&self) -> f64 {
        self.window_size.0 as f64 /
        self.window_size.1 as f64
    }

    #[inline(always)]
    pub fn cursor_moved(&self) -> bool {
        self.flags & Self::CURSOR_MOVED == Self::CURSOR_MOVED
    }

    #[inline(always)]
    pub(super) fn cursor_set(&self) -> bool {
        self.flags & Self::CURSOR_SET == Self::CURSOR_SET
    }

    #[inline(always)]
    pub fn set_cursor_hide(&self, hide: bool) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor_visible(!hide);
        }
    }

    #[inline(always)]
    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.current_cursor = cursor;
        self.flags |= Self::CURSOR_SET;
    }

    #[inline(always)]
    pub fn get_clipboard(&self) -> Option<String> {
        self.clipboard.get()
    }

    #[inline(always)]
    pub fn set_clipboard(&mut self, contents: &str) {
        self.clipboard.set(contents);
    }

    #[inline(always)]
    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_position
    }

    #[inline(always)]
    pub fn normalized_cursor_position(&self) -> (f64, f64) {
        (
            self.cursor_position.0 / self.window_size.0 as f64,
            self.cursor_position.1 / self.window_size.1 as f64,
        )
    }

    #[inline(always)]
    pub fn normalized_cursor_position_f32(&self) -> (f32, f32) {
        (
            self.cursor_position.0 as f32 / self.window_size.0 as f32,
            self.cursor_position.1 as f32 / self.window_size.1 as f32,
        )
    }

    #[inline(always)]
    pub fn mouse_scroll_pixel_delta(&self) -> (f64, f64) {
        self.mouse_scroll_pixel_delta
    }

    #[inline(always)]
    pub fn mouse_scroll_delta_lines(&self) -> (f32, f32) {
        self.mouse_scroll_line_delta
    }

    #[inline(always)]
    pub fn key_state(&self, key: KeyCode) -> InputState {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .copied()
            .unwrap_or_default()
    }

    #[inline(always)]
    pub fn key_value(&self, key: KeyCode) -> f32 {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .copied()
            .unwrap_or_default()
            .held as u32 as f32
    }

    #[inline(always)]
    pub fn get_input_text(&self) -> (usize, impl Iterator<Item = (KeyCode, &str)>) {
        (
            self.input_text.len(),
            self.input_text
                .iter()
                .map(|(key, text)| (*key, text.as_str()))
        )
    }

    #[inline(always)]
    pub fn mouse_button_state(&self, button: MouseButton) -> InputState {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
    }

    #[inline(always)]
    pub fn mouse_button_value(&self, button: MouseButton) -> f32 {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
            .held as u32 as f32
    }
}
