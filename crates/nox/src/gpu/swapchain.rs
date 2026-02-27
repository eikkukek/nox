use core::{
    ptr::{self, NonNull},
    num::NonZeroU64,
};

use compact_str::format_compact;

use nox_mem::{
    vec::{Vector, FixedVec32, NonNullVec32},
    conditional::True,
    alloc::LocalAlloc,
    result::ResultExt,
};

use nox_alloc::arena::*;

use nox_ash::{
    vk::{self, TaggedStructure, StructureTypeKHR},
    khr::swapchain,
};

use crate::dev::{
    error::{Context, Error, Result, location},
    has_bits, has_not_bits,
};

use super::Vulkan;

#[derive(Clone, Copy)]
pub struct SwapchainFrameData {
    pub image_index: u32,
    pub suboptimal: bool,
    pub acquire_image_semaphore: vk::Semaphore,
}

#[derive(Default, Clone, Copy)]
struct TiedResources {
    present_wait_semaphore: vk::Semaphore,
}

impl TiedResources {

    fn new(
        vk: &Vulkan,
        image: vk::Image,
        image_format: vk::Format,
    ) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let present_wait_semaphore = unsafe { vk.device()
            .create_semaphore(&semaphore_create_info, None)
            .context("failed to create semaphore")?
        };
        Ok(Self {
            present_wait_semaphore,
        })
    }

    fn destroy(self, vk: &Vulkan) {
        unsafe {
            vk.device().destroy_semaphore(self.present_wait_semaphore, None);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct UntiedResources {
    image_ready_semaphore: vk::Semaphore,
    fence: vk::Fence,
}

impl UntiedResources {

    pub fn new(vk: &Vulkan) -> Result<Self> { 
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };
        let image_ready_semaphore = unsafe { vk.device()
            .create_semaphore(&semaphore_create_info, None)
            .context("failed to create semaphore")?
        };
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        let fence = unsafe {
            vk.device().create_fence(&fence_create_info, None)
            .context("failed to create fence")?
        };
        Ok(Self {
            image_ready_semaphore,
            fence,
        })
    }

    fn destroy(self, vk: &Vulkan) {
        unsafe {
            vk.device().destroy_semaphore(self.image_ready_semaphore, None);
        }
    }
}

struct Resources {
    tied_resources: NonNull<TiedResources>,
    untied_resources: NonNull<UntiedResources>,
}

impl Resources {

    fn new<Alloc: LocalAlloc<Error = Error>>(
        vk: &Vulkan,
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
        vk: &Vulkan,
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
    pub wait_stage_mask: vk::PipelineStageFlags2,
    pub signal_semaphore: vk::Semaphore,
    pub signal_stage_mask: vk::PipelineStageFlags2,
}

pub struct Swapchain {
    resources: Resources,
    image_count: u32,
    images: NonNull<vk::Image>,
    handle: vk::SwapchainKHR,
    image_index: u32,
    surface_format: vk::SurfaceFormatKHR,
    image_extent: vk::Extent2D,
    image_usage: vk::ImageUsageFlags,
    present_id: Option<NonZeroU64>,
}

pub struct SwapchainImages<'a> {
    pub handles: &'a [vk::Image],
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub usage: vk::ImageUsageFlags,
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
        vk: &Vulkan,
        surface: vk::SurfaceKHR,
        framebuffer_extent: vk::Extent2D,
        buffered_frames: &mut u32,
        alloc: &RwArena,
        tmp_alloc: &(impl Arena<True> + ImmutArena),
    ) -> Result<Self>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Err(Error::just_context("frame buffer size was 0"))
        }
        let surface_format = find_surface_format(
            vk, surface, tmp_alloc,
        )?;
        let present_mode = find_present_mode(
            vk, surface, tmp_alloc,
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
            vk.get_surface_capabilities2_instance()
                .get_physical_device_surface_capabilities2(
                    vk.physical_device(), &mut surface_info, &mut capabilities,
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
        actual_image_count = actual_image_count.max(*buffered_frames);
        if capabilities.surface_capabilities.max_image_count > 0 &&
            actual_image_count > capabilities.surface_capabilities.max_image_count
        {
            actual_image_count = capabilities.surface_capabilities.max_image_count;
            if actual_image_count < *buffered_frames {
                *buffered_frames = actual_image_count;
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
            match vk.swapchain_device().create_swapchain(&create_info, None) {
                Ok(swapchain) => swapchain,
                Err(err) => return Err(Error::new(err, "failed to create swapchain")),
            }
        };
        let mut image_count = 0u32;
        unsafe {
            vk.swapchain_device().get_swapchain_images_khr(
                handle, &mut image_count, ptr::null_mut()
            ).context("failed to get Vulkan swapchain images")?;
        };
        let mut images = NonNullVec32
            ::with_capacity(
                image_count,
                alloc,
            )?;
        images.resize(image_count, Default::default());
        unsafe {
            vk.swapchain_device().get_swapchain_images_khr(
                handle,
                &mut image_count,
                images.as_mut_ptr(),
            ).context("failed to get Vulkan swapchain images")?;
        };
        let resources = Resources::new(
            vk,
            &images,
            *buffered_frames,
            surface_format.format,
            alloc,
        )?;
        Ok(Self {
            resources,
            image_count: images.len(),
            images: images.into_inner(),
            handle,
            image_index: 0,
            surface_format,
            image_extent,
            image_usage,
            present_id,
        })
    }

    pub fn destroy(
        &mut self,
        vk: &Vulkan,
        present_queue: vk::Queue,
    ) -> Result<()>
    {
        unsafe {
            if let Some(id) = self.present_id {
                let info = vk::PresentWait2InfoKHR {
                    s_type: vk::PresentWait2InfoKHR::STRUCTURE_TYPE,
                    present_id: id.get(),
                    timeout: vk.frame_timeout(),
                    ..Default::default()
                };
                let result = vk.present_wait2_device().wait_for_present2_khr(
                    self.handle,
                    &info
                ).filter_err(|&err| (err == vk::Result::ERROR_OUT_OF_DATE_KHR).then_some(err))
                .context("unexpected vulkan error at {}")?;
                if result == vk::Result::TIMEOUT {
                    return Err(Error::just_context(format_compact!(
                        "frame timeout {} nanoseconds reached at {}",
                        vk.frame_timeout(), location!(),
                    )))
                }
            } else {
                if vk.device().queue_wait_idle(present_queue).is_err() &&
                    vk.device().device_wait_idle().is_err()
                {
                    panic!("GPU hang")
                }
            }
        }
        self.resources.destroy(
            vk,
            present_queue,
            self.image_count,
            self.handle,
            self.present_id,
        );
        unsafe { vk.swapchain_device().destroy_swapchain(self.handle, None); }
        Ok(())
    }


    #[inline(always)]
    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    #[inline(always)]
    pub fn images(&self) -> SwapchainImages<'_> {
        SwapchainImages {
            handles: unsafe {
                core::slice::from_raw_parts(
                    self.images.as_ptr(),
                    self.image_count as usize,
                )
            },
            format: self.surface_format.format,
            extent: self.image_extent,
            usage: self.image_usage
        }
    }

    pub unsafe fn acquire_next_image(
        &mut self,
        vk: &Vulkan,
        frame_index: u32,
    ) -> Result<Option<SwapchainFrameData>>
    {
        let untied_resources = unsafe {
            self.resources.get_untied_resources(frame_index as usize)
        };
        unsafe {
            if vk.device().wait_for_fences(
                &[untied_resources.fence],
                true, vk.frame_timeout()
                ).context("unexpected fence wait error")? == vk::Result::TIMEOUT
            {
                return Err(Error::just_context(
                    format_compact!(
                        "frame timeout {} nanoseconds reached at {}", vk.frame_timeout(),
                        location!(),
                    )
                ))
            }
            vk.device().reset_fences(&[untied_resources.fence])
            .context("failed to reset fence")?;
        }
        let (image_index, suboptimal) = unsafe { match vk.swapchain_device()
            .acquire_next_image(
                self.handle,
                vk.frame_timeout(),
                untied_resources.image_ready_semaphore,
                untied_resources.fence,
            ) {
                Ok(image) => image,
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        return Ok(None)
                    }
                    return Err(Error::new(err, "failed to acquire swapchain image"))
                }
            }};
        if image_index >= self.image_count {
            return Err(Error::just_context(format_compact!(
                "aquired Vulkan image index {} was out of bounds with image count {}",
                image_index, self.image_count,
            )))
        }
        let tied_resources = unsafe { 
            self.resources.get_tied_resources(image_index)
        };
        self.image_index = image_index;
        unsafe {
            Ok(Some(
                SwapchainFrameData {
                    image_index,
                    suboptimal,
                    acquire_image_semaphore: untied_resources.image_ready_semaphore,
                }
            ))
        }
    }

    pub unsafe fn get_submit_semaphores(
        &mut self,
        vk: &Vulkan,
        frame_index: u32,
        wait_stage_mask: vk::PipelineStageFlags2,
    ) -> SwapchainSubmitSemaphores {
        let untied_resources = unsafe {
            self.resources.get_untied_resources(frame_index as usize)
        };
        let tied_resources = unsafe {
            self.resources.get_tied_resources(self.image_index)
        };
        SwapchainSubmitSemaphores {
            wait_semaphore: untied_resources.image_ready_semaphore,
            wait_stage_mask,
            signal_semaphore: tied_resources.present_wait_semaphore,
            signal_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        }
    }

    pub unsafe fn present_submit(
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
    vk: &Vulkan,
    surface_handle: vk::SurfaceKHR,
    tmp_alloc: &Alloc,
) -> Result<vk::SurfaceFormatKHR>
{
    unsafe {
        let physical_device = vk.physical_device();
        let tmp_alloc = tmp_alloc.guard();
        let mut count = 0u32;
        vk.surface_instance().get_physical_device_surface_formats_khr(
            physical_device,
            surface_handle,
            &mut count,
            ptr::null_mut(),
        ).context("failed to get Vulkan surface formats")?;
        let mut formats = FixedVec32
            ::with_len(count, Default::default(), &tmp_alloc)?;
        vk.surface_instance().get_physical_device_surface_formats_khr(
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
    vk: &Vulkan,
    surface: vk::SurfaceKHR,
    tmp_alloc: &Alloc,
) -> Result<vk::PresentModeKHR>
{
    unsafe {
        let tmp_alloc = tmp_alloc.guard();
        let physical_device = vk.physical_device();
        let mut count = 0u32;
        vk.surface_instance().get_physical_device_surface_present_modes_khr(
            physical_device,
            surface,
            &mut count,
            ptr::null_mut(),
        ).context("failed to get Vulkan surface present modes")?;
        let mut modes = FixedVec32
            ::with_len(count, Default::default(), &tmp_alloc)?;
        vk.surface_instance().get_physical_device_surface_present_modes_khr(
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
