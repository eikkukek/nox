pub mod win;
pub mod event_loop;
mod event;
mod expand_error;

pub mod error_util {
    pub use super::expand_error::*;
}

use std::{
    sync::OnceLock, time
};

use winit::{
    event_loop::ControlFlow,
    application::ApplicationHandler,
    event::*,
    window::WindowId,
};

use rustc_hash::FxHashSet;

use nox_mem::vec_types::Vector;
use nox_alloc::arena_alloc::ArenaGuard;

use nox_error::Context;

pub use event::Event;
use event_loop::{EventLoop, ActiveEventLoop, WinitEventLoop, WinitActiveEventLoop};

use crate::{
    dev::{
        error::{ErrorContext, Tracked},
        or_flag,
    },
    log,
    Attributes,
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
            event_loop: EventLoop::new(),
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
    event_loop: EventLoop<'a>,
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

    const CLOSE_ON_NO_WINDOWS: u32 = 0x1;

    pub fn run(mut self) {
        let event_loop = WinitEventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
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
            self.event_loop.clean_up();
            gpu.clean_up();
        }
    }
}

impl<'a, 'b, I: Interface> ApplicationHandler for NoxRun<'a, 'b, I> {

    fn window_event(
        &mut self,
        event_loop: &WinitActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent
    ) {
        if event_loop.exiting() {
            return
        }
        if let Some(mut window) = self.event_loop.window_mut(window_id) {
            match event {
                WindowEvent::RedrawRequested => {
                    self.redraws_requested.insert(window_id);
                },
                event => window.process_event(event),
            }
        }
        let mut redraw = true;
        for window in self.event_loop.active_window_ids() {
            if !self.redraws_requested.contains(window) {
                redraw = false;
            }
        }
        if redraw {
            self.redraws_requested.clear();
            let host_allocators = self.memory.gpu().host_allocators();
            if let Some(gpu) = &mut self.gpu {
                self.event_loop.delta_time = self.event_loop.delta_counter.elapsed();
                self.event_loop.delta_counter = time::Instant::now();
                if let Err(err) = (self.interface)(Event::Update {
                    event_loop: &mut ActiveEventLoop::new(
                        &mut self.event_loop, event_loop,
                        self.memory,
                    ),
                    gpu: &mut gpu.context(),
                }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                {
                    event_loop.exit();
                    expand_error!(err);
                    return
                }
                if !event_loop.exiting() {
                    for (_, win) in self.event_loop.window_iter_mut() {
                        if win.cursor_set() {
                            win.handle.set_cursor(win.current_cursor);
                        }
                        win.flags &= !(
                            win::Window::CURSOR_SET | win::Window::CURSOR_MOVED
                        );
                        win.input_text.clear();
                        win.handle.request_redraw();
                        if win.transparent_set() {
                            let is = win.is_transparent();
                            win.handle.set_transparent(is);
                            win.flags &= !win::Window::TRANSPARENT_SET;
                        }
                    }
                    let mut win = ActiveEventLoop::new(
                        &mut self.event_loop,
                        event_loop,
                        self.memory,
                    );
                    let tmp_alloc = ArenaGuard::new(self.memory.tmp_alloc());
                    if let Err(err) = gpu 
                        .render(&mut win, &mut self.interface, host_allocators, tmp_alloc)
                        .context("failed to render")
                    {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                } 
            }
            self.event_loop.update();
            if self.close_on_no_windows() && self.event_loop.active_window_ids().is_empty() {
                event_loop.exit();
            }
        }
    }

    fn resumed(&mut self, event_loop: &WinitActiveEventLoop) {
        if self.gpu.is_none() {
            or_flag!(self.flags, Self::CLOSE_ON_NO_WINDOWS, self.attributes.close_on_no_windows);
            self.gpu = match gpu::Gpu::new(
                event_loop,
                &self.attributes.gpu_attributes,
                self.memory.gpu(),
            ).context("failed to init gpu") {
                Ok(mut gpu) => {
                    if let Err(err) = self.on_init.init(
                        &mut ActiveEventLoop::new(&mut self.event_loop, event_loop, self.memory),
                        &mut gpu.context()
                    ) {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                    if let Err(err) = (self.interface)(Event::Initialized {
                        event_loop: &mut ActiveEventLoop::new(
                            &mut self.event_loop,
                            event_loop,
                            self.memory,
                        ),
                        gpu: &mut gpu.context(),
                    }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                    {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                    Some(gpu)
                },
                Err(err) => {
                    event_loop.exit();
                    expand_error!(err);
                    return
                },
            };
            if self.close_on_no_windows() && self.event_loop.active_window_ids().is_empty() {
                event_loop.exit();
            }
        }
    }
}
