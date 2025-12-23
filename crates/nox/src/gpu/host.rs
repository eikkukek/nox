use core::cell::UnsafeCell;

use compact_str::format_compact;

use nox_mem::{
    vec_types::{ArrayVec, Vector},
    slot_map::{GlobalSlotMap, SlotIndex},
};
use nox_alloc::arena_alloc::ArenaAlloc;

use crate::dev::{
    error::*,
    format_location,
};

use super::{MAX_BUFFERED_FRAMES, MemoryLayout};

pub(super) type ArenaAllocId = SlotIndex<ArenaAlloc>;

pub struct HostAllocators {
    swapchains: UnsafeCell<GlobalSlotMap<ArenaAlloc>>,
    frame_graphs: UnsafeCell<ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}>>,
    memory_layout: MemoryLayout,
}

impl HostAllocators {

    pub fn new(memory_layout: MemoryLayout) -> Result<Self> {
        Ok(Self {
            swapchains: UnsafeCell::new(Default::default()),
            frame_graphs: UnsafeCell::new(Default::default()),
            memory_layout: memory_layout,
        })
    }

    #[track_caller]
    pub(super) fn create_tmp_alloc(&self) -> Result<ArenaAlloc> {
        ArenaAlloc
            ::new(self.memory_layout.tmp_arena_size())
            .context_with(|| format_compact!(
                "failed to create temp arena allocator at {}",
                caller!(),
            ))
    }

    pub(super) fn create_swapchain_alloc(&self) -> Result<ArenaAllocId> {
        let swapchains = unsafe {
            &mut *self.swapchains.get()
        };
        let alloc = ArenaAlloc
            ::new(self.memory_layout.swapchain_size())
            .context("failed to create swapchain arena alloc")?;
        Ok(swapchains.insert(alloc))
    }

    pub(super) fn get_swapchain_alloc(&self, id: ArenaAllocId) -> Result<&ArenaAlloc> {
        unsafe {
            &*self.swapchains
                .get()
        }.get(id).context("failed to find swapchain arena alloc")
    }

    pub(super) fn destroy_swapchain_alloc(&self, id: ArenaAllocId) -> Result<()> {
        unsafe {
            &mut *self.swapchains.get()
        }.remove(id).context("failed to destroy swapchain alloc")?;
        Ok(())
    }

    pub(super) fn realloc_frame_graphs(&self, buffered_frames: u32) -> Result<()> {
        assert!(buffered_frames <= MAX_BUFFERED_FRAMES);
        unsafe { &mut *self.frame_graphs.get() }.try_resize_with(
            buffered_frames as usize,
            || ArenaAlloc
                ::new(self.memory_layout.frame_graph_arena_size())
                .context_with(|| format_location!("failed to create arena alloc at {loc}")),
            |err| Error::new(ErrorContext::VecError(location!()), err)
        )
    }

    pub(super) fn frame_graphs(&self) -> &ArrayVec<ArenaAlloc, {MAX_BUFFERED_FRAMES as usize}> {
        unsafe { &*self.frame_graphs.get() }
    }
}
