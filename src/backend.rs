mod physical_device;
mod swapchain_context;

use swapchain_context::SwapchainContext;
use physical_device::{PhysicalDeviceInfo, find_suitable_physical_device};

use super::{
    nox::AppName,
    string::String,
    version::Version,
    constants::SMALL_ERROR_LENGTH,
    stack_allocator::{StackAllocator, StackMemory},
};

use ash::{
    khr::{surface, swapchain, wayland_surface, win32_surface, xcb_surface, xlib_surface}, vk::{self}, Entry
};
use ash_window;
use winit::window::Window;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle,};
use arrayvec::ArrayVec;

use std::ffi::CString;

pub type DeviceName = String<{vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>;
type SmallError = String<SMALL_ERROR_LENGTH>;

fn get_required_instance_extensions<W>(
    window: &W,
    out: &mut ArrayVec::<*const i8, 8>
) -> Result<(), String<32>> 
    where
        W: HasWindowHandle + HasDisplayHandle
{
    out.push(surface::NAME.as_ptr());
    let ext = match window.display_handle().unwrap().as_raw() {
        raw_window_handle::RawDisplayHandle::Wayland(_) => wayland_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Windows(_) => win32_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xlib(_) => xlib_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xcb(_) => xcb_surface::NAME.as_ptr(),
        _ => {
            return Err(String::<32>::from_str("unsupported platform"));
        },
    };
    out.push(ext);
    Ok(())
}

pub struct Memory {
    memory: StackMemory,
    init_size: usize,
    swapchain_size: usize,
}

impl Memory {

    pub fn default() -> Option<Self> {
        let init_size = 1 << 18;
        let swapchain_size = 1 << 18;
        let memory = StackMemory::new(init_size + swapchain_size)?;
        Some(
            Self {
                memory,
                init_size,
                swapchain_size,
            }
        )
    }
}

pub struct InitSettings {
    pub app_name: AppName,
    pub app_version: u32,
    pub enable_validation: bool,
}

impl InitSettings {

    pub fn new(app_name: AppName, app_version: u32, enable_validation: bool) -> Self {
        InitSettings {
            app_name,
            app_version,
            enable_validation,
        }
    }
}

struct VkContext<'mem> {
    #[allow(unused)]
    entry: ash::Entry,
    instance: Option<ash::Instance>,
    surface_loader: surface::Instance,
    swapchain_loader: swapchain::Device,
    surface_handle: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,
    device: Option<ash::Device>,
    swapchain_context: Option<SwapchainContext<'mem>>,
}

pub struct Backend<'mem> {
    init_settings: InitSettings,
    memory: &'mem mut Memory,
    init_stack: StackAllocator<'mem>,
    swapchain_stack: StackAllocator<'mem>,
    vk_context: Option<VkContext<'mem>>,
}

impl<'mem> Backend<'mem> {

    pub fn new(
        init_settings: InitSettings,
        memory: &'mem mut Memory
    ) -> Option<Self>
    {
        let Some(init_stack) = StackAllocator::new(memory.init_size, &mut memory.memory) else {
            eprintln!("Nox backend error: failed to create init stack");
            return None
        };
        let Some(swapchain_stack) = StackAllocator::new(memory.swapchain_size, &mut memory.memory) else {
            eprintln!("Nox backend error: failed to create swapchain stack");
            return None
        };
        Some(
            Backend {
                init_settings,
                memory,
                init_stack,
                swapchain_stack,
                vk_context: None,
            }
        )
    }

    pub fn get_physical_device_name(&self) -> Option<DeviceName> {
        let vk = self.vk_context.as_ref()?;
        Some(vk.physical_device_info.device_name)
    }

    pub fn init_vulkan(&mut self, window: &Window) -> Result<(), SmallError> {
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
        let engine_name = CString::new("nox").unwrap();
        let application_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            api_version: vk::API_VERSION_1_3,
            p_application_name: CString::new(self.init_settings.app_name.as_str()).unwrap().as_c_str().as_ptr(),
            application_version: self.init_settings.app_version,
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        const VK_LAYER_KHRONOS_VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation";
        let mut instance_extensions = ArrayVec::<*const i8, 8>::new();
        let mut layers = ArrayVec::<*const i8, 1>::new();
        get_required_instance_extensions(window, &mut instance_extensions)
            .map_err(|e| String::format(format_args!(
                "Nox backend error: {:?}", e
            )))?;
        if self.init_settings.enable_validation {
            const VK_EXT_DEBUG_UTILS: &'static str = "VK_EXT_debug_utils";
            instance_extensions.push(CString::new(VK_EXT_DEBUG_UTILS).unwrap().as_ptr());
            layers.push(CString::new(VK_LAYER_KHRONOS_VALIDATION).unwrap().as_ptr());
        }
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &application_info,
            enabled_extension_count: instance_extensions.len() as u32,
            pp_enabled_extension_names: instance_extensions.as_ptr(),
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr(),
            ..Default::default()
        };
        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .map_err(|e| String::format(format_args!(
                        "Nox backend error: failed to create vulkan instance {:?}", e
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
                String::format(format_args!("Nox backend error: failed to create vulkan surface {:?}", e))
            })?
        };
        let (physical_device, physical_device_info) =
            find_suitable_physical_device(&instance, &surface_loader, surface_handle)
            .map_err(|e| {
                unsafe {
                    surface_loader.destroy_surface(surface_handle, None);
                    instance.destroy_instance(None);
                }
                String::format(format_args!("Nox backend error: failed to find suitable physical device {}", e))
            })?;
        let mut unique_device_queues = ArrayVec::<u32, 3>::new();
        let queue_indices = physical_device_info.get_queue_family_indices();
        unique_device_queues.push(queue_indices.get_graphics_index());
        if queue_indices.get_transfer_index() != unique_device_queues[0] {
            unique_device_queues.push(physical_device_info.queue_family_indices.transfer.index);
        }
        if !unique_device_queues.contains(&queue_indices.get_compute_index()) {
            unique_device_queues.push(queue_indices.get_compute_index());
        }
        let mut device_queue_create_infos = ArrayVec::<vk::DeviceQueueCreateInfo, 3>::new();
        let queue_priority = 1.0;
        for queue_family_index in unique_device_queues {
            device_queue_create_infos.push(
                vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    queue_count: 1,
                    queue_family_index,
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
            p_queue_create_infos: device_queue_create_infos.as_ptr(),
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
                    String::format(format_args!("Nox backend error: failed to create device {:?}", e))
                })?
        };
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        self.vk_context =
            Some(
                VkContext {
                    entry,
                    instance: Some(instance),
                    surface_loader,
                    swapchain_loader,
                    surface_handle,
                    physical_device,
                    physical_device_info,
                    device: Some(device),
                    swapchain_context: None,
                },
            );
        Ok(())
    }

    pub fn update_swapchain(&'mem mut self, framebuffer_extent: vk::Extent2D) -> Result<(), SmallError> {
        let Some(vk) = &mut self.vk_context else {
            return Err(String::from_str("vulkan context was None"))
        };
        let Some(device) = &vk.device else {
            return Err(String::from_str("device was None"))
        };
        if let Some(mut context) = vk.swapchain_context.take() {
            context.destroy(device, &vk.swapchain_loader);
        }
        vk.swapchain_context = match SwapchainContext::new(
            device,
            &vk.surface_loader,
            &vk.swapchain_loader,
            vk.physical_device,
            vk.surface_handle,
            &mut self.swapchain_stack,
            framebuffer_extent,
        ) {
            Ok(context) => context,
            Err(err) => return Err(err),
        };
        Ok(())
    }
}

impl<'mem> Drop for Backend<'mem> {

    fn drop(&mut self) {
        if let Some(mut vk) = self.vk_context.take() {
            unsafe {
                if let Some(device) = vk.device.take() {
                    if let Some(mut context) = vk.swapchain_context.take() {
                        context.destroy(&device, &vk.swapchain_loader);
                    }
                    device.destroy_device(None);
                }
                vk.surface_loader.destroy_surface(vk.surface_handle, None);
                vk.surface_handle = vk::SurfaceKHR::null();
                if let Some(instance) = vk.instance.take() {
                    instance.destroy_instance(None);
                }
                vk.physical_device = vk::PhysicalDevice::null();
            }
        }
        println!("Nox backend message: terminated backend");
    }
}
