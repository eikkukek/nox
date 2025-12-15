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

use crate::{
    export::mem::vec_types::GlobalVec,
    clipboard::Clipboard,
    dev::or_flag,
};

use super::*;

pub use winit::window::WindowButtons;
pub use winit::window::Icon as WindowIcon;
pub use winit::keyboard::KeyCode;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;
pub use winit::monitor::MonitorHandle;

use winit::dpi::LogicalSize;

#[derive(Clone)]
pub struct WindowAttributes {
    title: String,
    size: [u32; 2],
    enabled_buttons: win::WindowButtons,
    icon: Option<win::WindowIcon>,
    attr: u32,
}

impl WindowAttributes {

    const RESIZABLE: u32 = 0x1;
    const MAXIMIZED: u32 = 0x2;
    const TRANSPARENT: u32 = 0x4;
    const BLUR: u32 = 0x8;
    const DECORATIONS: u32 = 0x10;

    /// Creates [`WindowAttributes`] with default values.
    #[inline(always)]
    pub fn new(
        title: &str,
        size: [u32; 2],
    ) -> Self
    {
        Self {
            title: String::from(title),
            size,
            enabled_buttons: win::WindowButtons::all(),
            icon: None,
            attr: Self::DECORATIONS,
        }
    }

    /// Sets whether the window is resizeble or not.
    ///
    /// The default is `false`.
    #[inline(always)]
    pub fn with_resizable(mut self, value: bool) -> Self {
        self.attr &= !Self::RESIZABLE;
        or_flag!(self.attr, Self::RESIZABLE, value);
        self
    }

    /// Requests that the window is maximized upon creation.
    ///
    /// The default is `false`
    #[inline(always)]
    pub fn with_maximized(mut self, value: bool) -> Self {
        self.attr &= !Self::MAXIMIZED;
        or_flag!(self.attr, Self::MAXIMIZED, value);
        self
    }

    /// Sets whether the background of the window should be transparent.
    ///
    /// If this is set to `true`, alpha values different than `1.0` on the swapchain image will
    /// produce a transparent window.
    ///
    /// The default is `false`.
    #[inline(always)]
    pub fn with_transparent(mut self, value: bool) -> Self {
        self.attr &= !Self::TRANSPARENT;
        or_flag!(self.attr, Self::TRANSPARENT, value);
        self
    }

    /// Sets whether the background of the window should be blurred.
    ///
    /// The default is `false`.
    #[inline(always)]
    pub fn with_blur(mut self, value: bool) -> Self {
        self.attr &= !Self::BLUR;
        or_flag!(self.attr, Self::BLUR, value);
        self
    }

    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// The default is `false`.
    #[inline(always)]
    pub fn with_decorations(mut self, value: bool) -> Self {
        self.attr &= !Self::DECORATIONS;
        or_flag!(self.attr, Self::DECORATIONS, value);
        self
    }

    /// Sets the enabled window buttons
    ///
    /// The default is [`WindowButtons::all`].
    ///
    /// Primarily effective on Windows systems.
    #[inline(always)]
    pub fn with_enabled_buttons(mut self, buttons: win::WindowButtons) -> Self {
        self.enabled_buttons = buttons;
        self
    }

    /// Sets the window icon.
    ///
    /// The default is `None`
    #[inline(always)]
    pub fn with_window_icon(mut self, icon: impl Into<Option<win::WindowIcon>>) -> Self {
        self.icon = icon.into();
        self
    }

    #[inline(always)]
    pub fn resizable(&self) -> bool {
        self.attr & Self::RESIZABLE == Self::RESIZABLE
    }

    #[inline(always)]
    pub fn maximized(&self) -> bool {
        self.attr & Self::MAXIMIZED == Self::MAXIMIZED
    }

    #[inline(always)]
    pub fn transparent(&self) -> bool {
        self.attr & Self::TRANSPARENT == Self::TRANSPARENT
    }

    #[inline(always)]
    pub fn blur(&self) -> bool {
        self.attr & Self::BLUR == Self::BLUR
    }

    #[inline(always)]
    pub fn decorations(&self) -> bool {
        self.attr & Self::DECORATIONS == Self::DECORATIONS
    }

    #[inline(always)]
    pub fn enabled_buttons(&self) -> win::WindowButtons {
        self.enabled_buttons
    }

    #[inline(always)]
    pub fn icon(&self) -> Option<&win::WindowIcon> {
        self.icon.as_ref()
    }

    #[inline(always)]
    pub(crate) fn to_winit_attr(self) -> winit::window::WindowAttributes {
        Window::default_attributes()
            .with_resizable(self.resizable())
            .with_maximized(self.maximized())
            .with_transparent(self.transparent())
            .with_blur(self.blur())
            .with_decorations(self.decorations())
            .with_enabled_buttons(self.enabled_buttons)
            .with_window_icon(self.icon)
            .with_title(self.title)
            .with_inner_size(LogicalSize::new(
                self.size[0],
                self.size[1],
            ))
            .with_min_inner_size(LogicalSize::new(1.0, 1.0))
    }
}

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

pub struct WindowContext {
    pub(super) window: Option<Arc<Window>>,
    pub(super) monitors: GlobalVec<MonitorHandle>,
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

impl WindowContext {

    pub(super) const CURSOR_MOVED: u32 = 0x1;
    pub(super) const CURSOR_SET: u32 = 0x2;
    pub(super) const TRANSPARENT: u32 = 0x4;
    pub(super) const TRANSPARENT_SET: u32 = 0x8;

    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            monitors: Default::default(),
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
    pub fn monitors(&self) -> &[MonitorHandle] {
        &self.monitors
    }

    #[inline(always)]
    pub fn is_transparent(&self) -> bool {
        self.flags & Self::TRANSPARENT == Self::TRANSPARENT
    }

    #[inline(always)]
    pub fn set_transparent(&mut self, value: bool) {
        self.flags &= !Self::TRANSPARENT;
        self.flags |= Self::TRANSPARENT_SET;
        or_flag!(self.flags, Self::TRANSPARENT, value);
    }

    #[inline(always)]
    pub(super) fn transparent_set(&self) -> bool {
        self.flags & Self::TRANSPARENT_SET == Self::TRANSPARENT_SET
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

    pub(super) fn reset_input(&mut self) {
        self.mouse_scroll_pixel_delta = (0.0, 0.0); 
        self.mouse_scroll_line_delta = (0.0, 0.0);
        self.physical_keys.retain(|_, v| {
            v.pressed = false;
            v.released = false;
            v.held
        });
        self.logical_keys.retain(|_, v| {
            v.pressed = false;
            v.released = false;
            v.held
        });
        self.mouse_buttons.retain(|_, v| {
            v.pressed = false;
            v.released = false;
            v.held
        });
    }
}
