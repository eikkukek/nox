mod memory_layout;
mod device_allocators;
mod handle;
mod helpers;
mod physical_device;
mod vulkan_context;
mod swapchain_context;
mod thread_context;
mod frame;
mod frame_graph;
mod buffer_allocator;
mod image_state;

use core::{
    slice,
    cell::RefCell,
};

pub use ash;

use crate::stack_alloc::StackAlloc;

use super::{
    interface::Interface,
    Version,
    string_types::{ArrayString, array_format, LargeError},
    global_alloc::GlobalAlloc,
    stack_alloc::StackGuard,
    vec_types::DynVec,
    AppName
};

pub use memory_layout::MemoryLayout;
pub use handle::Handle;
pub use frame_graph::{UID, Pass, WriteInfo, FrameGraph, ResourcePool, ImageResource};
pub use frame::Frame;
pub use image_state::ImageState;
pub use buffer_allocator::{BufferAllocator, BufferAlloc};

use device_allocators::DeviceAllocators;
use vulkan_context::{VulkanContext, SwapchainState};
use physical_device::PhysicalDeviceInfo;
use swapchain_context::{PresentResult, SwapchainContext};
use thread_context::ThreadContext;
use frame::Construct;

use winit::{
    dpi::PhysicalSize,
    window::Window
};

pub type DeviceName = ArrayString<{ash::vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;

pub struct Allocators {
    init: RefCell<StackAlloc>,
    swapchain: RefCell<StackAlloc>,
    global_alloc: RefCell<GlobalAlloc>,
}

impl Allocators {

    pub fn new(memory_layout: MemoryLayout) -> Option<Self> {
        Some(Self {
            init: RefCell::new(StackAlloc::new(memory_layout.init_size())?),
            swapchain: RefCell::new(StackAlloc::new(memory_layout.swapchain_size())?),
            global_alloc: RefCell::new(GlobalAlloc::default()),
        })
    }
}

pub struct Renderer<'mem> {
    main_thread_context: ThreadContext,
    device_allocators: DeviceAllocators<'mem, DynVec<'mem, buffer_allocator::Block, GlobalAlloc>>,
    vulkan_context: VulkanContext<'mem>,
    _memory_layout: MemoryLayout,
}

impl<'mem> Renderer<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        memory_layout: MemoryLayout,
        allocators: &'mem Allocators,
    ) -> Result<Self, LargeError> {
        let vulkan_context = VulkanContext
            ::new(
                window,
                &app_name,
                app_version,
                enable_validation,
                allocators)
            .map_err(|e| e)?;
        let main_thread_context = ThreadContext
            ::new(
                vulkan_context.device(),
                vulkan_context.queue_family_indices())
            .map_err(|e|
                array_format!("failed to create main thread context ( {} )", e)
            )?;
        let device = vulkan_context.device().clone();
        let physical_device_info = vulkan_context.physical_device_info().clone();
        Ok(Self {
            vulkan_context,
            main_thread_context,
            device_allocators: DeviceAllocators::new(
                &memory_layout,
                device,
                &physical_device_info,
                DynVec::with_capacity(256, &allocators.global_alloc)
                    .ok_or_else(|| ArrayString::from_str("global alloc failed"))?,
                DynVec::with_capacity(256, &allocators.global_alloc)
                    .ok_or_else(|| ArrayString::from_str("global alloc failed"))?,
                DynVec::with_capacity(256, &allocators.global_alloc)
                    .ok_or_else(|| ArrayString::from_str("global alloc failed"))?)
                .map_err(|e|
                    array_format!("failed to create buffer allocators ( {} )", e)
                )?,
            _memory_layout: memory_layout,
        })
    }

    pub fn device_info(&self) -> &PhysicalDeviceInfo {
        self.vulkan_context.physical_device_info()
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.vulkan_context.request_resize(size);
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
        let Some(frame_data) = swapchain_context
            .setup_image(&device, &swapchain_loader)
            .map_err(|e| {
                array_format!("failed to setup render image ( {} )", e)
            })? else {
                self.vulkan_context.swapchain_state(SwapchainState::OutOfDate(window.inner_size()));
                return Ok(())
            };
        if let Err(e) = helpers::begin_command_buffer(&device, frame_data.command_buffer) {
            return Err(array_format!("failed to begin command buffer {:?}", e))
        }
        let frame = Frame::new(
            Handle::new(device.clone()),
            Handle::new(frame_data.command_buffer),
            RefCell::new(ImageResource::new(
                Handle::new(frame_data.image),
                Handle::new(frame_data.image_view),
                SwapchainContext::image_subresource_range(),
                frame_data.image_state,
                frame_data.format,
                frame_data.extent,
            )),
            StackGuard::new(&allocators.swapchain),
            StackGuard::new(&allocators.init),
            queue_family_indices);
        interface
            .render(&frame)
            .map_err(|e| {
                array_format!("interface failed to render ( {} )", e)
            })?;
        let (submit_info, fence) = swapchain_context
            .setup_submit(
                &device,
                frame.swapchain_image_resource().state(),
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
            .map_err(|e| array_format!("queue present failed {}", e
            ))?;
        if present_result != PresentResult::Success || frame_data.suboptimal {
            self.vulkan_context.swapchain_state(SwapchainState::OutOfDate(window.inner_size()))
        }
        Ok(())
    }

    pub fn clean_up(&mut self, allocators: &'mem Allocators) {
        println!("Nox renderer message: terminating renderer");
        self.vulkan_context.destroy_swapchain(self.main_thread_context.graphics_pool(), &allocators);
    }
}
