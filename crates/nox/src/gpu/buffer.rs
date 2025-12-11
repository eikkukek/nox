mod properties;
mod state;
mod error;

use std::{ptr::NonNull, sync::Arc};

use core::num::NonZeroU64;

use ash::vk::{self, Handle};

use nox_mem::{AsRaw, impl_as_raw_bit_op};

use crate::gpu::memory_binder::DeviceMemory;
use crate::dev::has_not_bits;

pub use error::BufferError;
pub(crate) use properties::BufferProperties;
pub(crate) use state::BufferState;

#[repr(u32)]
#[derive(Clone, Copy, AsRaw)]
pub enum BufferUsage {
    TransferSrc = vk::BufferUsageFlags::TRANSFER_SRC.as_raw(),
    TransferDst = vk::BufferUsageFlags::TRANSFER_DST.as_raw(),
    IndexBuffer = vk::BufferUsageFlags::INDEX_BUFFER.as_raw(),
    VertexBuffer = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw(),
    UniformBuffer = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw(),
    StorageBuffer = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw(),
}

impl_as_raw_bit_op!(BufferUsage);

pub(crate) struct Buffer {
    handle: NonZeroU64,
    memory: Option<Box<dyn DeviceMemory>>,
    device: Arc<ash::Device>,
    properties: BufferProperties,
    state: BufferState,
}

impl Buffer {

    #[inline(always)]
    pub fn new(
        device: Arc<ash::Device>,
        properties: BufferProperties,
    ) -> Result<Self, vk::Result>
    {
        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            flags: properties.create_flags,
            size: properties.size,
            usage: properties.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe {
            device.create_buffer(&create_info, None)?
        };
        Ok(Self {
            handle: NonZeroU64::new(buffer.as_raw()).unwrap(),
            memory: None,
            device,
            properties,
            state: BufferState::new(
                vk::AccessFlags::NONE,
                vk::QUEUE_FAMILY_IGNORED,
                vk::PipelineStageFlags::TOP_OF_PIPE,
            )
        })
    }

    #[inline(always)]
    pub fn handle(&self) -> vk::Buffer {
        vk::Handle::from_raw(self.handle.get())
    }

    #[inline(always)]
    pub fn properties(&self) -> BufferProperties {
        self.properties
    }

    #[inline(always)]
    pub fn state(&self) -> BufferState {
        self.state
    }

    #[inline(always)]
    pub unsafe fn set_memory(&mut self, memory: Box<dyn DeviceMemory>) {
        debug_assert!(self.memory.is_none());
        self.memory = Some(memory);
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
    pub fn cmd_memory_barrier(
        &mut self,
        state: BufferState,
        command_buffer: vk::CommandBuffer,
    )
    {
        if self.state == state {
            return
        }
        let device = &self.device;
        let memory_barrier = self.state.to_memory_barrier(
            self.handle(),
            state,
            0,
            self.properties.size,
        );
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                self.state.pipeline_stage,
                state.pipeline_stage,
                Default::default(), Default::default(),
                &[memory_barrier], Default::default()
            );
        }
        self.state = state;
    }

    #[inline(always)]
    pub unsafe fn map_memory(&mut self) -> Result<NonNull<u8>, BufferError>
    {
        unsafe {
            self.memory
                .as_mut()
                .ok_or(BufferError::UnbindedMemory)?
                .map_memory()
                .map_err(|e| e.into())
        }
    }
}

impl Drop for Buffer {

    fn drop(&mut self) {
        let device = &self.device;
        unsafe {
            device.destroy_buffer(self.handle(), None);
            if let Some(memory) = self.memory.take() {
                memory.free_memory();
            }
        }
    }
}
