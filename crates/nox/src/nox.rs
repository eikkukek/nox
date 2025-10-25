use std::sync::{Arc, RwLock};

use std::time;

use nox_mem::vec_types::{Vector, GlobalVec};

use rustc_hash::FxHashMap;
use compact_str::CompactString;

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::*,
    window::{WindowId, Window},
    dpi::LogicalSize,
    keyboard::*,
};

use nox_mem::string_types::ArrayString;

pub use winit::keyboard::KeyCode;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;

pub use crate::renderer;

use super::{
    interface::Interface,
    memory::Memory,
    renderer::*,
};

pub type AppName = ArrayString<128>;

#[derive(Default, Clone, Copy)]
struct InputState {
    pressed: bool,
    released: bool,
    held: bool,
    repeat: bool,
}

pub struct Nox<'a, I>
    where
        I: Interface,
{
    interface: Arc<RwLock<I>>,
    window: Option<Arc<Window>>,
    memory: &'a Memory,
    renderer: Option<Renderer<'a>>,
    cursor_pos: (f64, f64),
    mouse_scroll_delta: (f64, f64),
    mouse_scroll_delta_lines: (f32, f32),
    physical_keys: FxHashMap<PhysicalKey, InputState>,
    logical_keys: FxHashMap<Key, InputState>,
    mouse_buttons: FxHashMap<MouseButton, InputState>,
    input_text: GlobalVec<(KeyCode, CompactString)>,
    clip_board: CompactString,
    delta_counter: time::Instant,
    delta_time: time::Duration,
    window_size: (u32, u32),
    current_cursor: CursorIcon,
    flags: u32,
}

impl<'a, I: Interface> Nox<'a, I>
{

    const ERROR: u32 = 0x1;
    const CURSOR_SET: u32 = 0x2;

    fn cursor_set(&self) -> bool {
        self.flags & Self::CURSOR_SET == Self::CURSOR_SET
    }

    fn error_set(&self) -> bool {
        self.flags & Self::ERROR == Self::ERROR
    }

    pub fn new(interface: I, memory: &'a mut Memory) -> Self {
        Nox {
            interface: Arc::new(RwLock::new(interface)),
            window: None,
            memory,
            renderer: None,
            cursor_pos: (0.0, 0.0),
            mouse_scroll_delta: Default::default(),
            mouse_scroll_delta_lines: Default::default(),
            physical_keys: Default::default(),
            logical_keys: Default::default(),
            mouse_buttons: Default::default(),
            input_text: Default::default(),
            clip_board: Default::default(),
            delta_counter: time::Instant::now(),
            delta_time: time::Duration::ZERO,
            window_size: Default::default(),
            current_cursor: CursorIcon::Default,
            flags: 0,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    #[inline(always)]
    pub fn gpu_name(&mut self) -> renderer::DeviceName {
        self.renderer
            .as_ref()
            .unwrap()
            .device_info()
            .device_name().clone()
    }

    #[inline(always)]
    pub fn set_cursor(&mut self, cursor: CursorIcon) {
        self.current_cursor = cursor;
        self.flags |= Self::CURSOR_SET;
    }

    #[inline(always)]
    pub fn set_cursor_hide(&self, hide: bool) {
        if let Some(window) = self.window.as_ref() {
            window.set_cursor_visible(!hide);
        }
    }

    #[inline(always)]
    pub fn get_clipboard(&self) -> &str {
        &self.clip_board
    }

    #[inline(always)]
    pub fn set_clipboard(&mut self, text: &str) {
        self.clip_board = CompactString::new(text);
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
    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_pos
    }

    #[inline(always)]
    pub fn normalized_cursor_position(&self) -> (f64, f64) {
        (
            self.cursor_pos.0 / self.window_size.0 as f64,
            self.cursor_pos.1 / self.window_size.1 as f64,
        )
    }

    #[inline(always)]
    pub fn normalized_cursor_position_f32(&self) -> (f32, f32) {
        (
            self.cursor_pos.0 as f32 / self.window_size.0 as f32,
            self.cursor_pos.1 as f32 / self.window_size.1 as f32,
        )
    }

    #[inline(always)]
    pub fn mouse_scroll_delta(&self) -> (f64, f64) {
        self.mouse_scroll_delta
    }

    #[inline(always)]
    pub fn mouse_scroll_delta_lines(&self) -> (f32, f32) {
        self.mouse_scroll_delta_lines
    }

    #[inline(always)]
    pub fn was_key_pressed(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .map(|&v| v)
            .unwrap_or_default()
            .pressed
    }

    #[inline(always)]
    pub fn was_key_released(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .copied()
            .unwrap_or_default()
            .released
    }

    #[inline(always)]
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .copied()
            .unwrap_or_default()
            .held
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
    pub fn was_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
            .pressed
    }

    #[inline(always)]
    pub fn was_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
            .released
    }

    #[inline(always)]
    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
            .held
    }

    #[inline(always)]
    pub fn mouse_button_value(&self, button: MouseButton) -> f32 {
        self.mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or_default()
            .held as u32 as f32
    }

    #[inline(always)]
    pub fn get_input_text(&self) -> &[(KeyCode, CompactString)] {
        &self.input_text
    }

    #[inline(always)]
    fn reset_input(&mut self) {
        self.mouse_scroll_delta = Default::default();
        self.mouse_scroll_delta_lines = Default::default();
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

impl<'a, I: Interface> Drop for Nox<'a, I> {

    fn drop(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            self.interface
                .write()
                .unwrap()
                .clean_up(&mut renderer.renderer_context());
            renderer.clean_up(self.memory.renderer_allocators());
        }
    }
}

impl<'a, I: Interface> ApplicationHandler for Nox<'a, I> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.error_set() {
            return
        }
        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.cursor_pos = (position.x, position.y);
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
               match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_scroll_delta_lines = (x, y);
                    },
                    MouseScrollDelta::PixelDelta(d) => {
                        self.mouse_scroll_delta = (d.x, d.y);
                    }
                };
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event {
                    KeyEvent { physical_key, logical_key, text, location: _, state, repeat, .. } => {
                        let phys = self.physical_keys.entry(physical_key).or_default();
                        phys.pressed = state == ElementState::Pressed;
                        phys.released = state == ElementState::Released;
                        phys.held = state != ElementState::Released;
                        phys.repeat = repeat;
                        let logic = self.logical_keys.entry(logical_key).or_default();
                        logic.pressed = state == ElementState::Pressed;
                        logic.released = state == ElementState::Released;
                        logic.held = state != ElementState::Released;
                        logic.repeat = repeat;
                        if let Some(text) = text && (phys.pressed || repeat) {
                            self.input_text.push((
                                match physical_key {
                                    PhysicalKey::Code(c) => c,
                                    PhysicalKey::Unidentified(_) => KeyCode::Backspace,
                                },
                                CompactString::new(&text))
                            );
                        }
                    },
                };
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let button = self.mouse_buttons.entry(button).or_default();
                button.pressed = state == ElementState::Pressed;
                button.released = state == ElementState::Released;
                button.held = state != ElementState::Released;
            },
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
                self.window_size = (size.width, size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.request_resize(size);
                }
            },
            WindowEvent::RedrawRequested => {
                let interface = self.interface.clone();
                let mut renderer_context = self.renderer.as_mut().unwrap().renderer_context();
                let renderer_allocators = self.memory.renderer_allocators();
                if let Some(window) = self.window.clone() {
                    self.delta_time = self.delta_counter.elapsed();
                    self.delta_counter = time::Instant::now();
                    if let Err(e) = interface
                        .write()
                        .unwrap()
                        .update(self, &mut renderer_context)
                    {
                        event_loop.exit();
                        self.flags |= Self::ERROR;
                        eprintln!("Failed to update: {:?}", e);
                    }
                    if self.cursor_set() {
                        window.set_cursor(self.current_cursor);
                    }
                    self.flags &= !Self::CURSOR_SET;
                    self.input_text.clear();
                    window.request_redraw();
                    if let Some(renderer) = &mut self.renderer {
                        let mut handles = match renderer.transfer_requests(self.interface.clone(), renderer_context.transfer_requests) {
                            Ok(v) => v,
                            Err(e) => {
                                event_loop.exit();
                                self.flags |= Self::ERROR;
                                eprintln!("Failed to record transfer commands: {:?}", e);
                                return
                            },
                        };
                        for handle in &mut handles {
                            if let Some(handle) = handle.take() {
                                if let Err(e) = handle.join() {
                                    panic!("thread poisoned during transfer commands: {:?}", e)
                                }
                            }
                        }
                        if let Err(e) = renderer.render(&window, self.interface.clone(), renderer_allocators) {
                            event_loop.exit();
                            self.flags |= Self::ERROR;
                            eprintln!("Nox renderer error: {}", e);
                        }
                    }
                }
                self.reset_input();
            },
            _ => {},
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let init_settings = self.interface.read().unwrap().init_settings();
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(init_settings.app_name.as_str())
                .with_inner_size(LogicalSize::new(
                    init_settings.window_size[0],
                    init_settings.window_size[1],
                ))
                .with_min_inner_size(LogicalSize::new(1.0, 1.0))
                .with_resizable(init_settings.window_resizeable);
            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => window,
                Err(e) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    eprintln!("Nox error: failed to create window ( {} )", e);
                    return
                },
            };
            let inner_size = window.inner_size();
            self.window_size = (inner_size.width, inner_size.height);
            println!("Nox message: created window {}", init_settings.app_name);
            event_loop.set_control_flow(ControlFlow::Poll);
            let renderer_allocators = self.memory.renderer_allocators();
            window.request_redraw();
            self.renderer = match Renderer
                ::new(
                    &window,
                    &init_settings.app_name,
                    init_settings.app_version,
                    init_settings.enable_vulkan_validation,
                    *self.memory.renderer_layout(),
                    3,
                    renderer_allocators,
                ) {
                Ok(r) => Some(r),
                Err(e) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    eprintln!("Nox error: failed to create renderer ( {} )", e);
                    return
                }
            };
            self.window = Some(Arc::new(window));
            let mut renderer_context = self.renderer.as_mut().unwrap().renderer_context();
            if let Err(e) = self.interface
                .clone()
                .write()
                .unwrap()
                .init_callback(self, &mut renderer_context) {
                event_loop.exit();
                self.flags |= Self::ERROR;
                eprintln!("Nox error: init callback error ( {:?} )", e);
            }
            let mut handles = match self.renderer
                .as_mut()
                .unwrap()
                .transfer_requests(self.interface.clone(), renderer_context.transfer_requests) {
                Ok(v) => v,
                Err(e) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    eprintln!("Failed to record transfer commands: {:?}", e);
                    return
                },
            };
            for handle in &mut handles {
                if let Some(handle) = handle.take() {
                    if let Err(e) = handle.join() {
                        panic!("thread poisoned during transfer commands: {:?}", e)
                    }
                }
            }
        }
    }
}
