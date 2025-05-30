mod image_state;
mod frame_graph;
mod physical_device;
mod swapchain_context;
mod vulkan_context;
mod thread_context;
mod helpers;
mod buffer_allocator;

use super::{
    interface::Interface,
    Version,
    string::{String, LargeError},
    stack_allocator::{StackMemory, StackAllocator},
    allocator_traits::AllocateExt,
    map_types::FixedMap,
    stack_allocator::StackGuard,
    AppName
};

pub use ash;
pub use frame_graph::{UID, Pass, WriteInfo, FrameGraph, ResourcePool, ImageResource};
pub use image_state::ImageState;
use frame_graph::Exec;
use helpers::Handle;
use swapchain_context::{PresentResult, SwapchainContext};
use thread_context::ThreadContext;
use vulkan_context::{VulkanContext, SwapchainState};

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
    pub init_allocator: StackAllocator<'mem>,
    pub swapchain_allocator: StackAllocator<'mem>,
}

impl<'mem> Memory<'mem> {

    pub fn new(layout: MemoryLayout, pool: &mut StackMemory) -> Option<Self> {
        Some(Self {
            init_allocator: StackAllocator::new(layout.init_size, pool)?,
            swapchain_allocator: StackAllocator::new(layout.swapchain_size, pool)?,
        })
    }
}

pub struct Frame<'r, 'mem> {
    device: Handle<'r, ash::Device>,
    command_buffer: Handle<'r, vk::CommandBuffer>,
    swapchain_image_resource: ImageResource<'r>,
    allocator: Option<StackGuard<'r, 'mem>>,
    temp_allocator: Option<StackGuard<'r, 'mem>>,
    queue_family_indices: physical_device::QueueFamilyIndices,
}

impl<'r, 'mem> Frame<'r, 'mem> {

    fn new(
        device: Handle<'r, ash::Device>,
        command_buffer: Handle<'r, vk::CommandBuffer>,
        swapchain_image_resource: ImageResource<'r>,
        allocator: StackGuard<'r, 'mem>,
        temp_allocator: StackGuard<'r, 'mem>,
        queue_family_indices: physical_device::QueueFamilyIndices,
    ) -> Self {
        Self {
            device,
            command_buffer,
            swapchain_image_resource,
            allocator: Some(allocator),
            temp_allocator: Some(temp_allocator),
            queue_family_indices,
        }
    }

    pub fn swapchain_image_resource(&self) -> frame_graph::ImageResource<'r> {
        self.swapchain_image_resource.clone()
    }

    pub fn graphics_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_graphics_index()
    }

    pub fn transfer_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_transfer_index()
    }

    pub fn compute_queue_family_index(&self) -> u32 {
        self.queue_family_indices.get_compute_index()
    }

    pub fn take_allocator(&mut self) -> Option<StackGuard<'r, 'mem>> {
        self.allocator.take()
    }

    pub fn take_temp_allocator(&mut self) -> Option<StackGuard<'r, 'mem>> {
        self.temp_allocator.take()
    }

    pub fn render<'a, 'b, B: AllocateExt<'b>>(
        &mut self,
        frame_graph: &FrameGraph<'a>,
        resource_pool: &mut ResourcePool<'a, 'r>,
        callbacks: Option<&FixedMap<'a, UID, fn(UID)>>,
        temp_allocator: &mut B,
    ) {
        frame_graph.execute(
            &self.device,
            *self.command_buffer,
            resource_pool,
            &mut self.swapchain_image_resource,
            callbacks,
            temp_allocator,
        );
    }
} 

pub struct Renderer<'mem> {
    vulkan_context: ManuallyDrop<VulkanContext<'mem>>,
    main_thread_context: ManuallyDrop<ThreadContext<'mem>>,
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
                renderer_memory,
            ).map_err(|e| {
                e
            })?);
        let main_thread_context = ManuallyDrop::new(ThreadContext
            ::new(
                vulkan_context.device(),
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
        let queue_family_indices = *self.vulkan_context.queue_family_indices();
        let swapchain_context = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                renderer_memory)
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
        let mut frame = Frame::new(
            device.clone(),
            Handle::new(frame_data.command_buffer),
            ImageResource::<'_>::new(
                Handle::new(frame_data.image),
                Handle::new(frame_data.image_view),
                SwapchainContext::image_subresource_range(),
                frame_data.image_state,
                frame_data.format,
                frame_data.extent,
            ),
            StackGuard::new(&mut renderer_memory.swapchain_allocator),
            StackGuard::new(&mut renderer_memory.init_allocator),
            queue_family_indices);
        interface
            .render(&mut frame)
            .map_err(|e| {
                String::format(format_args!("interface failed to render ( {} )", e))
            })?;
        let (submit_info, fence) = swapchain_context
            .setup_submit(
                &device,
                frame.swapchain_image_resource.state(),
                queue_family_indices.get_graphics_index()
            );
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
