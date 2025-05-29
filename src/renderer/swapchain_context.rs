use super::{
    image_state::ImageState,
    helpers::allocate_command_buffers,
};

use crate::{
    allocator_traits::AllocateExt,
    stack_allocator::{StackAllocator, StackGuard, StackRegion},
    string::{LargeError, SmallError, String},
    utility::{clamp, has_bit, has_not_bit},
    vec_types::{FixedVec, VecOperations},
};

use ash::{khr::{surface, swapchain}, vk::{self, Handle}};

use core::panic;
use std::{slice, mem::ManuallyDrop};

pub struct FrameData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub command_buffer: vk::CommandBuffer,
    pub image_index: u32,
    pub image_state: ImageState,
    pub suboptimal: bool,
}

impl FrameData {

    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
        image_state: ImageState,
        suboptimal: bool,
    ) -> Self {
        Self {
            image,
            image_view,
            command_buffer,
            image_index,
            image_state,
            suboptimal,
        }
    }
}

#[derive(Default)]
struct TiedResources {
    image_view: vk::ImageView,
    present_wait_semaphore: vk::Semaphore,
}

impl TiedResources {

    fn new(
        device: &ash::Device,
        image: vk::Image,
        image_format: vk::Format,
    ) -> Result<Self, SmallError> {
        let image_view_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: image_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        let image_view = unsafe { device
            .create_image_view(&image_view_create_info, None)
            .map_err(|e| {
                String::format(format_args!("failed to create image view {}", e))
            })?};
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let present_wait_semaphore = unsafe { device
            .create_semaphore(&semaphore_create_info, None)
            .map_err(|e| {
                String::format(format_args!("failed to create semaphore {}", e))
        })?};
        Ok(
            Self {
                image_view,
                present_wait_semaphore,
            }
        )
    }

    fn destroy(
        &mut self,
        device: &ash::Device,
    )
    {
        unsafe {
            if !self.image_view.is_null() {
                device.destroy_image_view(self.image_view, None);
                self.image_view = vk::ImageView::null();
            }
            if !self.present_wait_semaphore.is_null() {
                device.destroy_semaphore(self.present_wait_semaphore, None);
                self.present_wait_semaphore = vk::Semaphore::null();
            }
        }
    }
}

#[derive(Default)]
struct UntiedResources {
    frame_ready_fence: vk::Fence,
    image_ready_semaphore: vk::Semaphore,
}

impl UntiedResources {

    pub fn new(device: &ash::Device) -> Result<Self, SmallError> {
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        let frame_ready_fence = unsafe { device
            .create_fence(&fence_create_info, None)
            .map_err(|e| {
                String::format(format_args!("failed to create fence {}", e))
            })?
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let image_ready_semaphore = unsafe { device
            .create_semaphore(&semaphore_create_info, None)
            .map_err(|e| {
                String::format(format_args!("failed to create semaphore {}", e))
            })?
        };
        Ok(Self {
            frame_ready_fence,
            image_ready_semaphore,
        })
    }

    fn destroy(
        &mut self,
        device: &ash::Device,
    )
    {
        unsafe {
            if !self.frame_ready_fence.is_null() {
                device.destroy_fence(self.frame_ready_fence, None);
                self.frame_ready_fence = vk::Fence::null();
            }
            if !self.image_ready_semaphore.is_null() {
                device.destroy_semaphore(self.image_ready_semaphore, None);
                self.image_ready_semaphore = vk::Semaphore::null();
            }
        }
    }
}

struct Resources<'a> {
    tied_resources: FixedVec<'a, ManuallyDrop<TiedResources>>,
    untied_resources: FixedVec<'a, ManuallyDrop<UntiedResources>>,
    command_buffers: FixedVec<'a, vk::CommandBuffer>,
    untied_resource_index: usize,
}

impl<'a> Resources<'a> {

    pub fn new(
        device: &ash::Device,
        images: &FixedVec<'a, vk::Image>,
        image_format: vk::Format,
        command_pool: vk::CommandPool,
        allocator: &mut StackRegion<'a>
    ) -> Result<Self, SmallError>
    {
        let image_count = images.len();
        let Some(mut command_buffers) = FixedVec::new_with_default(image_count, image_count, allocator) else {
            return Err(String::from_str("failed to allocate CPU resources"))
        };
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: image_count as u32,
            ..Default::default()
        };
        if let Err(e) = allocate_command_buffers(device, &command_buffer_alloc_info, &mut command_buffers) {
            return Err(String::format(format_args!(
                "failed to allocate command buffers {:?}", e,
            )))
        }
        let Some(mut tied_resources) = FixedVec::<ManuallyDrop<TiedResources>>::new(image_count, allocator) else {
            return Err(String::from_str("failed to allocate CPU resources"))
        };
        let Some(mut untied_resources) = FixedVec::<ManuallyDrop<UntiedResources>>::new(image_count, allocator) else {
            return Err(String::from_str("failed to allocate"))
        };
        for i in 0..image_count {
            tied_resources.push_back(
                match TiedResources::new(device, images[i], image_format) {
                    Ok(r) => ManuallyDrop::new(r),
                    Err(e) => {
                        unsafe {
                            device.free_command_buffers(command_pool, command_buffers.as_slice());
                        }
                        for j in 0..i {
                            tied_resources[j].destroy(device);
                            untied_resources[j].destroy(device);
                        }
                        return Err(e)
                    },
                }
            );
            untied_resources.push_back(
                match UntiedResources::new(device) {
                    Ok(r) => ManuallyDrop::new(r),
                    Err(e) => {
                        unsafe {
                            device.free_command_buffers(command_pool, command_buffers.as_slice());
                        }
                        for j in 0..i {
                            tied_resources[j].destroy(device);
                            untied_resources[j].destroy(device);
                        }
                        tied_resources[i].destroy(device);
                        return Err(e)
                    },
                }
            );
        }
        Ok(Self {
            tied_resources,
            untied_resources,
            command_buffers,
            untied_resource_index: 0,
        })
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        queue: vk::Queue,
        command_pool: Option<vk::CommandPool>,
    ) {
        unsafe {
            if device.queue_wait_idle(queue).is_err() && device.device_wait_idle().is_err() {
                panic!("GPU hang")
            }
        }
        for resource in &mut self.tied_resources {
            resource.destroy(device);
        }
        self.tied_resources.clear();
        for resource in &mut self.untied_resources {
            resource.destroy(device);
        }
        self.untied_resources.clear();
        if let Some(command_pool) = command_pool {
            if self.command_buffers.len() != 0 {
                unsafe { device.free_command_buffers(command_pool, self.command_buffers.as_slice()); }
            }
            self.command_buffers.clear();
        }
    }

    pub fn get_tied_resources(&self, image_index: u32) -> &TiedResources {
        &self.tied_resources[image_index as usize]
    }

    pub fn get_current_untied_resources(&self) -> (&UntiedResources, &vk::CommandBuffer) {
        (&self.untied_resources[self.untied_resource_index], &self.command_buffers[self.untied_resource_index])
    }

    pub fn increment_untied_resource_index(&mut self) {
        self.untied_resource_index = (self.untied_resource_index + 1) % self.untied_resources.len();
    }
}

#[derive(PartialEq, Eq)]
pub enum PresentResult {
    Success,
    Suboptimal,
    OutOfDate,
}

pub struct SwapchainContext<'a> {
    resources: Resources<'a>,
    images: FixedVec<'a, vk::Image>,
    image_states: FixedVec<'a, ImageState>,
    pub handle: vk::SwapchainKHR,
    pub current_image_index: u32,
    local_allocator: StackRegion<'a>,
    //pub image_usage: vk::ImageUsageFlags,
}

impl<'a> SwapchainContext<'a> {

    pub fn new(
        device: &ash::Device,
        surface_loader: &surface::Instance,
        swapchain_loader: &swapchain::Device,
        physical_device: vk::PhysicalDevice,
        surface_handle: vk::SurfaceKHR,
        framebuffer_extent: vk::Extent2D,
        graphics_command_pool: vk::CommandPool,
        graphics_queue_family_index: u32,
        mut local_allocator: StackRegion<'a>,
        init_allocator: &mut StackAllocator,
    ) -> Result<Option<Self>, LargeError>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Ok(None)
        }
        let surface_format = match find_surface_format(surface_loader, physical_device, surface_handle, init_allocator) {
            Ok(format) => format,
            Err(err) => return Err(String::from_str(err.as_str())),
        };
        let present_mode = match find_present_mode(surface_loader, physical_device, surface_handle, init_allocator) {
            Ok(mode) => mode,
            Err(err) => return Err(String::from_str(err.as_str())),
        };
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_handle)
                .map_err(|e| {
                    String::format(format_args!("failed to get surface capabilities {:?}", e))
                })?
        };
        let mut image_extent = capabilities.current_extent;
        if image_extent.width == u32::MAX {
            image_extent.width = clamp(
                framebuffer_extent.width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width
            );
            image_extent.height = clamp(
                framebuffer_extent.height,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height
            );
        }
        if image_extent.width == 0 || image_extent.height == 0 {
            return Err(String::from_str("swapchain extent size was zero"));
        }
        let mut min_image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && min_image_count > capabilities.max_image_count {
            min_image_count = capabilities.max_image_count;
        }
        let mut pre_transform = capabilities.current_transform;
        if has_bit!(capabilities.supported_transforms, vk::SurfaceTransformFlagsKHR::IDENTITY) {
            pre_transform = vk::SurfaceTransformFlagsKHR::IDENTITY;
        }
        let mut composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        if has_bit!(capabilities.supported_composite_alpha, vk::CompositeAlphaFlagsKHR::INHERIT) {
            composite_alpha = vk::CompositeAlphaFlagsKHR::INHERIT;
        }
        let image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        if has_not_bit!(capabilities.supported_usage_flags, image_usage) {
            return Err(String::from_str("swapchain does not support color attachment usage"))
        }
        //image_usage |= capabilities.supported_usage_flags & vk::ImageUsageFlags::TRANSFER_DST;
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface_handle,
            min_image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent,
            image_array_layers: 1,
            image_usage,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform,
            composite_alpha,
            present_mode,
            ..Default::default()
        };
        let swapchain_handle = unsafe {
            match swapchain_loader.create_swapchain(&create_info, None) {
                Ok(swapchain) => swapchain,
                Err(result) => return Err(String::format(format_args!("failed to create swapchain {:?}", result))),
            }
        };
        let get_swapchain_images_khr = swapchain_loader.fp().get_swapchain_images_khr;
        let mut image_count = 0u32;
        let mut result = unsafe { get_swapchain_images_khr(device.handle(), swapchain_handle, &mut image_count, std::ptr::null_mut()) };
        if image_count == 0 || result != vk::Result::SUCCESS {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            return Err(String::format(format_args!("failed to get swapchain image count {:?}", result)))
        }
        let Some(mut images) = FixedVec
            ::new_with_default(
                image_count as usize,
                image_count as usize,
                &mut local_allocator
            ) else {
                unsafe {
                    swapchain_loader.destroy_swapchain(swapchain_handle, None);
                }
                return Err(String::from_str("failed to allocate image handles")
            )};
        let Some(image_states) = FixedVec
            ::new_with(
                image_count as usize,
                image_count as usize,
                |_i| {
                    ImageState::new(
                        vk::AccessFlags::NONE,
                        vk::ImageLayout::UNDEFINED,
                        graphics_queue_family_index, 
                        vk::PipelineStageFlags::TOP_OF_PIPE
                    )},
                &mut local_allocator
            ) else {
                unsafe {
                    swapchain_loader.destroy_swapchain(swapchain_handle, None);
                }
                return Err(String::from_str("failed to allocate image states")
            )};
        result = unsafe { get_swapchain_images_khr(device.handle(), swapchain_handle, &mut image_count, images.as_mut_ptr() as _) };
        if result != vk::Result::SUCCESS {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            return Err(String::format(format_args!("failed to get swapchain ")))
        }
        let resources = Resources::new(
            &device,
            &images,
            surface_format.format,
            graphics_command_pool,
            &mut local_allocator,
        ).map_err(|e| {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            String::format(format_args!(
                "failed to create resources ( {} )", e
            ))
        })?;
        Ok(
            Some(
                Self {
                    local_allocator,
                    handle: swapchain_handle,
                    resources,
                    images,
                    image_states,
                    current_image_index: 0,
                }
            )
        )
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device,
        grapchis_queue: vk::Queue,
        graphics_command_pool: Option<vk::CommandPool>
    )
    {
        self.resources.destroy(device, grapchis_queue, graphics_command_pool);
        unsafe { swapchain_loader.destroy_swapchain(self.handle, None); }
        self.local_allocator.clear();
    }

    pub const fn image_subresource_range() -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }

    pub const fn frame_timeout() -> u64 {
        1_000_000_000
    }

    pub fn setup_image(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device
    ) -> Result<Option<FrameData>, SmallError>
    {
        self.resources.increment_untied_resource_index();
        let (untied_resources, command_buffer) = self.resources.get_current_untied_resources();
        let fences = slice::from_ref(&untied_resources.frame_ready_fence);
        unsafe { device
            .wait_for_fences(
                fences,
                true,
                Self::frame_timeout())
            .map_err(|e| {
                String::format(format_args!("failed to wait for fence {:?}", e))
            })?};
        unsafe { device
            .reset_fences(fences)
            .map_err(|e| {
                String::format(format_args!("failed to reset fence {:?}", e))
            })?};
        let next_image = unsafe { match swapchain_loader
            .acquire_next_image(
                self.handle,
                Self::frame_timeout(),
                untied_resources.image_ready_semaphore,
                vk::Fence::null()
            ) {
                Ok(r) => r,
                Err(e) => {
                    if e == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        return Ok(None)
                    }
                    return Err(String::format(format_args!("failed to acquire next image {:?}", e)))
                }
            }};
        self.current_image_index = next_image.0;
        let tied_resources = self.resources.get_tied_resources(self.current_image_index);
        let image_index = self.current_image_index as usize;
        Ok(Some(
            FrameData::new(
                self.images[image_index],
                tied_resources.image_view,
                *command_buffer,
                next_image.0,
                self.image_states[image_index],
                next_image.1,
            )
        ))
    }

    pub fn setup_submit(
        &mut self,
        device: &ash::Device,
        src_image_state: ImageState,
        graphics_queue_index: u32,
    ) -> (vk::SubmitInfo, vk::Fence) {
        let tied_resources = self.resources.get_tied_resources(self.current_image_index);
        let (untied_resources, command_buffer) = self.resources.get_current_untied_resources();
        let image_index = self.current_image_index as usize;
        let dst_image_state = ImageState::new(
            vk::AccessFlags::NONE,
            vk::ImageLayout::PRESENT_SRC_KHR,
            graphics_queue_index,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        let memory_barrier = src_image_state.to_memory_barrier(
            self.images[image_index],
            &dst_image_state,
            Self::image_subresource_range()
        );
        unsafe {
            device.cmd_pipeline_barrier(
                *command_buffer,
                src_image_state.pipeline_stage,
                dst_image_state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                slice::from_ref(&memory_barrier)
            );
        }
        self.image_states[image_index] = dst_image_state;
        const WAIT_STAGE_MASK: vk::PipelineStageFlags = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
        (
            vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                wait_semaphore_count: 1,
                p_wait_semaphores: &untied_resources.image_ready_semaphore,
                p_wait_dst_stage_mask: &WAIT_STAGE_MASK,
                command_buffer_count: 1,
                p_command_buffers: command_buffer,
                signal_semaphore_count: 1,
                p_signal_semaphores: &tied_resources.present_wait_semaphore,
                ..Default::default()
            },
            untied_resources.frame_ready_fence,
        )
    }

    pub fn present_submit(
        &mut self,
        swapchain_loader: &swapchain::Device,
        queue: vk::Queue,
    ) -> Result<PresentResult, SmallError> {
        let tied_resources = self.resources.get_tied_resources(self.current_image_index);
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: &tied_resources.present_wait_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.handle,
            p_image_indices: &self.current_image_index,
            ..Default::default()
        };
        unsafe {
            match swapchain_loader.queue_present(queue, &present_info) {
                Ok(r) => {
                    Ok(if r { PresentResult::Suboptimal } else { PresentResult::Success })
                }
                Err(e) => {
                    if e == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        Ok(PresentResult::OutOfDate)
                    }
                    else {
                        Err(String::format(format_args!("queue present failed {:?}", e)))
                    }
                }
            }
        }
    }
}

fn find_surface_format(
    surface_loader: &surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface_handle: vk::SurfaceKHR,
    allocator: &mut StackAllocator,
) -> Result<vk::SurfaceFormatKHR, SmallError>
{
    unsafe {
        let mut stack = StackGuard::new(allocator);
        let get_physical_device_surface_formats_khr = surface_loader.fp().get_physical_device_surface_formats_khr;
        let mut count = 0u32;
        let mut result = get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            std::ptr::null_mut(),
        );
        if count == 0 || result != vk::Result::SUCCESS {
            return Err(SmallError::format(format_args!("failed to get surface format count {:?}", result)))
        }
        let formats_ptr: *mut vk::SurfaceFormatKHR = match stack.allocate_uninit(count as usize) {
            Some(formats) => formats.as_ptr(),
            None => return Err(SmallError::from_str("main thread stack out of memory")),
        };
        result = get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            formats_ptr,
        );
        if result != vk::Result::SUCCESS {
            return Err(SmallError::format(format_args!("failed to get surface formats {:?}", result)))
        }
        let formats: &[vk::SurfaceFormatKHR] = std::slice::from_raw_parts(formats_ptr, count as usize);
        for format in formats {
            if format.format == vk::Format::R8G8B8A8_SRGB &&
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return Ok(*format);
            }
        }
        return Ok(formats[0]);
    }
}

fn find_present_mode(
    surface_loader: &surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface_handle: vk::SurfaceKHR,
    allocator: &mut StackAllocator,
) -> Result<vk::PresentModeKHR, SmallError>
{
    unsafe {
        let mut stack = StackGuard::new(allocator);
        let get_physical_device_surface_present_modes_khr = surface_loader.fp().get_physical_device_surface_present_modes_khr;
        let mut count = 0u32;
        let mut result = get_physical_device_surface_present_modes_khr(
            physical_device,
            surface_handle,
            &mut count,
            std::ptr::null_mut(),
        );
        if count == 0 || result != vk::Result::SUCCESS {
            return Err(SmallError::format(format_args!("failed to get surface present mode count {:?}", result)))
        }
        let modes_ptr: *mut vk::PresentModeKHR = match stack.allocate_uninit(count as usize) {
            Some(modes) => modes.as_ptr(),
            None => return Err(SmallError::from_str("main thread stack out of memory")),
        };
        result = get_physical_device_surface_present_modes_khr(
            physical_device,
            surface_handle,
            &mut count,
            modes_ptr,
        );
        if result != vk::Result::SUCCESS {
            return Err(SmallError::format(format_args!("failed to get surface present modes {:?}", result)))
        }
        let modes: &[vk::PresentModeKHR] = std::slice::from_raw_parts(modes_ptr, count as usize);
        for mode in modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return Ok(vk::PresentModeKHR::MAILBOX); // low latency and no tearing
            }
        }
        return Ok(vk::PresentModeKHR::FIFO); // always supported
    }
}
