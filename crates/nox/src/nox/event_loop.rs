use core::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

use nox_mem::vec_types::GlobalVec;

use crate::dev::error::*;
use crate::misc::ToRef;

use super::{
    win,
    *,
};

pub(crate) use winit::event_loop::EventLoop as WinitEventLoop;
pub(crate) use winit::event_loop::ActiveEventLoop as WinitActiveEventLoop;

pub struct EventLoop<'a> {
    pub(super) windows: FxHashMap<WindowId, win::Window<'a>>,
    pub(super) active_ids: GlobalVec<WindowId>,
    pub(super) delta_counter: time::Instant,
    pub(super) delta_time: time::Duration,
}

impl<'a> EventLoop<'a> {

    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            windows: FxHashMap::default(),
            active_ids: Default::default(),
            delta_counter: time::Instant::now(),
            delta_time: Default::default(),
        }
    }

    #[inline(always)]
    pub(crate) fn window_iter_mut(&mut self) -> impl Iterator<Item = (&WindowId, &mut win::Window<'a>)> {
        self.windows
            .iter_mut()
    }

    #[inline(always)]
    pub fn is_window_active(&self, id: impl ToRef<WindowId>) -> bool {
        self.windows.contains_key(id.to_ref())
    }

    #[inline(always)]
    pub fn window(&self, id: WindowId) -> Option<win::WindowRef<'_, 'a>> {
        Some(win::WindowRef {
            window: self.windows.get(&id)?,
            delta_time: self.delta_time,
        })
    }

    #[inline(always)]
    pub fn window_mut(&mut self, id: WindowId) -> Option<win::WindowRefMut<'_, 'a>> {
        Some(win::WindowRefMut {
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

    #[inline(always)]
    pub fn active_window_ids(&self) -> &[WindowId] {
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

    #[inline(always)]
    pub(super) fn clean_up(&mut self) {
        self.active_ids.clear();
        self.windows.clear();
    }
}

pub struct ActiveEventLoop<'a, 'b> {
    event_loop: &'a mut EventLoop<'b>,
    winit_event_loop: &'a WinitActiveEventLoop,
    memory: &'b Memory,
}

impl<'a, 'b> ActiveEventLoop<'a, 'b> {

    pub(super) fn new(
        event_loop: &'a mut EventLoop<'b>,
        winit_event_loop: &'a WinitActiveEventLoop,
        memory: &'b Memory,
    ) -> Self {
        Self {
            event_loop,
            winit_event_loop,
            memory,
        }
    }

    pub fn create_window(
        &mut self,
        gpu: &mut gpu::GpuContext,
        attributes: win::WindowAttributes,
    ) -> Result<win::WindowId>
    {
        let is_transparent = attributes.transparent();
        let attr = attributes.to_winit_attr();
        let window = self.winit_event_loop
            .create_window(attr)
            .context("failed to create window")?;
        let id = window.id();
        let window = win::Window::new(
            gpu, window,
            is_transparent,
            self.memory.gpu().host_allocators()
        )?;
        self.windows
            .entry(id)
            .or_insert(window);
        self.active_ids.clear();
        for (id, _) in &self.event_loop.windows {
            self.event_loop.active_ids.push(*id);
        }
        Ok(id)
    }

    #[inline(always)]
    pub fn exit(&self) {
        self.winit_event_loop.exit();
    }
}

impl<'a, 'b> Deref for ActiveEventLoop<'a, 'b> {

    type Target = EventLoop<'b>;

    fn deref(&self) -> &Self::Target {
        self.event_loop
    }
}

impl<'a, 'b> DerefMut for ActiveEventLoop<'a, 'b> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.event_loop
    }
}
