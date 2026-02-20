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

use ahash::AHashSet;

use nox_mem::vec::Vector;
use nox_error::Context;

pub use event::Event;
use event::RunEvent;
use event_loop::{EventLoop, ActiveEventLoop, WinitActiveEventLoop};

use crate::{
    dev::error::{self, ErrorContext, Tracked},
    log,
    Attributes,
    gpu,
    expand_error,
    OnInit,
};

use super::interface::Interface;

pub static ERROR_CAUSE_FMT: OnceLock<log::CustomFmt> = OnceLock::new();

pub struct Nox;

impl Nox {

    pub fn default_attributes() -> Attributes {
        Attributes::new()
    }

    pub fn on_init<'a>() -> OnInit<'a> {
        OnInit::new()
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, 'b, I: Interface>(
        attributes: Attributes,
        on_init: &'a OnInit<'b>,
        interface: I,
    ) -> error::Result<NoxRun<'a, 'b, I>>
    {
        let event_loop = EventLoop
            ::new()
            .context("failed to create event loop")?;
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
        Ok(NoxRun {
            interface,
            attributes,
            event_loop,
            redraws_requested: AHashSet::default(),
            on_init,
            gpu: None,
        })
    }
}

pub struct NoxRun<'a, 'b, I>
    where
        I: Interface,
{
    interface: I,
    attributes: Attributes,
    event_loop: EventLoop,
    redraws_requested: AHashSet<WindowId>,
    on_init: &'a OnInit<'b>,
    gpu: Option<gpu::Gpu>,
}

impl<'a, 'b, I> NoxRun<'a, 'b, I>
    where
        I: Interface,
{

    pub fn run(mut self) {
        self.event_loop
            .init()
            .run_app(&mut self)
            .expect("failed to run event loop");
    }
}

impl<'a, 'b, I: Interface> ApplicationHandler<RunEvent> for NoxRun<'a, 'b, I> {

    fn new_events(&mut self, event_loop: &WinitActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            let event_loop = ActiveEventLoop::new(&self.event_loop, event_loop);
            log::info!("event loop starting");
            match gpu::Gpu::new(
                &event_loop,
                &self.attributes.gpu_attributes,
            ).context("failed to init gpu") {
                Ok(gpu) => {
                    let gpu = self.gpu.insert(gpu);
                    if let Err(err) = self.on_init.init(
                        &event_loop,
                        &mut gpu.context()
                    ) {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                    if let Err(err) = (self.interface)(Event::Initialized {
                        event_loop: &event_loop,
                        gpu: &mut gpu.context(),
                    }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                    {
                        event_loop.exit();
                        expand_error!(err);
                    }
                },
                Err(err) => {
                    event_loop.exit();
                    expand_error!(err);
                },
            };
        }
    }

    fn exiting(&mut self, _event_loop: &WinitActiveEventLoop) {
        log::info!("event loop exiting");
        if let Some(mut gpu) = self.gpu.take() {
            gpu.wait_idle();
            (self.interface)(Event::CleanUp {
                gpu: &mut gpu.context()
            }).ok();
            self.event_loop.clean_up();
            gpu.clean_up();
        }
    }

    fn user_event(&mut self, event_loop: &WinitActiveEventLoop, event: RunEvent) {
        match event {
            RunEvent::Tick => {
                if let Some(gpu) = &mut self.gpu {
                    self.event_loop.delta_time = self.event_loop.delta_counter.elapsed();
                    self.event_loop.delta_counter = time::Instant::now();
                    if let Err(err) = (self.interface)(Event::Update {
                        event_loop: &ActiveEventLoop::new(
                            &self.event_loop, event_loop,
                        ),
                        gpu: &mut gpu.context(),
                    }).context_from_tracked(|orig| ErrorContext::EventError(orig.or_this()))
                    {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                    if !event_loop.exiting() {
                        for (_, win) in self.event_loop.window_iter() {
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
                        let event_loop = ActiveEventLoop::new(
                            &self.event_loop,
                            event_loop,
                        );
                        if let Err(err) = gpu 
                            .render(&event_loop, &mut self.interface)
                            .context("failed to render")
                        {
                            event_loop.exit();
                            expand_error!(err);
                            return
                        }
                    } 
                }
                self.event_loop.update();
                if self.event_loop.no_active_windows() {
                    self.event_loop.tick();
                }
            },
        }
    }

    fn resumed(&mut self, _event_loop: &WinitActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &WinitActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent
    ) {
        if event_loop.exiting() {
            return
        }
        if let Some(mut window) = self.event_loop.get_window(window_id) {
            match event {
                WindowEvent::RedrawRequested => {
                    self.redraws_requested.insert(window_id);
                },
                event => window.process_event(event),
            }
        }
        let mut redraw = true;
        for id in self.event_loop.active_ids.get_mut() {
            if !self.redraws_requested.contains(id) {
                redraw = false;
            }
        }
        if redraw {
            self.redraws_requested.clear();
            self.event_loop.tick();
        }
    } 
}
