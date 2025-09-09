use std::sync::{Arc, RwLock};

use std::time;

use rustc_hash::FxHashMap;

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

pub struct Nox<'mem, I>
    where
        I: Interface,
{
    interface: Arc<RwLock<I>>,
    window: Option<Arc<Window>>,
    memory: &'mem Memory,
    renderer: Option<Renderer<'mem>>,
    error_flag: bool,
    cursor_pos: (f64, f64),
    mouse_scroll_delta: (f64, f64),
    mouse_scroll_delta_lines: (f32, f32),
    physical_keys: FxHashMap<PhysicalKey, InputState>,
    logical_keys: FxHashMap<Key, InputState>,
    input_text: Option<SmolStr>,
    delta_counter: time::Instant,
    delta_time: time::Duration,
}

impl<'mem, I: Interface> Nox<'mem, I>
{

    pub fn new(interface: I, memory: &'mem mut Memory) -> Self {
        Nox {
            interface: Arc::new(RwLock::new(interface)),
            window: None,
            memory,
            renderer: None,
            error_flag: false,
            cursor_pos: (0.0, 0.0),
            mouse_scroll_delta: Default::default(),
            mouse_scroll_delta_lines: Default::default(),
            physical_keys: Default::default(),
            logical_keys: Default::default(),
            input_text: None,
            delta_counter: time::Instant::now(),
            delta_time: time::Duration::ZERO,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    pub fn gpu_name(&mut self) -> renderer::DeviceName {
        self.renderer
            .as_ref()
            .unwrap()
            .device_info()
            .device_name().clone()
    }

    pub fn delta_time(&self) -> time::Duration {
        self.delta_time
    }

    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_pos
    }

    pub fn mouse_cursor_delta(&self) -> (f64, f64) {
        self.mouse_scroll_delta
    }

    pub fn mouse_cursor_delta_lines(&self) -> (f32, f32) {
        self.mouse_scroll_delta_lines
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .map(|&v| v)
            .unwrap_or_default()
            .pressed
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .map(|&v| v)
            .unwrap_or_default()
            .released
    }

    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.physical_keys
            .get(&PhysicalKey::Code(key))
            .map(|&v| v)
            .unwrap_or_default()
            .held
    }

    pub fn key_value(&self, key: KeyCode) -> f32 {
        if self.physical_keys
            .get(&PhysicalKey::Code(key))
            .map(|&v| v)
            .unwrap_or_default()
            .held { 1.0 }
        else { 0.0 }
    }

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
    }
}

impl<'mem, I: Interface> Drop for Nox<'mem, I> {

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

impl<'mem, I: Interface> ApplicationHandler for Nox<'mem, I> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.error_flag {
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
                        self.input_text = text;
                    },
                };
            },
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
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
                        self.error_flag = true;
                        eprintln!("Failed to update: {:?}", e);
                    }
                    window.request_redraw();
                    if let Some(renderer) = &mut self.renderer {
                        let mut handles = match renderer.transfer_requests(self.interface.clone(), renderer_context.transfer_requests) {
                            Ok(v) => v,
                            Err(e) => {
                                event_loop.exit();
                                self.error_flag = true;
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
                            self.error_flag = true;
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
                .with_min_inner_size(LogicalSize::new(1.0, 1.0));
            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => window,
                Err(e) => {
                    event_loop.exit();
                    self.error_flag = true;
                    eprintln!("Nox error: failed to create window ( {} )", e);
                    return
                },
            };
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
                    self.error_flag = true;
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
                self.error_flag = true;
                eprintln!("Nox error: init callback error ( {:?} )", e);
            }
            let mut handles = match self.renderer
                .as_mut()
                .unwrap()
                .transfer_requests(self.interface.clone(), renderer_context.transfer_requests) {
                Ok(v) => v,
                Err(e) => {
                    event_loop.exit();
                    self.error_flag = true;
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
