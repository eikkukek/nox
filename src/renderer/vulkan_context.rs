use super::{
    helpers::Handle,
    physical_device::{self, find_suitable_physical_device, PhysicalDeviceInfo},
    swapchain_context::SwapchainContext,
    DeviceName,
};

use crate::{
    stack_allocator::{StackAllocator, StackGuard, StackRegion},
    string::{LargeError, SmallError, String},
    vec_types::{ArrayVec, FixedVec, VecOperations},
    version::Version, AppName
};

use winit::{dpi::PhysicalSize, window::Window};
use ash::{
    khr::{surface, swapchain, wayland_surface, win32_surface, xcb_surface, xlib_surface},
    vk,
    Entry
};
use ash_window;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle,};
use std::{ffi::CString, mem::ManuallyDrop};

pub struct VulkanMemory<'mem> {
    pub init_allocator: StackAllocator<'mem>,
    pub swapchain_allocator: StackAllocator<'mem>,
}

pub enum SwapchainState {
    Valid,
    OutOfDate(PhysicalSize<u32>),
}

pub struct VulkanContext<'mem> {
    #[allow(unused)]
    entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: surface::Instance,
    swapchain_loader: swapchain::Device,
    surface_handle: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,
    device: ash::Device,
    graphics_queue: vk::Queue,
    _transfer_queue: vk::Queue,
    _compute_queue: vk::Queue,
    pub swapchain_state: SwapchainState,
    swapchain_context: ManuallyDrop<Option<SwapchainContext<'mem>>>,
}

impl<'mem> VulkanContext<'mem> {

    pub fn new(
        window: &Window,
        app_name: &AppName,
        app_version: Version,
        enable_validation: bool,
        vulkan_memory: &mut VulkanMemory,
    ) -> Result<VulkanContext<'mem>, LargeError> {
        let mut init_allocator = StackGuard::new(&mut vulkan_memory.init_allocator);
        let entry = unsafe { Entry::load().unwrap() };
        match unsafe { entry.try_enumerate_instance_version() } {
            Ok(v) => {
                if v.is_none() {
                    println!("Nox backend warning: failed to enumerate vulkan loader version");
                }
                else {
                    let version = Version::from(v.unwrap());
                    println!("Vulkan loader version: {}.{}.{}",
                        version.major(),
                        version.minor(),
                        version.patch(),
                    );
                }
            },
            Err(result) => {
                println!("Nox backend warning: failed to enumerate vulkan loader version {:?}", result)
            },
        };
        let app_name = CString::new(app_name.as_str()).unwrap();
        let engine_name = CString::new("nox").unwrap();
        let application_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            api_version: vk::API_VERSION_1_3,
            p_application_name: app_name.as_ptr(),
            application_version: app_version.as_u32(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let Some(mut instance_extensions) = FixedVec::<*const i8>::new(8, &mut init_allocator) else {
            return Err(String::from_str("init stack was out of memory"))
        };
        let Some(mut layers) = FixedVec::<*const i8>::new(8, &mut init_allocator) else {
            return Err(String::from_str("init stack was out of memory"))
        };
        get_required_instance_extensions(window, &mut instance_extensions)
            .map_err(|e| String::format(format_args!(
                "failed to get required vulkan instance extensions ( {} )", e
            )))?;
        let validation_layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        let ext_debug_utils = CString::new("VK_EXT_debug_utils").unwrap();
        if enable_validation {
            instance_extensions.push_back(ext_debug_utils.as_ptr());
            layers.push_back(validation_layer_name.as_ptr());
        }
        verify_instance_layers(&entry, &layers)
            .map_err(|e| String::format(format_args!(
                "failed to verify instance layers ( {} )", e
            )))?;
        verify_instance_extensions(&entry, &instance_extensions)
            .map_err(|e| String::format(format_args!(
                "failed to verify instance extensions ( {} )", e
            )))?;
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &application_info,
            enabled_extension_count: instance_extensions.len() as u32,
            pp_enabled_extension_names: instance_extensions.as_ptr() as _,
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr() as _,
            ..Default::default()
        };
        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .map_err(|e| String::format(format_args!(
                    "failed to create vulkan instance {:?}", e
                )))?
        };
        let surface_loader = surface::Instance::new(&entry, &instance);
        let surface_handle = unsafe {
            ash_window
            ::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None
            )
            .map_err(|e| {
                instance.destroy_instance(None);
                String::format(format_args!("failed to create vulkan surface {:?}", e))
            })?
        };
        let (physical_device, physical_device_info) =
            find_suitable_physical_device(&instance, &surface_loader, surface_handle)
            .map_err(|e| {
                unsafe {
                    surface_loader.destroy_surface(surface_handle, None);
                    instance.destroy_instance(None);
                }
                String::format(format_args!("failed to find suitable physical device ( {} )", e))
            })?;
        let mut unique_device_queues = ArrayVec::<u32, 3>::new();
        let queue_family_indices = physical_device_info.get_queue_family_indices();
        unique_device_queues.push_back(queue_family_indices.get_graphics_index());
        unique_device_queues.push_back(queue_family_indices.get_transfer_index());
        unique_device_queues.push_back(queue_family_indices.get_compute_index());
        let mut device_queue_create_infos = ArrayVec::<vk::DeviceQueueCreateInfo, 3>::new();
        let queue_priority = 1.0;
        for queue_family_index in &unique_device_queues {
            device_queue_create_infos.push_back(
                vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    queue_count: 1,
                    queue_family_index: *queue_family_index,
                    p_queue_priorities: &queue_priority,
                    ..Default::default()
                }
            );
        }
        const ENABLED_DEVICE_EXTENSION_NAMES: [*const i8; 3] = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::timeline_semaphore::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            sample_rate_shading: vk::TRUE,
            sampler_anisotropy: vk::TRUE,
            fill_mode_non_solid: physical_device_info.features.sample_rate_shading,
            ..Default::default()
        };
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            queue_create_info_count: device_queue_create_infos.len() as u32,
            p_queue_create_infos: device_queue_create_infos.as_ptr() as *const _,
            enabled_extension_count: 3,
            pp_enabled_extension_names: ENABLED_DEVICE_EXTENSION_NAMES.as_ptr(),
            p_enabled_features: &features,
            ..Default::default()
        };
        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| {
                    surface_loader.destroy_surface(surface_handle, None);
                    instance.destroy_instance(None);
                    String::format(format_args!("failed to create vulkan device {:?}", e))
                })?
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.get_graphics_index(), 0) };
        let transfer_queue = unsafe { device.get_device_queue(queue_family_indices.get_transfer_index(), 0) };
        let compute_queue = unsafe { device.get_device_queue(queue_family_indices.get_compute_index(), 0) };
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        Ok(
            Self {
                entry,
                instance,
                surface_loader,
                swapchain_loader,
                surface_handle,
                physical_device,
                physical_device_info,
                device,
                graphics_queue,
                _transfer_queue: transfer_queue,
                _compute_queue: compute_queue,
                swapchain_state: SwapchainState::OutOfDate(window.inner_size()),
                swapchain_context: ManuallyDrop::new(None),
            },
        )
    }

    pub fn device(&self) -> Handle<'mem, ash::Device> {
        Handle::new(self.device.clone())
    }

    pub fn swapchain_loader(&self) -> Handle<'mem, swapchain::Device> {
        Handle::new(self.swapchain_loader.clone())
    }

    pub fn queue_family_indices(&self) -> &physical_device::QueueFamilyIndices {
        &self.physical_device_info.queue_family_indices
    }

    pub fn graphics_queue(&self) -> Handle<'mem, vk::Queue> {
        Handle::new(self.graphics_queue)
    }

    pub fn transfer_queue(&self) -> Handle<'mem, vk::Queue> {
        Handle::new(self.graphics_queue)
    }

    pub fn compute_queue(&self) -> Handle<'mem, vk::Queue> {
        Handle::new(self.graphics_queue)
    }

    pub fn physical_device_name(&self) -> DeviceName {
        self.physical_device_info.device_name
    }

    pub fn request_resize(&mut self, size: PhysicalSize<u32>) {
        self.swapchain_state = SwapchainState::OutOfDate(size);
    }

    pub fn update_swapchain(
        &mut self,
        framebuffer_size: PhysicalSize<u32>,
        graphics_command_pool: vk::CommandPool,
        vulkan_memory: &mut VulkanMemory<'mem>,
    ) -> Result<(), LargeError> {
        if let Some(mut context) = self.swapchain_context.take() {
            context.destroy(&self.device, &self.swapchain_loader, self.graphics_queue, Some(graphics_command_pool));
        }
        let allocator = &mut vulkan_memory.swapchain_allocator;
        allocator.clear();
        let Some(region) = StackRegion::new(allocator.size(), allocator) else {
            return Err(String::from_str("failed to create local swapchain allocator"))
        };
        self.swapchain_context = match SwapchainContext::new(
            &self.device,
            &self.surface_loader,
            &self.swapchain_loader,
            self.physical_device,
            self.surface_handle,
            vk::Extent2D { width: framebuffer_size.width, height: framebuffer_size.height },
            graphics_command_pool,
            self.queue_family_indices().get_graphics_index(),
            region,
            &mut vulkan_memory.init_allocator,
        ) {
            Ok(context) => ManuallyDrop::new(context),
            Err(err) => return Err(err),
        };
        self.swapchain_state = SwapchainState::Valid;
        Ok(())
    }

    pub fn get_swapchain_context(
        &mut self,
        graphics_command_pool: vk::CommandPool,
        vulkan_memory: &mut VulkanMemory<'mem>,
    ) -> Result<&mut SwapchainContext<'mem>, LargeError> {
        match self.swapchain_state {
            SwapchainState::Valid => {},
            SwapchainState::OutOfDate(framebuffer_size) => {
                self 
                    .update_swapchain(framebuffer_size, graphics_command_pool, vulkan_memory)
                    .map_err(|e| {
                        e
                    })?;
            },
        }
        Ok(self.swapchain_context.as_mut().expect("swapchain context was None"))
    }

    pub fn swapchain_state(&mut self, state: SwapchainState) {
        self.swapchain_state = state
    }

    pub fn destroy_swapchain(
        &mut self,
        graphics_command_pool: vk::CommandPool
    ) {
        if let Some(mut context) = self.swapchain_context.take() {
            context.destroy(&self.device, &self.swapchain_loader, self.graphics_queue, Some(graphics_command_pool));
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface_handle, None);
            self.surface_handle = vk::SurfaceKHR::null();
            self.instance.destroy_instance(None);
        }
    }
}

fn get_required_instance_extensions<W>(
    window: &W,
    out: &mut FixedVec::<*const i8>
) -> Result<(), SmallError> 
    where
        W: HasWindowHandle + HasDisplayHandle
{
    out.push_back(surface::NAME.as_ptr());
    let ext = match window.display_handle().unwrap().as_raw() {
        raw_window_handle::RawDisplayHandle::Wayland(_) => wayland_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Windows(_) => win32_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xlib(_) => xlib_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xcb(_) => xcb_surface::NAME.as_ptr(),
        _ => {
            return Err(SmallError::from_str("unsupported platform"));
        },
    };
    out.push_back(ext);
    Ok(())
}

fn verify_instance_layers(
    entry: &Entry,
    layers: &FixedVec::<*const i8>
) -> Result<(), SmallError>
{
    let available = unsafe {
        entry
            .enumerate_instance_layer_properties()
            .map_err(|e| String::format(format_args!(
                "failed to enumerate layer properties {:?}", e,
            )))?
    };
    for layer in layers {
        let string = unsafe {
            String::<{vk::MAX_EXTENSION_NAME_SIZE}>
                ::from_ascii_ptr(*layer)
                .map_err(|e| String::format(format_args!(
                    "failed to convert required layer to string ( {} )", e,
                )))?
        };
        if available
            .iter()
            .find(|a| {
                match String::<{vk::MAX_EXTENSION_NAME_SIZE}>::from_ascii(&a.layer_name) {
                    Ok(s) => string == s,
                    Err(_) => false,
                }
            }).is_none()
        {
            return Err(String::format(format_args!("failed to find layer {}", string)))
        }
    }
    Ok(())
}

fn verify_instance_extensions(
    entry: &Entry,
    extensions: &FixedVec::<*const i8>
) -> Result<(), SmallError>
{
    let available = unsafe {
        entry
            .enumerate_instance_extension_properties(None)
            .map_err(|e| String::format(format_args!(
                "failed to enumerate extension properties {:?}", e,
            )))?
    };
    for extension in extensions {
        let string = unsafe {
            String::<{vk::MAX_EXTENSION_NAME_SIZE}>
                ::from_ascii_ptr(*extension)
                .map_err(|e| String::format(format_args!(
                    "failed to convert required extension to string ( {} )", e,
                )))?
        };
        if available
            .iter()
            .find(|a| {
                match String::<{vk::MAX_EXTENSION_NAME_SIZE}>::from_ascii(&a.extension_name) {
                    Ok(s) => string == s,
                    Err(_) => false,
                }
            }).is_none()
        {
            return Err(String::format(format_args!("failed to find extension {}", string)))
        }
    }
    Ok(())
}
