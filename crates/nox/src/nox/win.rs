use std::time;

use core::ops::{Deref, DerefMut};

use winit::{
    keyboard::{PhysicalKey, Key},
    event_loop::ActiveEventLoop,
};

use rustc_hash::FxHashMap;

use compact_str::CompactString;

use crate::{
    export::mem::vec_types::GlobalVec,
    clipboard::Clipboard,
    dev::or_flag,
};

use super::*;

pub use winit::window::WindowId;
pub use winit::window::WindowButtons;
pub use winit::window::Icon as WindowIcon;
pub use winit::keyboard::KeyCode;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;
pub use winit::monitor::MonitorHandle;

use winit::dpi::LogicalSize;

use crate::dev::error::Result;

#[derive(Clone)]
pub struct NoxWindowAttributes {
    title: String,
    size: [u32; 2],
    enabled_buttons: win::WindowButtons,
    icon: Option<win::WindowIcon>,
    attr: u32,
}

impl NoxWindowAttributes {

    const RESIZABLE: u32 = 0x1;
    const MAXIMIZED: u32 = 0x2;
    const TRANSPARENT: u32 = 0x4;
    const BLUR: u32 = 0x8;
    const DECORATIONS: u32 = 0x10;

    /// Creates [`WindowAttributes`] with default values.
    #[inline(always)]
    fn new() -> Self
    {
        Self {
            title: String::new(),
            size: [100, 100],
            enabled_buttons: win::WindowButtons::all(),
            icon: None,
            attr: Self::DECORATIONS,
        }
    }

    /// Sets the `title` of the window.
    ///
    /// The default is an empty string.
    #[inline(always)]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Sets the size of the window's content area.
    ///
    /// The default is `[100, 100]`
    #[inline(always)]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = [width, height];
        self
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

pub struct NoxWindow<'a> {
    pub(super) handle: Window,
    host_allocators: &'a gpu::HostAllocators,
    buffered_frames: u32,
    surface: gpu::Surface<'a>,
    physical_keys: FxHashMap<PhysicalKey, InputState>,
    logical_keys: FxHashMap<Key, InputState>,
    mouse_buttons: FxHashMap<MouseButton, InputState>,
    pub(super) input_text: GlobalVec<(KeyCode, CompactString)>,
    clipboard: Clipboard,
    size: (u32, u32),
    cursor_position: (f64, f64),
    mouse_scroll_pixel_delta: (f64, f64),
    mouse_scroll_line_delta: (f32, f32),
    pub(super) current_cursor: CursorIcon,
    last_frame_data: gpu::FrameData,
    pub(super) flags: u32,
}

impl<'a> NoxWindow<'a> {

    pub(super) const CURSOR_MOVED: u32 = 0x1;
    pub(super) const CURSOR_SET: u32 = 0x2;
    pub(super) const TRANSPARENT: u32 = 0x4;
    pub(super) const TRANSPARENT_SET: u32 = 0x8;
    pub(super) const SHOULD_CLOSE: u32 = 0x10;

    pub fn default_attributes() -> NoxWindowAttributes {
        NoxWindowAttributes::new()
    }

    fn new(
        gpu: &mut gpu::GpuContext,
        window: Window,
        is_transparent: bool,
        host_allocators: &'a gpu::HostAllocators,
    ) -> Result<Self>
    {
        let clipboard = Clipboard
            ::new(&window)
            .context("failed to create clipboard")?;
        let surface = gpu.create_surface(&window, host_allocators)?;
        let size = window.inner_size();
        let mut flags = 0;
        or_flag!(flags, Self::TRANSPARENT, is_transparent);
        Ok(Self {
            handle: window,
            host_allocators,
            buffered_frames: gpu.buffered_frames(),
            surface,
            physical_keys: FxHashMap::default(),
            logical_keys: FxHashMap::default(),
            mouse_buttons: FxHashMap::default(),
            input_text: Default::default(),
            clipboard,
            size: (size.width, size.height),
            cursor_position: (0.0, 0.0),
            mouse_scroll_pixel_delta: (0.0, 0.0),
            mouse_scroll_line_delta: (0.0, 0.0),
            current_cursor: CursorIcon::Default,
            last_frame_data: Default::default(),
            flags,
        })
    }

    #[inline(always)]
    fn should_close(&self) -> bool {
        self.flags & Self::SHOULD_CLOSE == Self::SHOULD_CLOSE
    }

    #[inline(always)]
    pub(crate) fn surface(&mut self) -> &mut gpu::Surface<'a> {
        &mut self.surface
    }
    
    #[inline(always)]
    pub(crate) fn update_frame_data(&mut self, frame_data: gpu::FrameData) {
        self.last_frame_data = frame_data;
    }

    #[inline(always)]
    pub(crate) fn last_frame_data(&self) -> gpu::FrameData {
        self.last_frame_data
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
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    #[inline(always)]
    pub fn size_f32(&self) -> (f32, f32) {
        (self.size.0 as f32, self.size.1 as f32)
    }

    #[inline(always)]
    pub fn aspect_ratio(&self) -> f64 {
        self.size.0 as f64 /
        self.size.1 as f64
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
        self.handle.set_cursor_visible(!hide);
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
            self.cursor_position.0 / self.size.0 as f64,
            self.cursor_position.1 / self.size.1 as f64,
        )
    }

    #[inline(always)]
    pub fn normalized_cursor_position_f32(&self) -> (f32, f32) {
        (
            self.cursor_position.0 as f32 / self.size.0 as f32,
            self.cursor_position.1 as f32 / self.size.1 as f32,
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

    pub fn process_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.cursor_position = (position.x, position.y);
                self.flags |= Self::CURSOR_MOVED;
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_scroll_line_delta = (x, y);
                    },
                    MouseScrollDelta::PixelDelta(d) => {
                        self.mouse_scroll_pixel_delta = (d.x, d.y);
                    }
                };
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event {
                    KeyEvent { physical_key, logical_key, text, location: _, state, repeat, .. } => {
                        let phys = self.physical_keys
                            .entry(physical_key)
                            .or_default();
                        phys.pressed = state == ElementState::Pressed;
                        phys.released = state == ElementState::Released;
                        phys.held = state != ElementState::Released;
                        phys.repeat = repeat;
                        if let Some(text) = text && (phys.pressed || repeat) {
                            self.input_text.push((
                                match physical_key {
                                    PhysicalKey::Code(c) => c,
                                    PhysicalKey::Unidentified(_) => KeyCode::Backspace,
                                },
                                CompactString::new(&text))
                            );
                        }
                        let logic = self.logical_keys.entry(logical_key).or_default();
                        logic.pressed = state == ElementState::Pressed;
                        logic.released = state == ElementState::Released;
                        logic.held = state != ElementState::Released;
                        logic.repeat = repeat;
                    },
                };
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let button = self.mouse_buttons.entry(button).or_default();
                button.pressed = state == ElementState::Pressed;
                button.released = state == ElementState::Released;
                button.held = state != ElementState::Released;
            },
            WindowEvent::CloseRequested => {
                self.flags |= Self::SHOULD_CLOSE;
            },
            WindowEvent::Resized(size) => {
                self.size = (size.width, size.height);
                self.surface.request_swapchain_update(
                    self.buffered_frames,
                    size,
                );
            },
            _ => {},
        }
    }

    fn reset_input(&mut self) {
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

impl<'a> Drop for NoxWindow<'a> {

    fn drop(&mut self) {
        self.surface.clean_up(self.host_allocators);
    }
}

pub struct NoxWindowRef<'a, 'b> {
    window: &'a NoxWindow<'b>,
    delta_time: time::Duration,
}

pub struct NoxWindowRefMut<'a, 'b> {
    window: &'a mut NoxWindow<'b>,
    delta_time: time::Duration,
}

pub struct WindowStorage<'a> {
    windows: FxHashMap<WindowId, NoxWindow<'a>>,
    active_ids: GlobalVec<WindowId>,
    pub(super) monitors: GlobalVec<MonitorHandle>,
    pub(super) delta_counter: time::Instant,
    pub(super) delta_time: time::Duration,
}

impl<'a> WindowStorage<'a> {

    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            windows: FxHashMap::default(),
            active_ids: Default::default(),
            monitors: Default::default(),
            delta_counter: time::Instant::now(),
            delta_time: Default::default(),
        }
    }

    #[inline(always)]
    pub fn monitors(&self) -> &[MonitorHandle] {
        &self.monitors
    }

    #[inline(always)]
    pub(crate) fn window_iter_mut(&mut self) -> impl Iterator<Item = (&WindowId, &mut NoxWindow<'a>)> {
        self.windows
            .iter_mut()
    }

    #[inline(always)]
    pub fn window(&self, id: WindowId) -> Option<NoxWindowRef<'_, 'a>> {
        Some(NoxWindowRef {
            window: self.windows.get(&id)?,
            delta_time: self.delta_time,
        })
    }

    #[inline(always)]
    pub fn window_mut(&mut self, id: WindowId) -> Option<NoxWindowRefMut<'_, 'a>> {
        Some(NoxWindowRefMut {
            window: self.windows.get_mut(&id)?,
            delta_time: self.delta_time,
        })
    } 

    #[inline(always)]
    pub fn delta_time(&self) -> time::Duration {
        self.delta_time
    }

    #[inline(always)]
    pub fn delta_time_secs_f32(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn active_ids(&self) -> &[WindowId] {
        &self.active_ids
    }

    #[inline(always)]
    pub(super) fn update(&mut self) {
        let count = self.windows.len();
        self.windows.retain(|_, win| {
            win.reset_input();
            !win.should_close()
        });
        if count != self.windows.len() {
            self.active_ids.clear();
            for (id, _) in &self.windows {
                self.active_ids.push(*id);
            }
        }
    }
}

impl<'a, 'b> NoxWindowRef<'a, 'b> {

    #[inline(always)]
    pub fn delta_time(&self) -> time::Duration {
        self.delta_time
    }

    #[inline(always)]
    pub fn delta_time_secs_f32(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }
}

impl<'a, 'b> Deref for NoxWindowRef<'a, 'b> {

    type Target = NoxWindow<'b>;

    fn deref(&self) -> &Self::Target {
        self.window
    }
}

impl<'a, 'b> NoxWindowRefMut<'a, 'b> {

    #[inline(always)]
    pub fn delta_time(&self) -> time::Duration {
        self.delta_time
    }

    #[inline(always)]
    pub fn delta_time_secs_f32(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }
}

impl<'a, 'b> Deref for NoxWindowRefMut<'a, 'b> {

    type Target = NoxWindow<'b>;

    fn deref(&self) -> &Self::Target {
        self.window
    }
}

impl<'a, 'b> DerefMut for NoxWindowRefMut<'a, 'b> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.window
    }
}

pub struct WindowContext<'a, 'b> {
    storage: &'a mut WindowStorage<'b>,
    event_loop: &'a ActiveEventLoop,
    memory: &'b Memory,
}

impl<'a, 'b> WindowContext<'a, 'b> {

    pub(super) fn new(
        storage: &'a mut WindowStorage<'b>,
        event_loop: &'a ActiveEventLoop,
        memory: &'b Memory,
    ) -> Self {
        Self {
            storage,
            event_loop,
            memory,
        }
    }

    pub fn create_window(
        &mut self,
        gpu: &mut gpu::GpuContext,
        attributes: NoxWindowAttributes,
    ) -> Result<WindowId>
    {
        let is_transparent = attributes.transparent();
        let attr = attributes.to_winit_attr();
        let window = self.event_loop
            .create_window(attr)
            .context("failed to create window")?;
        let id = window.id();
        let window = NoxWindow::new(
            gpu, window,
            is_transparent,
            self.memory.gpu().host_allocators()
        )?;
        self.windows
            .entry(id)
            .or_insert(window);
        self.active_ids.clear();
        for (id, _) in &self.storage.windows {
            self.storage.active_ids.push(*id);
        }
        Ok(id)
    }
}

impl<'a, 'b> Deref for WindowContext<'a, 'b> {

    type Target = WindowStorage<'b>;

    fn deref(&self) -> &Self::Target {
        self.storage
    }
}

impl<'a, 'b> DerefMut for WindowContext<'a, 'b> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.storage
    }
}
