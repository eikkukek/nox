pub mod win;
mod expand_error;

pub mod error_util {
    pub use super::expand_error::*;
}

use std::{
    sync::{Arc, RwLock, OnceLock},
    time,
};

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

use nox_error::Context;
use nox_mem::vec_types::Vector;

use crate::{
    dev::error::{ErrorContext, Tracked},
    log,
    Event,
    gpu,
    expand_error,
};

use super::{
    interface::Interface,
    memory::Memory,
    clipboard::Clipboard,
};

pub type AppName = ArrayString<128>;

pub static ERROR_CAUSE_FMT: OnceLock<log::CustomFmt> = OnceLock::new();

pub struct Nox<'a, I>
    where
        I: Interface,
{
    interface: Arc<RwLock<I>>,
    window_context: win::WindowContext,
    window: Option<Arc<Window>>,
    memory: &'a Memory,
    gpu: Option<gpu::Gpu<'a>>,
    flags: u32,
}

impl<'a, I: Interface> Nox<'a, I>
{

    const ERROR: u32 = 0x1;

    #[inline(always)]
    fn error_set(&self) -> bool {
        self.flags & Self::ERROR == Self::ERROR
    }

    pub fn new(
        interface: I,
        memory: &'a mut Memory,
    ) -> Self
    {
        log::init();
        log::info_fmt(|fmt| {
            fmt.text("INFO:  ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Green)).set_bold(true);
            })).message(|spec| spec);
        });
        log::warn_fmt(|fmt| {
            fmt.text("WARN:  ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Yellow)).set_bold(true);
            })).message(|spec| spec);
        });
        log::error_fmt(|fmt| {
            fmt.text("ERROR: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Red)).set_bold(true);
            })).message(|spec| spec);
        });
        log::debug_fmt(|fmt| {
            fmt.text("DEBUG: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Blue)).set_bold(true);
            })).message(|spec| spec);
        });
        log::trace_fmt(|fmt| {
            fmt.text("TRACE: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Rgb(130, 130, 130))).set_bold(true);
            })).message(|spec| spec);
        }); 
        if ERROR_CAUSE_FMT.get().is_none() {
            let mut error_cause_fmt = log::LogFmt::default();
            log::LogFmtBuilder::new(&mut error_cause_fmt)
                .text("       caused by: ", |spec| spec.with_color_spec(|spec| {
                    spec.set_fg(Some(log::Color::Magenta)).set_bold(true);
                })).message(|spec| spec);
            ERROR_CAUSE_FMT.set(log::custom_fmt(error_cause_fmt)).ok();
        }
        Nox {
            interface: Arc::new(RwLock::new(interface)),
            window: None,
            memory,
            gpu: None,
            window_context: win::WindowContext::new(),
            flags: 0,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }
}

impl<'a, I: Interface> Drop for Nox<'a, I> {

    fn drop(&mut self) {
        if let Some(mut gpu) = self.gpu.take() {
            gpu.wait_idle();
            self.interface
                .write()
                .unwrap()
                .clean_up(&mut gpu.context());
            gpu.clean_up(self.memory.gpu().host_allocators());
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
                self.window_context.cursor_position = (position.x, position.y);
                self.window_context.flags |= win::WindowContext::CURSOR_MOVED;
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.window_context.mouse_scroll_line_delta = (x, y);
                    },
                    MouseScrollDelta::PixelDelta(d) => {
                        self.window_context.mouse_scroll_pixel_delta = (d.x, d.y);
                    }
                };
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event {
                    KeyEvent { physical_key, logical_key, text, location: _, state, repeat, .. } => {
                        let phys = self.window_context.physical_keys
                            .entry(physical_key)
                            .or_default();
                        phys.pressed = state == ElementState::Pressed;
                        phys.released = state == ElementState::Released;
                        phys.held = state != ElementState::Released;
                        phys.repeat = repeat;
                        if let Some(text) = text && (phys.pressed || repeat) {
                            self.window_context.input_text.push((
                                match physical_key {
                                    PhysicalKey::Code(c) => c,
                                    PhysicalKey::Unidentified(_) => KeyCode::Backspace,
                                },
                                CompactString::new(&text))
                            );
                        }
                        let logic = self.window_context.logical_keys.entry(logical_key).or_default();
                        logic.pressed = state == ElementState::Pressed;
                        logic.released = state == ElementState::Released;
                        logic.held = state != ElementState::Released;
                        logic.repeat = repeat;
                    },
                };
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let button = self.window_context.mouse_buttons.entry(button).or_default();
                button.pressed = state == ElementState::Pressed;
                button.released = state == ElementState::Released;
                button.held = state != ElementState::Released;
            },
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
                self.window_context.window_size = (size.width, size.height);
                if let Some(gpu) = &mut self.gpu {
                    gpu.request_resize(size);
                }
            },
            WindowEvent::RedrawRequested => {
                let host_allocators = self.memory.gpu().host_allocators();
                if let Some(window) = self.window.clone() {
                    self.window_context.delta_time = self.window_context.delta_counter.elapsed();
                    self.window_context.delta_counter = time::Instant::now();
                    let mut interface = self.interface.write().unwrap();
                    if let Err(err) = interface
                        .event(Event::Update {
                            win: &mut self.window_context,
                            gpu: self.gpu
                                .as_mut()
                                .unwrap()
                                .context(),
                        }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                    {
                        event_loop.exit();
                        self.flags |= Self::ERROR;
                        expand_error!(err);
                        return
                    }
                    if self.window_context.cursor_set() {
                        window.set_cursor(self.window_context.current_cursor);
                    }
                    self.window_context.flags &= !(
                        win::WindowContext::CURSOR_SET | win::WindowContext::CURSOR_MOVED
                    );
                    self.window_context.input_text.clear();
                    window.request_redraw();
                    if let Some(gpu) = &mut self.gpu {
                        if let Err(err) = gpu 
                            .render(&window, &mut *interface, host_allocators)
                            .context("failed to render")
                        {
                            event_loop.exit();
                            self.flags |= Self::ERROR;
                            expand_error!(err);
                            return
                        }
                    }
                }
                self.window_context.reset_input();
            },
            _ => {},
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut interface = self.interface.write().unwrap();
        let init_settings = interface.init_settings();
        self.window_context.monitors.clear();
        for monitor in event_loop.available_monitors() {
            self.window_context.monitors.push(monitor);
        }
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(init_settings.app_name.as_str())
                .with_inner_size(LogicalSize::new(
                    init_settings.window_size[0],
                    init_settings.window_size[1],
                ))
                .with_min_inner_size(LogicalSize::new(1.0, 1.0))
                .with_resizable(init_settings.window_resizeable);
            let window = match event_loop
                .create_window(window_attributes)
                .context("failed to create window")
            {
                Ok(window) => window,
                Err(err) => {
                    event_loop.exit();
                    expand_error!(err);
                    self.flags |= Self::ERROR;
                    return
                },
            };
            let inner_size = window.inner_size();
            self.window_context.window_size = (inner_size.width, inner_size.height);
            log::info!("created window");
            event_loop.set_control_flow(ControlFlow::Poll);
            let host_allocators = self.memory.gpu().host_allocators();
            window.request_redraw();
            self.gpu = match gpu::Gpu
                ::new(
                    &window,
                    &init_settings.app_name,
                    init_settings.app_version,
                    init_settings.enable_vulkan_validation,
                    *self.memory.gpu().layout(),
                    3,
                    host_allocators,
                ).context("failed to init gpu")
            {
                Ok(r) => Some(r),
                Err(err) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    return
                }
            };
            self.window_context.clipboard = match Clipboard
                ::new(&window)
                .context("failed to create clipboard")
            {
                Ok(cb) => cb,
                Err(err) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    Clipboard::None
                }
            };
            self.window = Some(Arc::new(window));
            self.window_context.window = self.window.clone();
            let mut gpu = self.gpu.as_mut().unwrap().context();
            if let Err(err) = interface
                .init(&mut self.window_context, &mut gpu)
                .context("init callback failed")
            {
                event_loop.exit();
                self.flags |= Self::ERROR;
                expand_error!(err);
            }
        }
    }
}
