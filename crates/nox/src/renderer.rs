mod memory_layout;
mod device_allocators;
mod handle;
mod helpers;
mod physical_device;
mod vulkan_context;
mod swapchain_context;
mod thread_context;
mod pipeline_cache;
mod frame_graph;
//mod transient_allocator;
mod linear_device_alloc;
mod buffer_allocator;
mod image_state;

use core::slice;

pub use ash;

use ash::vk;
use nox_mem::{GlobalVec, Vector};

use crate::stack_alloc::StackAlloc;

use super::{
    interface::Interface,
    Version,
    string_types::{ArrayString, array_format, LargeError},
    stack_alloc::StackGuard,
    AppName
};

pub use memory_layout::MemoryLayout;
pub use handle::Handle;
pub use frame_graph::{FrameGraph, FrameGraphInit, RenderError, ResourceID, Pass, WriteInfo, ResourcePool, Image, ImageResource};
pub use image_state::ImageState;
pub use buffer_allocator::{BufferAllocator, DeviceMemory, BufferAlloc};

use device_allocators::DeviceAllocators;
use linear_device_alloc::LinearDeviceAlloc;
use vulkan_context::VulkanContext;
use physical_device::PhysicalDeviceInfo;
use swapchain_context::{PresentResult, SwapchainContext};
use thread_context::ThreadContext;

use winit::{
    dpi::PhysicalSize,
    window::Window
};

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub struct Allocators {
    init: StackAlloc,
    swapchain: StackAlloc,
}

impl Allocators {

    pub fn new(memory_layout: MemoryLayout) -> Option<Self> {
        Some(Self {
            init: StackAlloc::new(memory_layout.init_size())?,
            swapchain: StackAlloc::new(memory_layout.swapchain_size())?,
        })
    }
}

pub struct Renderer<'mem> {
    main_thread_context: ThreadContext,
    _device_allocators: DeviceAllocators<'mem, GlobalVec<buffer_allocator::Block>>,
    frame_device_allocs: GlobalVec<LinearDeviceAlloc>,
    vulkan_context: VulkanContext<'mem>,
    _memory_layout: MemoryLayout,
    buffered_frame_count: u32,
}

impl<'mem> Renderer<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        buffered_frame_count: u32,
        allocators: &'mem Allocators,
    ) -> Result<Self, LargeError> {
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                buffered_frame_count,
                enable_validation,
                allocators)
            .map_err(|e| e)?;
        let main_thread_context = ThreadContext
            ::new(
                vulkan_context.device(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                array_format!("failed to create main thread context ( {:?} )", e)
            )?;
        let device = vulkan_context.device().clone();
        let physical_device_info = vulkan_context.physical_device_info().clone();
        let mut frame_device_allocs = GlobalVec
            ::with_capacity(buffered_frame_count as usize)
            .map_err(|e |array_format!("global alloc failed ( {:?} )", e))?;
        for _ in 0..buffered_frame_count {
            frame_device_allocs
                .push(
                    LinearDeviceAlloc
                        ::new(
                            device.clone(),
                            memory_layout.device_frame_size(),
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                            &physical_device_info,
                        )
                        .map_err(|e| array_format!("failed to create device allocator ( {:?} )", e))?
                )
                .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?;
        }
        Ok(Self {
            vulkan_context,
            main_thread_context,
            _device_allocators: DeviceAllocators::new(
                &memory_layout,
                device,
                &physical_device_info,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?,
                GlobalVec::with_capacity(256)
                    .map_err(|e| array_format!("global alloc failed ( {:?} )", e))?)
                .map_err(|e|
                    array_format!("failed to create device allocators ( {} )", e)
                )?,
            frame_device_allocs,
            _memory_layout: memory_layout,
            buffered_frame_count,
        })
    }

    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_swapchain_update(self.buffered_frame_count, size);
    }

    pub fn render<I: Interface>(
        &mut self,
        window: &Window,
        interface: &mut I,
        allocators: &'mem Allocators,
    ) -> Result<(), LargeError> {
        let device = self.vulkan_context.device().clone();
        let swapchain_loader = self.vulkan_context.swapchain_loader().clone();
        let queue_family_indices = *self.vulkan_context.queue_family_indices();
        let graphics_queue = self.vulkan_context.graphics_queue();
        let swapchain_context = self.vulkan_context
            .get_swapchain_context(
                self.main_thread_context.graphics_pool(),
                allocators)
            .map_err(|e| {
                array_format!("failed to get swapchain context ( {} )", e)
            })?;
        let frame_data = match swapchain_context
            .setup_image(&device, &swapchain_loader)
            .map_err(|e| {
                array_format!("failed to setup render image ( {} )", e)
            })? {
                Some(r) => r,
                None => return Ok(())
            };
        if let Err(e) = helpers::begin_command_buffer(&device, frame_data.command_buffer) {
            return Err(array_format!("failed to begin command buffer {:?}", e))
        }
        let mut render_image =
            Image::with_resource(
                ImageResource::new(
                    frame_data.image,
                    frame_data.image_view,
                    frame_data.image_state,
                ),
                frame_data.format,
                frame_data.extent,
                vk::ImageUsageFlags::SAMPLED,
                vk::SampleCountFlags::TYPE_1,
                SwapchainContext::image_subresource_range(),
            );
        let alloc = StackGuard::new(&allocators.swapchain);
        let frame_graph = frame_graph::new(
            device.clone(),
            frame_data.command_buffer,
            &mut render_image,
            &alloc,
            &self.frame_device_allocs[frame_data.frame_index as usize],
            queue_family_indices,
            frame_data.frame_index,
        );
        interface
            .render(frame_graph)
            .map_err(|e| array_format!("interface failed to render ( {:?} )", e))?;
        let (submit_info, fence) = swapchain_context
            .setup_submit(
                device.clone(),
                render_image.resource().unwrap().state(),
                queue_family_indices.get_graphics_index()
            );
        if let Err(e) = unsafe { device.end_command_buffer(frame_data.command_buffer) } {
            return Err(array_format!("failed to end command buffer {:?}", e))
        }
        if let Err(e) = unsafe { device.queue_submit(graphics_queue, slice::from_ref(&submit_info), fence) } {
            return Err(array_format!("graphics queue submit failed {:?}", e))
        }
        let present_result = swapchain_context
            .present_submit(&swapchain_loader, graphics_queue)
            .map_err(|e| array_format!("queue present failed {}", e))?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.request_swapchain_update(self.buffered_frame_count, window.inner_size());
        }
        Ok(())
    }

    pub fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
