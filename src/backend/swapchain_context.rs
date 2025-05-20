use super::SmallError;

use crate::{
    allocator_traits::AllocateExt,
    stack_allocator::{StackAllocator, StackGuard},
    string::String, utility::{clamp, has_bit, has_not_bit},
    vec_types::FixedVec,
};

use std::{
    mem::ManuallyDrop,
    ptr::NonNull,
};

use ash::{khr::{surface, swapchain}, vk};

pub enum SwapchainState {
    Valid,
    Invalid,
    Suboptimal,
}

pub struct SwapchainResources<'mem> {
    guard: StackGuard<'mem, 'mem>,
    images: FixedVec<'mem, vk::Image>,
    image_views: FixedVec<'mem, vk::ImageView>,
}

impl<'mem> SwapchainResources<'mem> {

    pub unsafe fn new(
        image_count: usize,
        stack: &'mem mut StackAllocator<'mem>
    ) -> Option<NonNull<ManuallyDrop<Self>>>
    {
        let mut guard = StackGuard::new(stack);
        let images = FixedVec::new_with_len(image_count, image_count, &mut guard)?;
        let image_views = FixedVec::new_with_len(image_count, image_count, &mut guard)?;
        let ptr = unsafe { guard.allocate_uninit(1)? };
        unsafe {
            ptr.write(
                ManuallyDrop::new(
                    SwapchainResources {
                        guard,
                        images,
                        image_views,
                    }
                )
            );
        }
        Some(ptr)
    }

    pub fn destroy_resources(&mut self, device: &ash::Device) {
        for image_view in self.image_views.iter() {
            unsafe { device.destroy_image_view(*image_view, None); }
        }
        self.images.clear();
        self.image_views.clear();
    }
}

type SwapchainResourceHandle<'mem> = NonNull<ManuallyDrop<SwapchainResources<'mem>>>;

pub struct SwapchainContext<'mem> {
    pub handle: vk::SwapchainKHR,
    pub resources: Option<SwapchainResourceHandle<'mem>>,
    pub swapchain_state: SwapchainState,
    pub image_count: u32,
    pub image_usage: vk::ImageUsageFlags,
}

impl<'mem> SwapchainContext<'mem> {

    pub fn new(
        device: &ash::Device,
        surface_loader: &surface::Instance,
        swapchain_loader: &swapchain::Device,
        physical_device: vk::PhysicalDevice,
        surface_handle: vk::SurfaceKHR,
        allocator: &'mem mut StackAllocator<'mem>,
        framebuffer_extent: vk::Extent2D
    ) -> Result<Option<Self>, SmallError>
    {
        if framebuffer_extent.width == 0 || framebuffer_extent.height == 0 {
            return Ok(None)
        }
        let surface_format = match find_surface_format(surface_loader, physical_device, surface_handle, allocator) {
            Ok(format) => format,
            Err(err) => return Err(err),
        };
        let present_mode = match find_present_mode(surface_loader, physical_device, surface_handle, allocator) {
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
        let mut image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
        if has_not_bit!(capabilities.supported_usage_flags, image_usage) {
            return Err(String::from_str("swapchain does not support color attachment usage"))
        }
        image_usage |= capabilities.supported_usage_flags & vk::ImageUsageFlags::TRANSFER_DST;
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
        let mut resources_ptr = unsafe {
            match SwapchainResources::new(image_count as usize, allocator) {
                Some(resources) => resources,
                None => {
                    return Err(String::from_str("failed to allocate swapchain resources"))
                },
            }
        };
        unsafe{
            let resources = resources_ptr.as_mut();
            result = get_swapchain_images_khr(device.handle(), swapchain_handle, &mut image_count, resources.images.as_mut_ptr());
            if result != vk::Result::SUCCESS {
                resources_ptr.drop_in_place();
                swapchain_loader.destroy_swapchain(swapchain_handle, None);
                return Err(String::format(format_args!("failed to get swapchain ")))
            }
            for i in 0..image_count as usize {
                let create_info = vk::ImageViewCreateInfo {
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
                    .create_image_view(&create_info, None)
                    .map_err(|e| {
                        for j in 0..i {
                            device.destroy_image_view(resources.image_views[j], None);
                        }
                        resources_ptr.drop_in_place();
                        swapchain_loader.destroy_swapchain(swapchain_handle, None);
                        String::format(format_args!("failed to create image view {}", e))
                })?
            }
        }
        Ok(
            Some(
                Self {
                    handle: swapchain_handle,
                    resources: Some(resources_ptr),
                    swapchain_state: SwapchainState::Valid,
                    image_count,
                    image_usage,
                }
            )
        )
    }

    pub fn destroy(
        &mut self,
        device: &ash::Device,
        swapchain_loader: &swapchain::Device,
    ) {
        let Some(mut resource_handle) = self.resources.take() else { return };
        unsafe {
            let resources = resource_handle.as_mut();
            resources.destroy_resources(device);
            resource_handle.drop_in_place();
            swapchain_loader.destroy_swapchain(self.handle, None);
        }
    }

    pub fn is_transfer_dst_supported(&self) -> bool {
        return has_bit!(self.image_usage, vk::ImageUsageFlags::TRANSFER_DST);
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
