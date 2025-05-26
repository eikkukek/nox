mod image_state;
mod physical_device;
mod swapchain_context;
mod vulkan_context;

use crate::AppName;

use super::{
    Version,
    string::{String, LargeError},
    stack_allocator::{StackMemory, StackAllocator},
};

use vulkan_context::{VulkanContext, VulkanMemory};
use swapchain_context::ImageData;

use winit::{
    dpi::PhysicalSize,
    window::Window
};

pub type DeviceName = String<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub struct MemoryLayout {
    init_size: usize,
    swapchain_size: usize,
}

impl MemoryLayout {

    pub fn default() -> Self {
        Self {
            init_size: 1 << 18,
            swapchain_size: 1 << 18,
        }
    }

    pub fn alloc_size(&self) -> usize {
        self.init_size + self.swapchain_size
    }
}

pub struct Memory<'mem> {
    pub vulkan_memory: VulkanMemory<'mem>,
}

impl<'mem> Memory<'mem> {

    pub fn new(layout: MemoryLayout, pool: &mut StackMemory) -> Option<Self> {
        let vulkan_memory = VulkanMemory {
            init_allocator: StackAllocator::new(layout.init_size, pool)?,
            swapchain_allocator: StackAllocator::new(layout.swapchain_size, pool)?,
        };
        Some(
            Self {
                vulkan_memory,
            }
        )
    }
}

pub struct Renderer<'mem> {
    vulkan_context: VulkanContext<'mem>,
}

impl<'mem> Renderer<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        renderer_memory: &mut Memory<'mem>,
    ) -> Result<Self, LargeError> {
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                enable_validation,
                &mut renderer_memory.vulkan_memory,
            ).map_err(|e| {
                e
            })?;
        Ok(
            Self {
                vulkan_context,
            }
        )
    }

    pub fn gpu_name(&self) -> DeviceName {
        self.vulkan_context.device_name()
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_resize(size);
    }

    pub fn draw(
        &mut self,
        window: &Window,
        renderer_memory: &mut Memory<'mem>
    ) -> Result<(), LargeError> {
        let Some(image_data) = self.vulkan_context
            .begin_frame(
                window,
                &mut renderer_memory.vulkan_memory)
            .map_err(|e| {
                String::format(format_args!("failed to begin frame ( {} )", e))
            })? else { return Ok(()) };
        Ok(self.vulkan_context
            .end_frame(image_data.image_state)
            .map_err(|e| {
                String::format(format_args!("failed to end frame ( {} )", e))
            })?
        )
    }
}
