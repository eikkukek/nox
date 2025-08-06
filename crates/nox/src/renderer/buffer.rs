mod properties;
mod state;
mod error;

use std::sync::Arc;

use core::num::NonZeroU64;

use ash::vk::{self, Handle};

use crate::renderer::memory_binder::DeviceMemory;

pub use error::BufferError;
pub(crate) use properties::BufferProperties;
pub(crate) use state::BufferState;

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
