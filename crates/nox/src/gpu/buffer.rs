mod create_info;
mod properties;
mod state;
mod error;

use crate::sync::Arc;

use nox_ash::vk;

use parking_lot::{
    RwLockWriteGuard, MappedRwLockWriteGuard,
    RwLockReadGuard, MappedRwLockReadGuard,
};

use compact_str::format_compact;

use nox_mem::{
    vec::{Vec32, Vector},
    vec32,
    slot_map::SlotMap,
};

use crate::gpu::prelude::{
    memory_binder::{DeviceMemory, MemoryBinder},
    Vulkan,
    subresource_state::*,
    COMMAND_INDEX_IGNORED,
    CommandOrdering,
    BufferId,
    ResourceId,
};

use crate::dev::has_not_bits;

use crate::dev::error as dev_error;
use dev_error::Context;

pub use create_info::*;
pub use error::BufferError;
pub(crate) use properties::BufferProperties;
pub(crate) use state::*;

pub(crate) struct BufferMeta {
    vk: Arc<Vulkan>,
    handle: vk::Buffer,
    memory: Box<dyn DeviceMemory>,
    properties: BufferProperties,
    state: Vec32<BufferRange>,
    last_used_frame: u64,
}

impl BufferMeta {

    #[inline(always)]
    fn new(
        vk: Arc<Vulkan>,
        create_info: &BufferCreateInfo<'_>,
        alloc: &mut (impl MemoryBinder + ?Sized),
        bind_memory_info: &mut vk::BindBufferMemoryInfo<'static>,
    ) -> Result<Self, dev_error::Error>
    {
        let properties = BufferProperties {
            size: create_info.size.get(),
            usage: create_info.usage.into(),
            create_flags: create_info.create_flags,
        };
        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            flags: properties.create_flags,
            size: properties.size,
            usage: properties.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let device_mem_requirements = vk::DeviceBufferMemoryRequirements {
            s_type: vk::StructureType::DEVICE_BUFFER_MEMORY_REQUIREMENTS,
            p_create_info: &create_info,
            ..Default::default()
        };
        let mut mem_requirements = Default::default();
        unsafe {
            vk.device()
            .get_device_buffer_memory_requirements(&device_mem_requirements, &mut mem_requirements);
        }
        let memory = unsafe { alloc.alloc(&mem_requirements)
            .context("failed to allocate GPU memory for buffer")?
        };
        let handle = unsafe {
            vk.device().create_buffer(&create_info, None)
            .context("failed to create Vulkan buffer")?
        };
        *bind_memory_info = vk::BindBufferMemoryInfo {
             buffer: handle,
             memory: memory.device_memory(),
             memory_offset: memory.offset(),
             ..Default::default()
        };
        Ok(Self {
            handle,
            memory,
            vk,
            properties,
            state: vec32![BufferRange {
                state: BufferState::new(
                    vk::AccessFlags2::NONE,
                    vk::PipelineStageFlags2::NONE,
                    vk::QUEUE_FAMILY_IGNORED,
                    COMMAND_INDEX_IGNORED,
                    0
                ),
                offset: 0,
                size: properties.size,
            }],
            last_used_frame: 0,
        })
    }

    #[inline(always)]
    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }

    #[inline(always)]
    pub fn properties(&self) -> BufferProperties {
        self.properties
    }

    #[inline(always)]
    pub fn get_last_used_frame(&self) -> u64 {
        self.last_used_frame
    }

    #[inline(always)]
    pub unsafe fn set_last_used_frame(&mut self, frame: u64) {
        self.last_used_frame = frame;
    }

    #[inline(always)]
    pub fn validate_usage(
        &self,
        usage: vk::BufferUsageFlags,
    ) -> Option<BufferError> {
        let has = self.properties.usage;
        (has_not_bits!(has, usage))
            .then_some(BufferError::UsageMismatch {
                missing_usage: usage ^ has & usage,
        })
    }

    #[inline(always)]
    pub fn validate_range(
        &self,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> Option<BufferError> {
        (self.properties.size < offset + size)
            .then_some(BufferError::OutOfRange {
                buffer_size: self.properties.size, requested_offset: offset, requested_size: size,
        })
    }

    #[inline(always)]
    pub fn memory_barrier<'a>(
        &mut self,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        state: BufferState,
        ordering: CommandOrdering,
        cache: &'a mut BufferMemoryBarrierCache,
    ) -> Result<&'a [BufferMemoryBarrier], BufferError>
    {
        if size == 0 {
            return Ok(&[])
        }
        if offset + size > self.properties.size {
            return Err(BufferError::OutOfRange {
                buffer_size: self.properties.size,
                requested_offset: offset,
                requested_size: size,
            })
        }
        let mut not_inserted = None;
        let mut range = BufferRange {
            state,
            offset,
            size,
        };
        for i in (0..self.state.len()).rev() {
            match unsafe { self.state.get_unchecked(i as usize) }.overwrite(&range) {
                StateOverwrite::NoOverlap => continue,
                StateOverwrite::Combine(new_range) => {
                    self.state.remove(i);
                    range = new_range;
                    not_inserted = Some(i);
                },
                StateOverwrite::Consume(barrier) => {
                    self.state.remove(i);
                    match ordering {
                        CommandOrdering::None => {
                            if barrier.src_queue_family_index != barrier.dst_queue_family_index {
                                cache.cache.push(barrier);
                            }
                        },
                        CommandOrdering::Strict => {
                            cache.cache.push(barrier);
                        }
                    }
                    not_inserted = Some(i);
                },
                StateOverwrite::Cut(left, right, barrier) => {
                    self.state.remove(i);
                    match ordering {
                        CommandOrdering::None => {
                            if barrier.src_queue_family_index != barrier.dst_queue_family_index {
                                cache.cache.push(barrier);
                            }
                        },
                        CommandOrdering::Strict => {
                            cache.cache.push(barrier);
                        }
                    }
                    if left.size != 0 {
                        self.state.insert(i, left);
                    }
                    self.state.insert(i + 1, range);
                    if right.size != 0 {
                        self.state.insert(i + 2, right);
                    }
                    not_inserted = None;
                    break
                },
                StateOverwrite::Shrink(new_range, barrier) => {
                    unsafe {
                        *self.state.get_unchecked_mut(i as usize)
                            = new_range;
                    }
                    match ordering {
                        CommandOrdering::None => {
                            if barrier.src_queue_family_index != barrier.dst_queue_family_index {
                                cache.cache.push(barrier);
                            }
                        },
                        CommandOrdering::Strict => {
                            cache.cache.push(barrier);
                        }
                    }
                    if new_range.offset < range.offset {
                        self.state.insert(i + 1, range);
                        not_inserted = None;
                        break
                    }
                },
            }
        }
        if let Some(i) = not_inserted {
            self.state.insert(i, range);
        }
        Ok(&cache.cache)
    }

    #[inline(always)]
    pub fn flush_state(&mut self) {
        for range in &mut self.state {
            range.state.stage_mask = vk::PipelineStageFlags2::ALL_COMMANDS;
            range.state.access_mask = vk::AccessFlags2::MEMORY_WRITE;
        }
    }

    #[inline(always)]
    pub fn map_memory(&mut self) -> Result<&mut [u8], BufferError>
    {
        self.memory
            .map_memory()
            .map_err(|e| e.into())
    }
}

impl Drop for BufferMeta {

    #[inline(always)]
    fn drop(&mut self) {
        let device = self.vk.device();
        unsafe {
            device.destroy_buffer(self.handle(), None);
        }
    }
}

pub struct BufferWriteGuard<'a> {
    id: BufferId,
    meta: MappedRwLockWriteGuard<'a, BufferMeta>,
}

impl<'a> BufferWriteGuard<'a> {

    #[inline(always)]
    pub(crate) fn new(
        id: BufferId,
        buffers: RwLockWriteGuard<'a, SlotMap<BufferMeta>>,
    ) -> Option<Self>
    {
        Some(Self {
            id,
            meta: RwLockWriteGuard::try_map(buffers, |buffers| {
                buffers.get_mut(id.slot_index()).ok()
            }).ok()?,
        })
    }

    #[inline(always)]
    pub(crate) fn meta(&mut self) -> &mut BufferMeta {
        &mut self.meta
    }

    #[inline(always)]
    pub fn id(&self) -> BufferId {
        self.id
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        self.meta.properties.size
    }

    /// Tries to map buffer memory.
    ///
    /// # Safety
    ///
    #[inline(always)]
    pub unsafe fn map_memory(&mut self) -> dev_error::Result<&mut [u8]> {
        self.meta
        .map_memory()
        .context_with(|| format_compact!(
            "failed to map buffer (id: {}) memory", self.id
        ))
    }
}

pub struct BufferReadGuard<'a> {
    id: BufferId,
    meta: MappedRwLockReadGuard<'a, BufferMeta>,
}

impl<'a> BufferReadGuard<'a> {

    #[inline(always)]
    pub(crate) fn new(
        id: BufferId,
        buffers: RwLockReadGuard<'a, SlotMap<BufferMeta>>,
    ) -> Option<Self>
    {
        Some(Self {
            id,
            meta: RwLockReadGuard::try_map(buffers, |buffers| {
                buffers.get(id.slot_index()).ok()
            }).ok()?,
        })
    }

    #[inline(always)]
    pub(crate) fn meta(&self) -> &BufferMeta {
        &self.meta
    }

    #[inline(always)]
    pub fn id(&self) -> BufferId {
        self.id
    }

    #[inline(always)]
    pub fn size(&self) -> u64 {
        self.meta.properties.size
    }
}
