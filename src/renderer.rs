pub mod image_state;
pub mod frame_graph;
mod physical_device;
mod swapchain_context;
mod vulkan_context;
mod thread_context;
mod helpers;

use crate::AppName;

use super::{
    interface::Interface,
    Version,
    string::{String, LargeError},
    stack_allocator::{StackMemory, StackAllocator},
};

use image_state::ImageState;
use swapchain_context::{PresentResult, SwapchainContext};
use thread_context::ThreadContext;
use vulkan_context::{VulkanContext, VulkanMemory, SwapchainState};

use ash::vk;

use winit::{
    dpi::PhysicalSize,
    window::Window
};

use std::{
    slice,
    mem::ManuallyDrop
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
    vulkan_context: ManuallyDrop<VulkanContext<'mem>>,
    main_thread_context: ManuallyDrop<ThreadContext>,
}

impl<'mem> Renderer<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        renderer_memory: &mut Memory<'mem>,
    ) -> Result<Self, LargeError> {
        let vulkan_context = ManuallyDrop::new(VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                enable_validation,
                &mut renderer_memory.vulkan_memory,
            ).map_err(|e| {
                e
            })?);
        let main_thread_context = ManuallyDrop::new(ThreadContext
            ::new(
                vulkan_context.device().clone(),
                vulkan_context.queue_family_indices())
            .map_err(|e| {
                String::format(format_args!(
                    "failed to create main thread context ( {} )", e
                ))
            })?);
        Ok(Self {
                vulkan_context,
                main_thread_context,
        })
    }

    pub fn gpu_name(&self) -> DeviceName {
        self.vulkan_context.physical_device_name()
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_resize(size);
    }

    pub fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: &mut I,
        renderer_memory: &mut Memory<'mem>,
    ) -> Result<(), LargeError> {
        let device = self.vulkan_context.device();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let swapchain_loader = self.vulkan_context.swapchain_loader();
        let graphics_index = self.vulkan_context.queue_family_indices().get_graphics_index();
        let swapchain_context = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                &mut renderer_memory.vulkan_memory)
            .map_err(|e| {
                String::format(format_args!(
                    "failed to get swapchain context ( {} )", e
                ))
            })?;
        let Some(frame_data) = swapchain_context
            .setup_image(&device, &swapchain_loader)
            .map_err(|e| {
                String::format(format_args!(
                    "failed to setup render image ( {} )", e
                ))
            })? else {
                self.vulkan_context.swapchain_state(SwapchainState::OutOfDate(window.inner_size()));
                return Ok(())
            };
        if let Err(e) = helpers::begin_command_buffer(&device, frame_data.command_buffer) {
            return Err(String::format(format_args!(
                "failed to begin command buffer {:?}", e
            )))
        }
        let image_state = ImageState::new(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            graphics_index,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        );
        let memory_barrier = frame_data.image_state.to_memory_barrier(
            frame_data.image,
            &image_state,
            SwapchainContext::image_subresource_range()
        );
        unsafe {
            device.cmd_pipeline_barrier(
                frame_data.command_buffer,
                frame_data.image_state.pipeline_stage,
                image_state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                slice::from_ref(&memory_barrier),
            );
        }
        let (submit_info, fence) = swapchain_context.setup_submit(&device, image_state, graphics_index);
        if let Err(e) = unsafe { device.end_command_buffer(frame_data.command_buffer) } {
            return Err(String::format(format_args!(
                "failed to end command buffer {:?}", e
            )))
        }
        if let Err(e) = unsafe { device.queue_submit(*graphics_queue, slice::from_ref(&submit_info), fence) } {
            return Err(
                String::format(format_args!(
                    "graphics queue submit failed {:?}", e,
                ))
            )
        }
        let present_result = swapchain_context
            .present_submit(&swapchain_loader, *graphics_queue)
            .map_err(|e| {
                String::format(format_args!(
                    "queue present failed {}", e
                ))
            })?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.swapchain_state(SwapchainState::OutOfDate(window.inner_size()))
        }
        Ok(())
    }

    pub fn destroy(&mut self) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool());
        self.main_thread_context.destroy();
        self.vulkan_context.destroy();
    }
}
