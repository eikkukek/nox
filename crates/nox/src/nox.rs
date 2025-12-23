pub mod win;
mod expand_error;

pub mod error_util {
    pub use super::expand_error::*;
}

use std::{
    sync::OnceLock, time
};

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::*,
    window::{WindowId, Window},
};

use rustc_hash::FxHashSet;

use nox_mem::vec_types::Vector;
use nox_alloc::arena_alloc::ArenaGuard;

use nox_error::Context;

use crate::{
    dev::{
        error::{ErrorContext, Tracked},
        or_flag,
    },
    log,
    Attributes,
    Event,
    gpu,
    expand_error,
    OnInit,
};

use super::{
    interface::Interface,
    memory::Memory,
};

pub static ERROR_CAUSE_FMT: OnceLock<log::CustomFmt> = OnceLock::new();

pub struct Nox;

impl Nox {

    pub fn default_attributes() -> Attributes {
        Attributes::new()
    }

    pub fn on_init<'a>() -> OnInit<'a> {
        OnInit::new()
    }

    pub fn new<'a, 'b, I: Interface>(
        attributes: Attributes,
        on_init: &'a OnInit<'b>,
        memory: &'a mut Memory,
        interface: I,
    ) -> NoxRun<'a, 'b, I>
        where 
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
        NoxRun {
            interface,
            attributes,
            window_storage: win::WindowStorage::new(),
            redraws_requested: FxHashSet::default(),
            on_init,
            memory,
            gpu: None,
            flags: 0,
        }
    }
}

pub struct NoxRun<'a, 'b, I>
    where
        I: Interface,
{
    interface: I,
    attributes: Attributes,
    window_storage: win::WindowStorage<'a>,
    redraws_requested: FxHashSet<WindowId>,
    on_init: &'a OnInit<'b>,
    memory: &'a Memory,
    gpu: Option<gpu::Gpu>,
    flags: u32,
}

impl<'a, 'b, I> NoxRun<'a, 'b, I>
    where
        I: Interface,
{

    const ERROR: u32 = 0x1; 
    const CLOSE_ON_NO_WINDOWS: u32 = 0x2;

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    #[inline(always)]
    fn error_set(&self) -> bool {
        self.flags & Self::ERROR == Self::ERROR
    }

    #[inline(always)]
    fn close_on_no_windows(&self) -> bool {
        self.flags & Self::CLOSE_ON_NO_WINDOWS == Self::CLOSE_ON_NO_WINDOWS
    }
}

impl<'a, 'b, I: Interface> Drop for NoxRun<'a, 'b, I> {

    fn drop(&mut self) {
        if let Some(mut gpu) = self.gpu.take() {
            gpu.wait_idle();
            (self.interface)(Event::CleanUp {
                gpu: &mut gpu.context()
            }).ok();
            gpu.clean_up();
        }
    }
}

impl<'a, 'b, I: Interface> ApplicationHandler for NoxRun<'a, 'b, I> {

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent
    ) {
        if self.error_set() {
            return
        }
        if let Some(mut window) = self.window_storage.window_mut(window_id) {
            match event {
                WindowEvent::RedrawRequested => {
                    self.redraws_requested.insert(window_id);
                },
                event => window.process_event(event),
            }
        }
        let mut redraw = true;
        for window in self.window_storage.active_ids() {
            if !self.redraws_requested.contains(window) {
                redraw = false;
            }
        }
        if redraw {
            self.redraws_requested.clear();
            let host_allocators = self.memory.gpu().host_allocators();
            if let Some(gpu) = &mut self.gpu {
                self.window_storage.delta_time = self.window_storage.delta_counter.elapsed();
                self.window_storage.delta_counter = time::Instant::now();
                if let Err(err) = (self.interface)(Event::Update {
                    win: &mut win::WindowContext::new(
                        &mut self.window_storage, event_loop,
                        self.memory,
                    ),
                    gpu: &mut gpu.context(),
                }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    return
                }
                for (_, win) in self.window_storage.window_iter_mut() {
                    if win.cursor_set() {
                        win.handle.set_cursor(win.current_cursor);
                    }
                    win.flags &= !(
                        win::NoxWindow::CURSOR_SET | win::NoxWindow::CURSOR_MOVED
                    );
                    win.input_text.clear();
                    win.handle.request_redraw();
                    if win.transparent_set() {
                        let is = win.is_transparent();
                        win.handle.set_transparent(is);
                        win.flags &= !win::NoxWindow::TRANSPARENT_SET;
                    }
                }
                let mut win = win::WindowContext::new(
                    &mut self.window_storage,
                    event_loop,
                    self.memory,
                );
                let tmp_alloc = ArenaGuard::new(self.memory.tmp_alloc());
                if let Err(err) = gpu 
                    .render(&mut win, &mut self.interface, host_allocators, tmp_alloc)
                    .context("failed to render")
                {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    return
                }
            }
            self.window_storage.update();
            if self.close_on_no_windows() && self.window_storage.active_ids().is_empty() {
                event_loop.exit();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window_storage.monitors.clear();
        for monitor in event_loop.available_monitors() {
            self.window_storage.monitors.push(monitor);
        }
        if self.gpu.is_none() {
            or_flag!(self.flags, Self::CLOSE_ON_NO_WINDOWS, self.attributes.close_on_no_windows);
            self.gpu = match gpu::Gpu::new(
                event_loop,
                &self.attributes.app_name,
                self.attributes.app_version,
                self.attributes.vulkan_validation,
                *self.memory.gpu().layout(),
                3,
                self.memory.gpu().host_allocators(),
            ).context("failed to init gpu") {
                Ok(mut gpu) => {
                    if let Err(err) = self.on_init.init(
                        &mut win::WindowContext::new(&mut self.window_storage, event_loop, self.memory),
                        &mut gpu.context()
                    ) {
                        event_loop.exit();
                        self.flags |= Self::ERROR;
                        expand_error!(err);
                        return
                    }
                    if let Err(err) = (self.interface)(Event::Initialized {
                        win: &mut win::WindowContext::new(
                            &mut self.window_storage,
                            event_loop,
                            self.memory,
                        ),
                        gpu: &mut gpu.context(),
                    }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                    {
                        event_loop.exit();
                        self.flags |= Self::ERROR;
                        expand_error!(err);
                        return
                    }
                    Some(gpu)
                },
                Err(err) => {
                    event_loop.exit();
                    self.flags |= Self::ERROR;
                    expand_error!(err);
                    return
                },
            };
            if self.close_on_no_windows() && self.window_storage.active_ids().is_empty() {
                event_loop.exit();
            }
        }
    }
}
