use super::image_state::ImageState;

use crate::{
    allocator_traits::AllocateExt,
    stack_allocator::{StackAllocator, StackGuard, StackRegion},
    string::{String, SmallError},
    utility::{clamp, has_bit, has_not_bit},
    vec_types::{VecOperations, FixedVec},
};

use std::slice;

use ash::{khr::{surface, swapchain}, vk::{self, AccessFlags, Fence, Handle}};

pub struct ImageData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_index: u32,
    pub image_state: ImageState,
    pub suboptimal: bool,
}

impl ImageData {

    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        image_index: u32,
        image_state: ImageState,
        suboptimal: bool,
    ) -> Self {
        Self {
            image,
            image_view,
            image_index,
            image_state,
            suboptimal,
        }
    }
}

pub struct SwapchainResources<'a> {
    images: FixedVec<'a, vk::Image>,
    image_views: FixedVec<'a, vk::ImageView>,
    frame_ready_fences: FixedVec<'a, vk::Fence>,
    image_ready_semaphores: FixedVec<'a, vk::Semaphore>,
    present_wait_semaphores: FixedVec<'a, vk::Semaphore>,
}

impl<'a> SwapchainResources<'a> {

    pub fn new(
        image_count: usize,
        allocator: &mut StackRegion<'a>
    ) -> Option<Self>
    {
        let images = FixedVec::new_with_default(image_count, image_count, allocator)?;
        let image_views = FixedVec::new_with_default(image_count, image_count, allocator)?;
        let frame_ready_fences = FixedVec::new_with_default(image_count, image_count, allocator)?;
        let image_ready_semaphores = FixedVec::new_with_default(image_count, image_count, allocator)?;
        let present_wait_semaphores = FixedVec::new_with_default(image_count, image_count, allocator)?;
        Some(Self {
            images,
            image_views,
            frame_ready_fences,
            image_ready_semaphores,
            present_wait_semaphores,
        })
    }

    pub fn destroy_resources(&mut self, device: &ash::Device) {
        for i in 0..self.images.len() {
            unsafe {
                let image_view = self.image_views[i];
                if !image_view.is_null() {
                    device.destroy_image_view(image_view, None);
                }
                let fence = self.frame_ready_fences[i];
                if !fence.is_null() {
                    device.destroy_fence(fence, None);
                }
                let semaphore1 = self.image_ready_semaphores[i];
                if !semaphore1.is_null() {
                    device.destroy_semaphore(semaphore1, None);
                }
                let semaphore2 = self.present_wait_semaphores[i];
                if !semaphore2.is_null() {
                    device.destroy_semaphore(semaphore2, None);
                }
            }
        }
        self.images.clear();
        self.image_views.clear();
        self.frame_ready_fences.clear();
        self.image_ready_semaphores.clear();
        self.present_wait_semaphores.clear();
    }
}

pub enum PresentResult {
    Success,
    Suboptimal,
    OutOfDate,
}

pub struct SwapchainContext<'a> {
    _local_allocator: StackRegion<'a>,
    resources: SwapchainResources<'a>,
    image_states: FixedVec<'a, ImageState>,
    pub handle: vk::SwapchainKHR,
    pub image_count: u32,
    pub current_image: u32,
    pub current_sync_object: u32,
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
        mut local_allocator: StackRegion<'a>,
        init_allocator: &mut StackAllocator,
    ) -> Result<Option<Self>, SmallError>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Ok(None)
        }
        let surface_format = match find_surface_format(surface_loader, physical_device, surface_handle, init_allocator) {
            Ok(format) => format,
            Err(err) => return Err(err),
        };
        let present_mode = match find_present_mode(surface_loader, physical_device, surface_handle, init_allocator) {
            Ok(mode) => mode,
            Err(err) => return Err(err),
        };
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device,surface_handle)
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
        let Some(image_states) = FixedVec
            ::new_with_default(
                image_count as usize,
                image_count as usize,
                &mut local_allocator
            ) else {
                unsafe {
                    swapchain_loader.destroy_swapchain(swapchain_handle, None);
                }
                return Err(String::from_str("failed to allocate image states")
            )};
        let Some(mut resources) = SwapchainResources::new(image_count as usize, &mut local_allocator) else {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            return Err(String::from_str("failed to allocate swapchain resources"))
        };
        unsafe {
            result = get_swapchain_images_khr(device.handle(), swapchain_handle, &mut image_count, resources.images.as_mut_ptr());
            if result != vk::Result::SUCCESS {
                swapchain_loader.destroy_swapchain(swapchain_handle, None);
                return Err(String::format(format_args!("failed to get swapchain ")))
            }
            let fence_create_info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            let semaphore_create_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            };
            for i in 0..image_count as usize {
                let image_view_create_info = vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                    image: resources.images[i],
                    view_type: vk::ImageViewType::TYPE_2D,
                    format: surface_format.format,
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
                resources.image_views[i] = device
                    .create_image_view(&image_view_create_info, None)
                    .map_err(|e| {
                        resources.destroy_resources(device);
                        swapchain_loader.destroy_swapchain(swapchain_handle, None);
                        String::format(format_args!("failed to create image view {}", e))
                })?;
                resources.frame_ready_fences[i] = device
                    .create_fence(&fence_create_info, None)
                    .map_err(|e| {
                        resources.destroy_resources(device);
                        swapchain_loader.destroy_swapchain(swapchain_handle, None);
                        String::format(format_args!("failed to create fence {}", e))
                })?;
                resources.image_ready_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .map_err(|e| {
                        resources.destroy_resources(device);
                        swapchain_loader.destroy_swapchain(swapchain_handle, None);
                        String::format(format_args!("failed to create semaphore {}", e))
                })?;
                resources.present_wait_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .map_err(|e| {
                        resources.destroy_resources(device);
                        swapchain_loader.destroy_swapchain(swapchain_handle, None);
                        String::format(format_args!("failed to create semaphore {}", e))
                })?;
            }
        }
        Ok(
            Some(
                Self {
                    _local_allocator: local_allocator,
                    handle: swapchain_handle,
                    resources,
                    image_states,
                    image_count,
                    current_image: 0,
                    current_sync_object: 0,
                }
            )
        )
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device,
    ) {
        self.resources.destroy_resources(device);
        unsafe { swapchain_loader.destroy_swapchain(self.handle, None); }
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
    ) -> Result<Option<ImageData>, SmallError>
    {
        let current_sync_object = self.current_sync_object as usize;
        unsafe {
            let fences = std::slice::from_ref(&self.resources.frame_ready_fences[current_sync_object]);
            device
                .wait_for_fences(
                    fences,
                    true,
                    Self::frame_timeout())
                .map_err(|e| {
                    String::format(format_args!("failed to wait for fence {:?}", e))
                })?;
            device
                .reset_fences(fences)
                .map_err(|e| {
                    String::format(format_args!("failed to reset fence {:?}", e))
                })?;
            let next_image = match swapchain_loader
                .acquire_next_image(
                    self.handle,
                    0,
                    self.resources.image_ready_semaphores[current_sync_object],
                    Fence::null()
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        if e == vk::Result::ERROR_OUT_OF_DATE_KHR {
                            return Ok(None)
                        }
                        return Err(String::format(format_args!("failed to acquire next image {:?}", e)))
                    }
                };
            let image_index = next_image.0 as usize;
            Ok(Some(
                ImageData::new(
                    self.resources.images[image_index],
                    self.resources.image_views[image_index],
                    next_image.0,
                    self.image_states[image_index],
                    next_image.1,
                )
            ))
        }
    }

    pub fn setup_submit(
        &mut self,
        device: &ash::Device,
        command_buffer: &vk::CommandBuffer,
        src_image_state: ImageState,
        graphics_queue_index: u32,
    ) -> (vk::SubmitInfo, vk::Fence) {
        let image_index = self.current_image as usize;
        let sync_object_index = self.current_sync_object as usize;
        let dst_image_state = ImageState::new(
            AccessFlags::NONE,
            vk::ImageLayout::PRESENT_SRC_KHR,
            graphics_queue_index,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        let memory_barrier = src_image_state.to_memory_barrier(
            self.resources.images[image_index],
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
                p_wait_semaphores: &self.resources.image_ready_semaphores[sync_object_index],
                p_wait_dst_stage_mask: &WAIT_STAGE_MASK,
                command_buffer_count: 1,
                p_command_buffers: command_buffer,
                signal_semaphore_count: 1,
                p_signal_semaphores: &self.resources.present_wait_semaphores[sync_object_index],
                ..Default::default()
            },
            self.resources.frame_ready_fences[self.current_sync_object as usize]
        )
    }

    pub fn present_submit(
        &mut self,
        swapchain_loader: &swapchain::Device,
        queue: vk::Queue,
    ) -> Result<PresentResult, SmallError> {
        let sync_object_index = self.current_sync_object as usize;
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.resources.present_wait_semaphores[sync_object_index],
            swapchain_count: 1,
            p_swapchains: &self.handle,
            p_image_indices: &self.current_image,
            ..Default::default()
        };
        unsafe {
            match swapchain_loader.queue_present(queue, &present_info) {
                Ok(r) => Ok(if r { PresentResult::Suboptimal } else { PresentResult::Success }),
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
