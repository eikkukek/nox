use core::{
    ptr::{self, NonNull},
    num::NonZeroU64,
};

use compact_str::format_compact;

use nox_mem::{
    vec::{Vector, FixedVec32, NonNullVec32},
    conditional::True,
    alloc::LocalAlloc,
};

use nox_alloc::arena::*;

use nox_ash::{
    vk::{self, TaggedStructure, StructureTypeExt},
    khr::swapchain,
};

use crate::dev::{
    error::{Context, Error, Result, location},
    has_bits, has_not_bits,
};

use super::{
    Gpu, ImageSubresourceState,
    ImageSubresourceRange, ImageAspect,
    COMMAND_REQUEST_IGNORED,
};

#[derive(Clone, Copy)]
pub struct FrameData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_state: ImageSubresourceState,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub suboptimal: bool,
}

impl FrameData {

    pub fn new(
        image: vk::Image,
        image_view: vk::ImageView,
        image_state: ImageSubresourceState,
        format: vk::Format,
        extent: vk::Extent2D,
        suboptimal: bool,
    ) -> Self {
        Self {
            image,
            image_view,
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
        gpu: &Gpu,
        image: vk::Image,
        image_format: vk::Format,
    ) -> Result<Self> {
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
        let image_view = unsafe { gpu.vk().device()
            .create_image_view(&image_view_create_info, None)
            .context("failed to create image view")?
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let present_wait_semaphore = unsafe { gpu.vk().device()
            .create_semaphore(&semaphore_create_info, None)
            .context("failed to create semaphore")?
        };
        Ok(Self {
            image_view,
            present_wait_semaphore,
        })
    }

    fn destroy(self, resources: &Gpu) {
        unsafe {
            resources.vk().device().destroy_image_view(self.image_view, None);
            resources.vk().device().destroy_semaphore(self.present_wait_semaphore, None);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct UntiedResources {
    image_ready_semaphore: vk::Semaphore,
}

impl UntiedResources {

    pub fn new(gpu: &Gpu) -> Result<Self> { 
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let image_ready_semaphore = unsafe { gpu.vk().device()
            .create_semaphore(&semaphore_create_info, None)
            .context("failed to create semaphore")?
        };
        Ok(Self {
            image_ready_semaphore,
        })
    }

    fn destroy(self, gpu: &Gpu) {
        unsafe {
            gpu.vk().device().destroy_semaphore(self.image_ready_semaphore, None);
        }
    }
}

struct Resources {
    tied_resources: NonNull<TiedResources>,
    untied_resources: NonNull<UntiedResources>,
}

impl Resources {

    fn new<Alloc: LocalAlloc<Error = Error>>(
        gpu: &Gpu,
        images: &[vk::Image],
        buffered_frames: u32,
        image_format: vk::Format,
        alloc: &Alloc,
    ) -> Result<Self>
    {
        let image_count = images.len();
        let mut tied_resources = NonNullVec32::<TiedResources>
            ::with_capacity(image_count as u32, alloc)?;
        for i in 0..image_count {
            tied_resources
                .push(match TiedResources::new(vk, images[i], image_format) {
                    Ok(resources) => resources,
                    Err(err) => {
                        for j in 0..i {
                            tied_resources[j].destroy(vk);
                        }
                        return Err(Error::new(err, "failed to create tied resources"))
                    },
                });
        }
        let mut untied_resources = NonNullVec32::<UntiedResources>
            ::with_capacity(buffered_frames, alloc)?;
        for i in 0..buffered_frames as usize {
            untied_resources
                .push(
                    match UntiedResources::new(vk) {
                        Ok(resources) => resources,
                        Err(err) => {
                            for j in 0..image_count {
                                tied_resources[j].destroy(vk);
                            }
                            for k in 0..i {
                                untied_resources[k].destroy(vk);
                            }
                            return Err(Error::new(err, "failed to create untied resources"))
                        },
                });
        }
        Ok(Self {
            tied_resources: tied_resources.into_inner(),
            untied_resources: untied_resources.into_inner(),
        })
    }

    fn destroy(
        &mut self,
        gpu: &Gpu,
        present_queue: vk::Queue,
        image_count: u32,
        handle: vk::SwapchainKHR,
        present_id2: Option<NonZeroU64>,
    ) {
        let tied_resources = unsafe {
            NonNullVec32::new(self.tied_resources, image_count)
            .with_len(image_count)
        };
        for resource in &tied_resources {
            resource.destroy(vk);
        }
        let untied_resources = unsafe {
            NonNullVec32::new(self.untied_resources, image_count)
            .with_len(image_count)
        };
        for resource in &untied_resources {
            resource.destroy(vk);
        }
    }

    #[inline(always)]
    unsafe fn get_tied_resources(&self, image_index: u32) -> TiedResources {
        unsafe {
            self.tied_resources.add(image_index as usize).read()
        }
    }

    #[inline(always)]
    unsafe fn get_untied_resources(&self, frame_index: usize) -> UntiedResources {
        unsafe {
            self.untied_resources.add(frame_index).read()
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum PresentResult {
    Success,
    Suboptimal,
    OutOfDate,
}

pub struct SwapchainSubmitSemaphores {
    pub wait_semaphore: vk::Semaphore,
    pub wait_stage: vk::PipelineStageFlags,
    pub signal_semaphore: vk::Semaphore,
}

pub struct Swapchain {
    resources: Resources,
    image_count: u32,
    images: NonNull<vk::Image>,
    image_states: NonNull<ImageSubresourceState>,
    handle: vk::SwapchainKHR,
    image_index: u32,
    surface_format: vk::SurfaceFormatKHR,
    image_extent: vk::Extent2D,
    present_id: Option<NonZeroU64>,
}

unsafe impl Send for Swapchain {}
unsafe impl Sync for Swapchain {}

impl Swapchain {

    const SUPPORTS_PRESENT_WAIT2: u32 = 0x1;

    #[inline(always)]
    pub fn supports_present_wait2(&self) -> bool {
        self.present_id.is_some()
    }

    pub fn new(
        gpu: &Gpu,
        surface: vk::SurfaceKHR,
        framebuffer_extent: vk::Extent2D,
        mut buffered_frames: u32,
        alloc: &RwArena,
        tmp_alloc: &(impl Arena<True> + ImmutArena),
    ) -> Result<Self>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Err(Error::just_context("frame buffer size was 0"))
        }
        let surface_format = find_surface_format(
            gpu, surface, tmp_alloc,
        )?;
        let present_mode = find_present_mode(
            gpu, surface, tmp_alloc,
        )?;
        let mut id2 = vk::SurfaceCapabilitiesPresentId2KHR::default();
        let mut wait2 = vk::SurfaceCapabilitiesPresentWait2KHR::default();
        let mut surface_info = vk::PhysicalDeviceSurfaceInfo2KHR {
            s_type: vk::StructureType::PHYSICAL_DEVICE_SURFACE_INFO_2_KHR,
            surface,
            ..Default::default()
        };
        let mut capabilities = vk::SurfaceCapabilities2KHR
            ::default()
            .push_next(&mut id2)
            .push_next(&mut wait2);
        unsafe {
            gpu.vk().get_surface_capabilities2_instance()
                .get_physical_device_surface_capabilities2(
                    gpu.vk().physical_device(), &mut surface_info, &mut capabilities,
                ).context("failed to get physical device surface capabilities")?;
        };
        let present_id = (id2.present_id2_supported != 0 && wait2.present_wait2_supported != 0)
        .then_some(unsafe { NonZeroU64::new_unchecked(1) });
        let mut image_extent = capabilities.surface_capabilities.current_extent;
        if image_extent.width == u32::MAX {
            image_extent.width = framebuffer_extent.width.clamp(
                capabilities.surface_capabilities.min_image_extent.width,
                capabilities.surface_capabilities.max_image_extent.width,
            );
            image_extent.height = framebuffer_extent.height.clamp(
                capabilities.surface_capabilities.min_image_extent.height,
                capabilities.surface_capabilities.max_image_extent.height,
            );
        }
        if image_extent.width == 0 || image_extent.height == 0 {
            return Err(Error::just_context("swapchain extent was 0"));
        }
        let mut actual_image_count = capabilities.surface_capabilities.min_image_count + 1;
        actual_image_count = actual_image_count.max(buffered_frames);
        if capabilities.surface_capabilities.max_image_count > 0 &&
            actual_image_count > capabilities.surface_capabilities.max_image_count
        {
            actual_image_count = capabilities.surface_capabilities.max_image_count;
            if actual_image_count < buffered_frames {
                buffered_frames = actual_image_count;
            }
        }
        let mut pre_transform = capabilities.surface_capabilities.current_transform;
        if has_bits!(
            capabilities.surface_capabilities.supported_transforms,
            vk::SurfaceTransformFlagsKHR::IDENTITY
        ) {
            pre_transform = vk::SurfaceTransformFlagsKHR::IDENTITY;
        }
        let mut composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;
        if has_bits!(capabilities.surface_capabilities
                .supported_composite_alpha,
                vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
            )
        {
            composite_alpha = vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED;
        }
        let image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        if has_not_bits!(capabilities.surface_capabilities.supported_usage_flags, image_usage) {
            return Err(Error::just_context(format_compact!(
                "unsupported swapchain, missing image usage {image_usage:?}"
            )))
        }
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface,
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
        let handle = unsafe {
            match gpu.vk().swapchain_device().create_swapchain(&create_info, None) {
                Ok(swapchain) => swapchain,
                Err(err) => return Err(Error::new(err, "failed to create swapchain")),
            }
        };
        let mut image_count = 0u32;
        unsafe {
            gpu.vk().swapchain_device().get_swapchain_images_khr(
                handle, &mut image_count, ptr::null_mut()
            ).context("failed to get Vulkan swapchain images")?;
        };
        let mut images = NonNullVec32
            ::with_capacity(
                image_count,
                alloc,
            )?;
        images.resize(image_count, Default::default());
        let mut image_states = NonNullVec32
            ::with_capacity(
                image_count,
                alloc,
            )?;
        image_states.resize(image_count,
            ImageSubresourceState::new(
                vk::PipelineStageFlags2::NONE,
                vk::AccessFlags2::NONE,
                vk::ImageLayout::UNDEFINED,
                vk::QUEUE_FAMILY_IGNORED,
                COMMAND_REQUEST_IGNORED,
                0,
            )
        );
        unsafe {
            gpu.vk().swapchain_device().get_swapchain_images_khr(
                handle,
                &mut image_count,
                images.as_mut_ptr(),
            ).context("failed to get Vulkan swapchain images")?;
        };
        let resources = Resources::new(
            gpu,
            &images,
            buffered_frames,
            surface_format.format,
            alloc,
        )?;
        Ok(Self {
            resources,
            image_count: images.len(),
            images: images.into_inner(),
            image_states: image_states.into_inner(),
            handle,
            image_index: 0,
            surface_format,
            image_extent,
            present_id,
        })
    }

    pub fn destroy(
        &mut self,
        gpu: &Gpu,
        present_queue: vk::Queue,
    ) -> Result<()>
    {
        unsafe {
            if let Some(id) = self.present_id {
                let info = vk::PresentWait2InfoKHR {
                    s_type: vk::PresentWait2InfoKHR::STRUCTURE_TYPE,
                    present_id: id.get(),
                    timeout: gpu.vk().frame_timeout(),
                    ..Default::default()
                };
                if let Err(err) = gpu.vk().present_wait2_device().wait_for_present2_khr(
                    self.handle,
                    &info
                ) {
                    if err == vk::Result::TIMEOUT {
                        return Err(Error::just_context(format_compact!(
                            "frame timeout {} nanoseconds reached at {}",
                            gpu.vk().frame_timeout(), location!(),
                        )))
                    } else {
                        return Err(Error::just_context(format_compact!(
                            "unexpected vulkan error at {}",
                            location!(),
                        )))
                    }
                }
            } else {
                if gpu.vk().device().queue_wait_idle(present_queue).is_err() &&
                    gpu.vk().device().device_wait_idle().is_err()
                {
                    panic!("GPU hang")
                }
            }
        }
        self.resources.destroy(
            gpu,
            present_queue,
            self.image_count,
            self.handle,
            self.present_id,
        );
        unsafe { gpu.vk().swapchain_device().destroy_swapchain(self.handle, None); }
        Ok(())
    }

    pub fn subresource_range_info() -> ImageSubresourceRange {
        ImageSubresourceRange
            ::new(ImageAspect::Color as _, 0, 1, 0, 1)
            .unwrap()
    }

    pub unsafe fn setup_image(
        &mut self,
        gpu: &Gpu,
        frame_index: usize,
    ) -> Result<Option<FrameData>>
    {
        assert!(frame_index < self.image_count as usize);
        let untied_resources = unsafe {
            self.resources.get_untied_resources(frame_index)
        };
        let (next_image, suboptimal) = unsafe { match gpu.vk().swapchain_device()
            .acquire_next_image(
                self.handle,
                gpu.vk().frame_timeout(),
                untied_resources.image_ready_semaphore,
                vk::Fence::null()
            ) {
                Ok(image) => image,
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        return Ok(None)
                    }
                    return Err(Error::new(err, "failed to acquire swapchain image"))
                }
            }};
        if next_image >= self.image_count {
            return Err(Error::just_context(format_compact!(
                "aquired Vulkan image index {} was out of bounds with image count {}",
                next_image, self.image_count,
            )))
        }
        let tied_resources = unsafe { 
            self.resources.get_tied_resources(next_image)
        };
        self.image_index = next_image;
        let image_index = next_image as usize;
        unsafe {
            Ok(Some(
                FrameData::new(
                    self.images.add(image_index).read(),
                    tied_resources.image_view,
                    self.image_states.add(image_index).read(),
                    self.surface_format.format,
                    self.image_extent,
                    suboptimal,
                )
            ))
        }
    }

    pub unsafe fn setup_submit(
        &mut self,
        device: &Gpu,
        command_buffer: vk::CommandBuffer,
        src_image_state: ImageState,
        graphics_queue_index: u32,
        frame_index: usize,
    ) -> SwapchainSubmitSemaphores {
        let untied_resources = unsafe {
            self.resources.get_untied_resources(frame_index)
        };
        let tied_resources = unsafe {
            self.resources.get_tied_resources(self.image_index)
        };
        let image_index = self.image_index as usize;
        let dst_image_state = ImageState::new(
            vk::AccessFlags::NONE,
            vk::ImageLayout::PRESENT_SRC_KHR,
            graphics_queue_index,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );
        let memory_barrier = src_image_state.to_memory_barrier(
            unsafe {
                self.images.add(image_index).read()
            },
            dst_image_state,
            Self::subresource_range_info()
        );
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                src_image_state.pipeline_stage,
                dst_image_state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                &[memory_barrier],
            );
        }
        unsafe {
            self.image_states
                .add(image_index)
                .write(dst_image_state);
        }
        SwapchainSubmitSemaphores {
            wait_semaphore: untied_resources.image_ready_semaphore,
            wait_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            signal_semaphore: tied_resources.present_wait_semaphore,
        }
    }

    pub fn present_submit(
        &mut self,
        swapchain_device: &swapchain::Device,
        present_queue: vk::Queue,
    ) -> Result<PresentResult> {
        let tied_resources = unsafe {
            self.resources.get_tied_resources(self.image_index)
        };
        let mut present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: &tied_resources.present_wait_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.handle,
            p_image_indices: &self.image_index,
            ..Default::default()
        };
        let mut present_id = 0;
        let mut present_id2 = vk::PresentId2KHR {
            s_type: vk::StructureType::PRESENT_ID_2_KHR,
            swapchain_count: 1,
            p_present_ids: &present_id,
            ..Default::default()
        };
        if let Some(id) = &mut self.present_id {
            present_id = id.get() + 1;
            *id = unsafe {
                NonZeroU64::new_unchecked(present_id)
            };
            present_info = present_info.push_next(&mut present_id2);
        }
        unsafe {
            match swapchain_device.queue_present(present_queue, &present_info) {
                Ok(suboptimal) => {
                    Ok(if suboptimal { PresentResult::Suboptimal } else { PresentResult::Success })
                }
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        Ok(PresentResult::OutOfDate)
                    }
                    else {
                        Err(Error::new(err, "swapchain failed to present"))
                    }
                }
            }
        }
    }
}

fn find_surface_format<Alloc: Arena<True> + ImmutArena>(
    gpu: &Gpu,
    surface_handle: vk::SurfaceKHR,
    tmp_alloc: &Alloc,
) -> Result<vk::SurfaceFormatKHR>
{
    unsafe {
        let physical_device = gpu.vk().physical_device();
        let tmp_alloc = tmp_alloc.guard();
        let mut count = 0u32;
        gpu.vk().surface_instance().get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            ptr::null_mut(),
        ).context("failed to get Vulkan surface formats")?;
        let mut formats = FixedVec32
            ::with_len(count, Default::default(), &tmp_alloc)?;
        gpu.vk().surface_instance().get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            formats.as_mut_ptr(),
        ).context("failed to get Vulkan surface formats")?;
        for &format in &formats {
            if format.format == vk::Format::R8G8B8A8_SRGB &&
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return Ok(format);
            }
        }
        Ok(formats[0])
    }
}

fn find_present_mode<Alloc: Arena<True> + ImmutArena>(
    gpu: &Gpu,
    surface: vk::SurfaceKHR,
    tmp_alloc: &Alloc,
) -> Result<vk::PresentModeKHR>
{
    unsafe {
        let tmp_alloc = tmp_alloc.guard();
        let physical_device = gpu.vk().physical_device();
        let mut count = 0u32;
        gpu.vk().surface_instance().get_physical_device_surface_present_modes_khr(
            physical_device,
            surface,
            &mut count,
            ptr::null_mut(),
        ).context("failed to get Vulkan surface present modes")?;
        let mut modes = FixedVec32
            ::with_len(count, Default::default(), &tmp_alloc)?;
        gpu.vk().surface_instance().get_physical_device_surface_present_modes_khr(
            physical_device,
            surface,
            &mut count,
            modes.as_mut_ptr(),
        ).context("failed to get Vulkan surface present modes")?;
        for mode in &modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return Ok(vk::PresentModeKHR::MAILBOX); // low latency and no tearing
            }
        }
        Ok(vk::PresentModeKHR::FIFO) // always supported
    }
}
