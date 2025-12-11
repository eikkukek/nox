mod window_context;
mod expand_error;

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

pub use winit::keyboard::KeyCode;
pub use winit::event::MouseButton;
pub use winit::window::CursorIcon;
pub use winit::monitor::MonitorHandle;

pub use window_context::WindowContext;
pub use expand_error::{fn_expand_error, fn_expand_warn};

use crate::{
    dev::error::ErrorContext,
    log,
    expand_error,
    Event,
    gpu,
};

use super::{
    interface::Interface,
    memory::Memory,
    gpu::Gpu,
    clipboard::Clipboard,
};

pub type AppName = ArrayString<128>;

pub static ERROR_CAUSE_FMT: OnceLock<log::CustomFmt> = OnceLock::new();

pub struct Nox<'a, I>
    where
        I: Interface,
{
    interface: Arc<RwLock<I>>,
    window_ctx: Option<WindowContext>,
    window: Option<Arc<Window>>,
    memory: &'a Memory,
    gpu: Option<Gpu<'a>>,
    window_size: (u32, u32),
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
            window_ctx: Some(WindowContext::new()),
            window_size: (0, 0),
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
            gpu.clean_up(self.memory.gpu());
        }
    }
}

impl<'a, I: Interface> ApplicationHandler for Nox<'a, I> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.error_set() {
            return
        }
        let mut window_ctx = self.window_ctx.take().unwrap();
        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                window_ctx.cursor_position = (position.x, position.y);
                window_ctx.flags |= WindowContext::CURSOR_MOVED;
            },
            WindowEvent::MouseWheel { device_id: _, delta, phase: _ } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        window_ctx.mouse_scroll_line_delta = (x, y);
                    },
                    MouseScrollDelta::PixelDelta(d) => {
                        window_ctx.mouse_scroll_pixel_delta = (d.x, d.y);
                    }
                };
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event {
                    KeyEvent { physical_key, logical_key, text, location: _, state, repeat, .. } => {
                        let phys = window_ctx.physical_keys.entry(physical_key).or_default();
                        phys.pressed = state == ElementState::Pressed;
                        phys.released = state == ElementState::Released;
                        phys.held = state != ElementState::Released;
                        phys.repeat = repeat;
                        if let Some(text) = text && (phys.pressed || repeat) {
                            window_ctx.input_text.push((
                                match physical_key {
                                    PhysicalKey::Code(c) => c,
                                    PhysicalKey::Unidentified(_) => KeyCode::Backspace,
                                },
                                CompactString::new(&text))
                            );
                        }
                        let logic = window_ctx.logical_keys.entry(logical_key).or_default();
                        logic.pressed = state == ElementState::Pressed;
                        logic.released = state == ElementState::Released;
                        logic.held = state != ElementState::Released;
                        logic.repeat = repeat;
                    },
                };
            },
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let button = window_ctx.mouse_buttons.entry(button).or_default();
                button.pressed = state == ElementState::Pressed;
                button.released = state == ElementState::Released;
                button.held = state != ElementState::Released;
            },
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
                self.window_size = (size.width, size.height);
                window_ctx.window_size = (size.width, size.height);
                if let Some(gpu) = &mut self.gpu {
                    gpu.request_resize(size);
                }
            },
            WindowEvent::RedrawRequested => {
                let host_allocators = self.memory.gpu().host_allocators();
                if let Some(window) = self.window.clone() {
                    window_ctx.delta_time = window_ctx.delta_counter.elapsed();
                    window_ctx.delta_counter = time::Instant::now();
                    let mut interface = self.interface.write().unwrap();
                    if let Err(err) = interface
                        .event(Event::Update {
                            win: &mut window_ctx,
                            gpu: self.gpu
                                .as_mut()
                                .unwrap()
                                .context(),
                        }).context_from_origin(|orig| ErrorContext::EventError(orig))?
                    {
                        event_loop.exit();
                        self.flags |= Self::ERROR;
                        expand_error!(err);
                        return
                    }
                    if window_ctx.cursor_set() {
                        window.set_cursor(window_ctx.current_cursor);
                    }
                    window_ctx.flags &= !(WindowContext::CURSOR_SET | WindowContext::CURSOR_MOVED);
                    window_ctx.input_text.clear();
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
                window_ctx.reset_input();
            },
            _ => {},
        }
        self.window_ctx = Some(window_ctx);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut write = self.interface.write().unwrap();
        let init_settings = write.init_settings();
        self.monitors.clear();
        for handle in event_loop.available_monitors() {
            self.monitors.push(handle);
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
                .ctx_err("failed to create window")
            {
                Ok(window) => window,
                Err(err) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    return
                },
            };
            let inner_size = window.inner_size();
            self.window_size = (inner_size.width, inner_size.height);
            log::info!("created window");
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
                ).ctx_err("failed to init renderer")
            {
                Ok(r) => Some(r),
                Err(err) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    return
                }
            };
            let mut window_ctx = self.window_ctx.take().unwrap();
            window_ctx.clipboard = match Clipboard::new(&window).ctx_err("failed to create clipboard")
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
            window_ctx.window = self.window.clone();
            self.window_ctx = Some(window_ctx);
            let mut renderer_context = self.renderer.as_mut().unwrap().renderer_context();
            if let Err(err) = write
                .init_callback(self, &mut renderer_context)
                .ctx_err("init callback failed")
            {
                event_loop.exit();
                self.flags |= Self::ERROR;
                expand_error!(err);
            }
        }
    }
}
