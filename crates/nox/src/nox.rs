mod platform;
pub mod win;
pub mod event_loop;
mod event;

use std::time;

use winit::{
    event_loop::ControlFlow,
    application::ApplicationHandler,
    event::*,
};

use compact_str::format_compact;

use ahash::AHashSet;

pub use event::Event;
pub use platform::Platform;
use event::RunEvent;
use event_loop::{EventLoop, ActiveEventLoop, WinitActiveEventLoop};

use crate::{
    error::*,
    expand_error,
    log,
    Attributes,
    gpu,
    Globals,
    win::WindowId,
};

pub fn init() {
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
    if expand::ERROR_CAUSE_FMT.get().is_none() {
        let mut error_cause_fmt = log::LogFmt::default();
        log::LogFmtBuilder::new(&mut error_cause_fmt)
            .text("       caused by: ", |spec| spec.with_color_spec(|spec| {
                spec.set_fg(Some(log::Color::Magenta)).set_bold(true);
            })).message(|spec| spec);
        expand::ERROR_CAUSE_FMT.set(log::custom_fmt(error_cause_fmt)).ok();
    }
}

pub struct Nox;

pub fn default_attributes() -> Attributes
{
    Attributes::new()
}

pub fn create_globals<'a>() -> Globals<'a> {
    Globals::new()
}

impl Nox {

    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, 'b, F>(
        platform: Platform,
        logical_device: gpu::LogicalDevice,
        attributes: Attributes,
        globals: &'a Globals<'b>,
        event_handler: F,
    ) -> Result<NoxRun<'a, 'b, F>>
        where F: FnMut(&ActiveEventLoop, Event) -> EventResult<()>,
    {
        let event_loop = EventLoop
            ::new(platform)
            .context("failed to create event loop")?; 
        let (gpu, gpu_cache) = gpu::Gpu
            ::new(&event_loop, logical_device, attributes)
            .context("failed to create Gpu")?;
        Ok(NoxRun {
            event_handler,
            event_loop,
            redraws_requested: AHashSet::default(),
            globals,
            gpu,
            gpu_cache,
        })
    }
}

pub struct NoxRun<'a, 'b, F>
    where
        F: FnMut(&ActiveEventLoop, Event) -> EventResult<()>,
{
    event_handler: F,
    event_loop: EventLoop,
    redraws_requested: AHashSet<winit::window::WindowId>,
    globals: &'a Globals<'b>,
    gpu: gpu::Gpu,
    gpu_cache: gpu::Cache,
}

impl<'a, 'b, F> NoxRun<'a, 'b, F>
    where
        F: FnMut(&ActiveEventLoop, Event) -> EventResult<()>,
{

    pub fn run(mut self) {
        self.event_loop
            .init()
            .event_loop
            .run_app(&mut self)
            .expect("failed to run event loop");
    }
}

impl<'a, 'b, F> ApplicationHandler<RunEvent> for NoxRun<'a, 'b, F> 
    where F: FnMut(&ActiveEventLoop, Event) -> EventResult<()>,
{

    fn new_events(&mut self, event_loop: &WinitActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            log::info!("event loop starting");
            let event_loop = ActiveEventLoop::new(
                self.gpu.clone(),
                &self.event_loop,
                event_loop
            );
            if let Err(err) = self.globals.init(
                &event_loop,
            ) {
                event_loop.exit();
                expand_error!(err);
                return
            }
            if let Err(err) = (self.event_handler)(
                &event_loop,
                Event::Initialized,
            ).context_from_tracked(|orig| format_compact!(
                "init event error at {}", orig.or_this(),
            ))
            {
                event_loop.exit();
                expand_error!(err);
            }
            self.event_loop.tick();
        }
    }

    fn exiting(&mut self, event_loop: &WinitActiveEventLoop) {
        log::info!("event loop exiting");
        let event_loop = ActiveEventLoop::new(
            self.gpu.clone(),
            &self.event_loop,
            event_loop
        );
        if let Err(err) = (self.event_handler)(&event_loop, Event::CleanUp)
            .context_from_tracked(|orig| format_compact!(
                "clean up event error at {}", orig.or_this(),
            ))
        {
            expand_error!(err);
        }
        self.event_loop.clean_up();
    }

    fn user_event(&mut self, event_loop: &WinitActiveEventLoop, event: RunEvent) {
        match event {
            RunEvent::Tick => {
                self.event_loop.delta_time = self.event_loop.delta_counter.elapsed();
                self.event_loop.delta_counter = time::Instant::now();
                let event_loop = ActiveEventLoop::new(
                    self.gpu.clone(),
                    &self.event_loop,
                    event_loop
                );
                if let Err(err) = (self.event_handler)(
                    &event_loop,
                    Event::Update
                ).context_from_tracked(|orig| format_compact!(
                    "update event error at {}", orig.or_this()
                ))
                {
                    event_loop.exit();
                    expand_error!(err);
                    return
                }
                if !event_loop.winit().exiting() {
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
                    if let Err(err) = self.gpu
                        .tick(|event| {
                            (self.event_handler)(&event_loop, Event::GpuEvent(event))
                        }, &mut self.gpu_cache)
                        .context("gpu error")
                    {
                        event_loop.exit();
                        expand_error!(err);
                        return
                    }
                } 
                self.event_loop.update();
                self.event_loop.tick();
            },
        }
    }

    fn resumed(&mut self, _event_loop: &WinitActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &WinitActiveEventLoop,
        window_id: winit::window::WindowId,
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
        let redraw = !self.event_loop.active_ids
            .get_mut()
            .iter()
            .any(|id| !self.redraws_requested.contains(&id.0));
        if redraw {
            self.redraws_requested.clear();
        }
    } 
}
