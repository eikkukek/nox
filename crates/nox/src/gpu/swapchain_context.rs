use core::{
    slice,
    ptr,
    num::NonZeroI32,
    fmt::{self, Display, Formatter}
};

use ash::{khr::{surface, swapchain}, vk};

use compact_str::format_compact;

use nox_mem::{vec_types::{Vector, FixedVec}};

use nox_alloc::arena_alloc::*;

use crate::{
    dev::{error::{Result, Error, Context, location}, utility::clamp},
    has_bits, has_not_bits,
};

use super::{
    image::{ImageState, ImageSubresourceRangeInfo, ImageAspect},
    helpers,
};

pub struct FrameData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub command_buffer: vk::CommandBuffer,
    pub frame_index: u32,
    pub image_state: ImageState,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub suboptimal: bool,
}

impl FrameData {

    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        command_buffer: vk::CommandBuffer,
        frame_index: u32,
        image_state: ImageState,
        format: vk::Format,
        extent: vk::Extent2D,
        suboptimal: bool,
    ) -> Self {
        Self {
            image,
            image_view,
            command_buffer,
            frame_index,
            image_state,
            format,
            extent,
            suboptimal,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct TiedResources {
    image_view: vk::ImageView,
    present_wait_semaphore: vk::Semaphore,
}

impl TiedResources {

    fn new(
        device: &ash::Device,
        image: vk::Image,
        image_format: vk::Format,
    ) -> Result<Self, vk::Result> {
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
            .create_image_view(&image_view_create_info, None)?
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let present_wait_semaphore = unsafe { device
            .create_semaphore(&semaphore_create_info, None)?
        };
        Ok(Self {
            image_view,
            present_wait_semaphore,
        })
    }

    fn destroy(
        &mut self,
        device: &ash::Device,
    )
    {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_semaphore(self.present_wait_semaphore, None);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct UntiedResources {
    frame_ready_fence: vk::Fence,
    image_ready_semaphore: vk::Semaphore,
}

impl UntiedResources {

    pub fn new(device: &ash::Device) -> Result<Self, vk::Result> {
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        let frame_ready_fence = unsafe { device
            .create_fence(&fence_create_info, None)?
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let image_ready_semaphore = unsafe { device
            .create_semaphore(&semaphore_create_info, None)?
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
            device.destroy_fence(self.frame_ready_fence, None);
            device.destroy_semaphore(self.image_ready_semaphore, None);
        }
    }
}

struct Resources<'a> {
    tied_resources: FixedVec<'a, TiedResources, ArenaAlloc>,
    untied_resources: FixedVec<'a, UntiedResources, ArenaAlloc>,
    command_buffers: FixedVec<'a, vk::CommandBuffer, ArenaAlloc>,
}

impl<'a> Resources<'a> {

    fn new(
        device: &ash::Device,
        images: &FixedVec<'a, vk::Image, ArenaAlloc>,
        buffered_frame_count: u32,
        image_format: vk::Format,
        command_pool: vk::CommandPool,
        allocator: &'a ArenaAlloc
    ) -> Result<Self>
    {
        let image_count = images.len();
        let mut command_buffers = FixedVec
            ::with_capacity(image_count, allocator)?;
        command_buffers.resize(image_count, Default::default()).unwrap();
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: image_count as u32,
            ..Default::default()
        };
        helpers
            ::allocate_command_buffers(device, &command_buffer_alloc_info, &mut command_buffers)
            .context_with(|| format_compact!("failed to allocate command buffers at {}", location!()))?;
        let mut tied_resources = FixedVec::<TiedResources, ArenaAlloc>
            ::with_capacity(image_count, allocator)?;
        for i in 0..image_count {
            tied_resources
                .push(match TiedResources::new(device, images[i], image_format) {
                    Ok(resources) => resources,
                    Err(err) => {
                        unsafe {
                            device.free_command_buffers(command_pool, command_buffers.as_slice());
                        }
                        for j in 0..i {
                            tied_resources[j].destroy(device);
                        }
                        return Err(Error::new("failed to create tied resources", err))
                    },
                }).unwrap();
        }
        let mut untied_resources = FixedVec::<UntiedResources, ArenaAlloc>
            ::with_capacity(buffered_frame_count as usize, allocator)?;
        for i in 0..buffered_frame_count as usize {
            untied_resources
                .push(
                    match UntiedResources::new(device) {
                        Ok(resources) => resources,
                        Err(err) => {
                            unsafe {
                                device.free_command_buffers(command_pool, command_buffers.as_slice());
                            }
                            for j in 0..image_count {
                                tied_resources[j].destroy(device);
                            }
                            for k in 0..i {
                                untied_resources[k].destroy(device);
                            }
                            return Err(Error::new("failed to create untied resources", err))
                        },
                })
                .unwrap();
        }
        Ok(Self {
            tied_resources,
            untied_resources,
            command_buffers,
        })
    }

    fn destroy(
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
                unsafe {
                    device.free_command_buffers(command_pool, self.command_buffers.as_slice());
                }
            }
            self.command_buffers.clear();
        }
    }

    #[inline(always)]
    fn get_tied_resources(&self, image_index: u32) -> &TiedResources {
        &self.tied_resources[image_index as usize]
    }

    #[inline(always)]
    fn buffered_frame_count(&self) -> u32 {
        self.untied_resources.len() as u32
    }

    #[inline(always)]
    fn get_untied_resources(&self, frame_index: u32) -> (&UntiedResources, &vk::CommandBuffer) {
        let index = frame_index as usize;
        (&self.untied_resources[index], &self.command_buffers[index])
    }
}

#[derive(PartialEq, Eq)]
pub enum PresentResult {
    Success,
    Suboptimal,
    OutOfDate,
}

pub struct SubmitSemaphores {
    pub wait_semaphore: vk::Semaphore,
    pub wait_stage: vk::PipelineStageFlags,
    pub signal_semaphore: vk::Semaphore,
}

pub struct SwapchainContext<'a> {
    resources: Resources<'a>,
    images: FixedVec<'a, vk::Image, ArenaAlloc>,
    image_states: FixedVec<'a, ImageState, ArenaAlloc>,
    handle: vk::SwapchainKHR,
    frame_index: u32,
    image_index: u32,
    surface_format: vk::SurfaceFormatKHR,
    image_extent: vk::Extent2D,
}

impl<'a> SwapchainContext<'a> {

    pub fn new(
        device: &ash::Device,
        surface_loader: &surface::Instance,
        swapchain_loader: &swapchain::Device,
        physical_device: vk::PhysicalDevice,
        surface_handle: vk::SurfaceKHR,
        framebuffer_extent: vk::Extent2D,
        mut buffered_frame_count: u32,
        graphics_command_pool: vk::CommandPool,
        graphics_queue_family_index: u32,
        local_allocator: &'a ArenaAlloc,
        init_allocator: &ArenaAlloc,
    ) -> Result<Option<Self>>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Ok(None)
        }
        let surface_format = find_surface_format(
            surface_loader, physical_device,
            surface_handle, init_allocator
        )?;
        let present_mode = find_present_mode(
            surface_loader, physical_device,
            surface_handle, init_allocator
        )?;
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_handle)
                .context("failed to get physical device surface capabilities")?
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
            return Err(Error::just_context("swapchain extent was 0"));
        }
        let mut actual_image_count = capabilities.min_image_count + 1;
        actual_image_count = actual_image_count.max(buffered_frame_count);
        if capabilities.max_image_count > 0 && actual_image_count > capabilities.max_image_count {
            actual_image_count = capabilities.max_image_count;
            if actual_image_count < buffered_frame_count {
                buffered_frame_count = actual_image_count;
            }
        }
        let mut pre_transform = capabilities.current_transform;
        if has_bits!(capabilities.supported_transforms, vk::SurfaceTransformFlagsKHR::IDENTITY) {
            pre_transform = vk::SurfaceTransformFlagsKHR::IDENTITY;
        }
        let mut composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        if has_bits!(capabilities.supported_composite_alpha, vk::CompositeAlphaFlagsKHR::INHERIT) {
            composite_alpha = vk::CompositeAlphaFlagsKHR::INHERIT;
        }
        let image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        if has_not_bits!(capabilities.supported_usage_flags, image_usage) {
            return Err(Error::just_context(format_compact!("unsupported swapchain, missing usage {image_usage:?}")))
        }
        //image_usage |= capabilities.supported_usage_flags & vk::ImageUsageFlags::TRANSFER_DST;
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface_handle,
            min_image_count: actual_image_count,
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
                Err(err) => return Err(Error::new("failed to create swapchain", err)),
            }
        };
        let get_swapchain_images_khr = swapchain_loader.fp().get_swapchain_images_khr;
        let mut image_count = 0u32;
        let mut result = unsafe {
            get_swapchain_images_khr(device.handle(), swapchain_handle, &mut image_count, ptr::null_mut())
        };
        if image_count == 0 || result != vk::Result::SUCCESS {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            return Err(Error::new("failed to get swapchain images at creation", result))
        }
        let mut images = FixedVec
            ::with_len(
                image_count as usize,
                Default::default(),
                local_allocator
            )?;
        let image_states = FixedVec
            ::with_len(
                images.len(),
                ImageState::new(
                    vk::AccessFlags::NONE,
                    vk::ImageLayout::UNDEFINED,
                    graphics_queue_family_index, 
                    vk::PipelineStageFlags::TOP_OF_PIPE
                ),
                local_allocator
            )?;
        result = unsafe {
            get_swapchain_images_khr(
                device.handle(),
                swapchain_handle,
                &mut image_count,
                images.as_mut_ptr() as _
            )
        };
        if result != vk::Result::SUCCESS {
            unsafe { swapchain_loader.destroy_swapchain(swapchain_handle, None); }
            return Err(Error::new("failed to get swapchain images at creation", result))
        }
        let resources = Resources::new(
            &device,
            &images,
            buffered_frame_count,
            surface_format.format,
            graphics_command_pool,
            &local_allocator,
        )?;
        Ok(Some(
            Self {
                resources,
                images,
                image_states,
                handle: swapchain_handle,
                frame_index: 0,
                image_index: 0,
                surface_format,
                image_extent,
            }
        ))
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device,
        grapchis_queue: vk::Queue,
        graphics_command_pool: Option<vk::CommandPool>,
    )
    {
        self.resources.destroy(device, grapchis_queue, graphics_command_pool);
        unsafe { swapchain_loader.destroy_swapchain(self.handle, None); }
    }

    pub fn subresource_range_info() -> ImageSubresourceRangeInfo {
        ImageSubresourceRangeInfo
            ::new(ImageAspect::Color as _, 0, 1, 0, 1)
            .unwrap()
    }

    pub const fn frame_timeout() -> u64 {
        1_000_000_000
    }

    pub fn setup_image(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device,
    ) -> Result<Option<FrameData>>
    {
        let (untied_resources, command_buffer) = self.resources.get_untied_resources(self.frame_index);
        let fences = slice::from_ref(&untied_resources.frame_ready_fence);
        unsafe { device
            .wait_for_fences(
                fences,
                true,
                Self::frame_timeout()
            )
            .context_with(|| format_compact!("failed to wait for fences at {}", location!()))?;
        };
        unsafe { device
            .reset_fences(fences)
            .context_with(|| format_compact!("failed to reset fences at {}", location!()))?
        };
        let next_image = unsafe { match swapchain_loader
            .acquire_next_image(
                self.handle,
                Self::frame_timeout(),
                untied_resources.image_ready_semaphore,
                vk::Fence::null()
            ) {
                Ok(image) => image,
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        return Ok(None)
                    }
                    return Err(Error::new("failed to acquire swapchain image", err))
                }
            }};
        self.image_index = next_image.0;
        let tied_resources = self.resources.get_tied_resources(self.image_index);
        let image_index = self.image_index as usize;
        Ok(Some(
            FrameData::new(
                self.images[image_index],
                tied_resources.image_view,
                *command_buffer,
                self.frame_index,
                self.image_states[image_index],
                self.surface_format.format,
                self.image_extent,
                next_image.1,
            )
        ))
    }

    pub fn setup_submit(
        &mut self,
        device: &ash::Device,
        src_image_state: ImageState,
        graphics_queue_index: u32,
    ) -> (SubmitSemaphores, vk::Fence) {
        let (untied_resources, command_buffer) = self.resources.get_untied_resources(self.frame_index);
        let tied_resources = self.resources.get_tied_resources(self.image_index);
        let image_index = self.image_index as usize;
        let dst_image_state = ImageState::new(
            vk::AccessFlags::NONE,
            vk::ImageLayout::PRESENT_SRC_KHR,
            graphics_queue_index,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        let memory_barrier = src_image_state.to_memory_barrier(
            self.images[image_index],
            dst_image_state,
            Self::subresource_range_info()
        );
        unsafe {
            device.cmd_pipeline_barrier(
                *command_buffer,
                src_image_state.pipeline_stage,
                dst_image_state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                &[memory_barrier],
            );
        }
        self.image_states[image_index] = dst_image_state;
        (
            SubmitSemaphores {
                wait_semaphore: untied_resources.image_ready_semaphore,
                wait_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                signal_semaphore: tied_resources.present_wait_semaphore,
            },
            untied_resources.frame_ready_fence,
        )
    }

    pub fn present_submit(
        &mut self,
        swapchain_loader: &swapchain::Device,
        queue: vk::Queue,
    ) -> Result<PresentResult> {
        let tied_resources = self.resources.get_tied_resources(self.image_index);
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: &tied_resources.present_wait_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.handle,
            p_image_indices: &self.image_index,
            ..Default::default()
        };
        unsafe {
            match swapchain_loader.queue_present(queue, &present_info) {
                Ok(suboptimal) => {
                    self.frame_index = (self.frame_index + 1) % self.resources.buffered_frame_count();
                    Ok(if suboptimal { PresentResult::Suboptimal } else { PresentResult::Success })
                }
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        Ok(PresentResult::OutOfDate)
                    }
                    else {
                        Err(Error::new("swapchain failed to present", err))
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
    alloc: &ArenaAlloc,
) -> Result<vk::SurfaceFormatKHR>
{
    unsafe {
        let alloc = ArenaGuard::new(alloc);
        let get_physical_device_surface_formats_khr = surface_loader.fp().get_physical_device_surface_formats_khr;
        let mut count = 0u32;
        let mut result = get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            ptr::null_mut(),
        );
        if count == 0 || result != vk::Result::SUCCESS {
            return Err(Error::new("failed to get vulkan surface formats", result))
        }
        let mut formats = FixedVec::with_len(count as usize, Default::default(), &alloc)?;
        result = get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            formats.as_mut_ptr(),
        );
        if result != vk::Result::SUCCESS {
            return Err(Error::new("failed to get vulkan surface formats", result))
        }
        for format in &formats {
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
    alloc: &ArenaAlloc,
) -> Result<vk::PresentModeKHR>
{
    unsafe {
        let alloc = ArenaGuard::new(alloc);
        let get_physical_device_surface_present_modes_khr = surface_loader.fp().get_physical_device_surface_present_modes_khr;
        let mut count = 0u32;
        let mut result = get_physical_device_surface_present_modes_khr(
            physical_device,
            surface_handle,
            &mut count,
            ptr::null_mut(),
        );
        if count == 0 || result != vk::Result::SUCCESS {
            return Err(Error::new("failed to get vulkan surface present modes", result))
        }
        let mut modes = FixedVec::with_len(count as usize, Default::default(), &alloc)?;
        result = get_physical_device_surface_present_modes_khr(
            physical_device,
            surface_handle,
            &mut count,
            modes.as_mut_ptr(),
        );
        if result != vk::Result::SUCCESS {
            return Err(Error::new("failed to get vulkan surface present modes", result))
        }
        for mode in &modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return Ok(vk::PresentModeKHR::MAILBOX); // low latency and no tearing
            }
        }
        return Ok(vk::PresentModeKHR::FIFO); // always supported
    }
}
