use core::{
    ops::Deref,
    cell::UnsafeCell,
};

use ahash::AHashMap;

use winit::event_loop::EventLoopProxy;

use nox_mem::vec::StdVec;
use nox_threads::executor::ThreadPool;

use crate::dev::error::*;
use crate::misc::ToRef;

use super::{
    win,
    *,
};

pub(super) type WinitEventLoop = winit::event_loop::EventLoop<RunEvent>;
pub(crate) use winit::event_loop::ActiveEventLoop as WinitActiveEventLoop;

pub struct EventLoop {
    pub(super) thread_pool: ThreadPool,
    pub(super) windows: UnsafeCell<AHashMap<WindowId, Box<win::Window>>>,
    pub(super) active_ids: UnsafeCell<StdVec<WindowId>>,
    pub(super) delta_counter: time::Instant,
    pub(super) delta_time: time::Duration,
    pub(super) proxy: OnceLock<EventLoopProxy<super::RunEvent>>,
}

crate::assert_sync!(EventLoopProxy<super::RunEvent>);

impl EventLoop {

    #[inline(always)]
    pub(super) fn new() -> Result<Self> {
        Ok(Self {
            thread_pool: ThreadPool
                ::new()
                .context("failed to create thread pool")?,
            windows: UnsafeCell::new(AHashMap::default()),
            active_ids: Default::default(),
            delta_counter: time::Instant::now(),
            delta_time: Default::default(),
            proxy: OnceLock::new(),
        })
    }

    pub(super) fn init(&self) -> WinitEventLoop {
        let event_loop = WinitEventLoop
            ::with_user_event()
            .build().expect("failed to create winit event loop");
        event_loop.set_control_flow(ControlFlow::Poll);
        self.proxy.get_or_init(|| event_loop.create_proxy());
        event_loop
    }

    pub(super) fn tick(&self) {
        self.proxy
            .get()
            .unwrap()
            .send_event(RunEvent::Tick)
            .ok();
    }

    #[inline(always)]
    pub(crate) fn window_iter(&self) -> impl Iterator<Item = (WindowId, &mut win::Window)> {
        unsafe { &mut *self.windows.get() }
            .iter_mut()
            .map(|(k, v)| (*k, v.as_mut()))
    }

    #[inline(always)]
    pub fn is_window_valid<T>(&self, id: T) -> bool
        where 
            T: ToRef<WindowId>,
    {
        unsafe { &*self.active_ids.get() }.contains(id.to_ref())
    }

    #[inline(always)]
    pub fn get_window<T>(&self, id: T) -> Option<win::WindowContext<'_>>
        where
            T: ToRef<WindowId>,
    {
        Some(win::WindowContext {
            window: unsafe {
                (&mut *self.windows.get())
                    .get_mut(id.to_ref())?
                    .as_mut()
            },
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

    #[inline(always)]
    pub fn active_window_ids(&self) -> Box<[WindowId]> {
        Box::from(unsafe {
            &*self.active_ids.get()
        }.as_slice())
    }

    #[inline(always)]
    pub fn no_active_windows(&self) -> bool {
        unsafe { &*self.active_ids.get() }.is_empty()
    }

    #[inline(always)]
    pub(super) fn update(&mut self) {
        let windows = self.windows.get_mut();
        let count = windows.len();
        windows.retain(|_, win| {
            win.reset_input();
            !win.should_close()
        });
        if count != windows.len() {
            let active_ids = self.active_ids.get_mut();
            active_ids.clear();
            for id in windows.keys() {
                active_ids.push(*id);
            }
        }
    }

    #[inline(always)]
    pub(super) fn clean_up(&mut self) {
        log::info!("cleaning up event loop");
        self.active_ids.get_mut().clear();
        self.windows.get_mut().clear();
    }
}

pub struct ActiveEventLoop<'a> {
    event_loop: &'a EventLoop,
    winit_event_loop: &'a WinitActiveEventLoop,
}

impl<'a> ActiveEventLoop<'a> {

    pub(super) fn new(
        event_loop: &'a EventLoop,
        winit_event_loop: &'a WinitActiveEventLoop,
    ) -> Self {
        Self {
            event_loop,
            winit_event_loop,
        }
    }

    pub fn create_window(
        &self,
        gpu: &mut gpu::GpuContext,
        attributes: win::WindowAttributes,
    ) -> Result<win::WindowId>
    {
        let is_transparent = attributes.transparent();
        let attr = attributes.into_winit_attr();
        let window = self.winit_event_loop
            .create_window(attr)
            .context("failed to create window")?;
        let id = window.id();
        let window = win::Window::new(
            gpu, window,
            is_transparent,
        )?;
        let windows = unsafe { &mut *self.windows.get() };
        windows
            .entry(id)
            .or_insert(Box::new(window));
        let active_ids = unsafe {
            &mut *self.active_ids.get()
        };
        active_ids.clear();
        for id in windows.keys() {
            active_ids.push(*id);
        }
        Ok(id)
    }

    #[inline(always)]
    pub fn thread_pool(&self) -> ThreadPool {
        self.thread_pool.clone()
    }

    #[inline(always)]
    pub(crate) fn winit(&self) -> &WinitActiveEventLoop {
        self.winit_event_loop
    }

    #[inline(always)]
    pub fn exit(&self) {
        self.winit_event_loop.exit();
    }
}

impl<'a> Deref for ActiveEventLoop<'a> {

    type Target = EventLoop;

    fn deref(&self) -> &Self::Target {
        self.event_loop
    }
}
