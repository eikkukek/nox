use std::ffi::CString;
use core::str::FromStr;
use winit::event_loop::ActiveEventLoop;
use ash::{
    khr::{surface, swapchain, wayland_surface, win32_surface, xcb_surface, xlib_surface},
    vk,
    Entry
};
use raw_window_handle::{HasDisplayHandle};
use compact_str::{format_compact, CompactString};
use rustc_hash::FxHashSet;

use nox_mem::{
    vec_types::{ArrayVec, GlobalVec, Vector},
    string_types::ArrayString,
};

use nox_log::{warn, info};

use super::{
    physical_device::{self, find_suitable_physical_device, PhysicalDeviceInfo},
    GpuAttributes, Layer, Extension,
};

use crate::dev::{
    error::{Context, Error, ErrorContext, Result, location}, export::Version, has_bits
};

pub(crate) struct Vulkan {
    entry: ash::Entry,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,
    instance: ash::Instance,
    device: ash::Device,
    surface_instance: surface::Instance,
    swapchain_device: swapchain::Device,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
    compute_queue: vk::Queue,
    enabled_extensions: u8,
    enabled_layers: u8,
}

impl Vulkan {

    pub fn new(
        event_loop: &ActiveEventLoop,
        attributes: &GpuAttributes,
    ) -> Result<Vulkan> {
        let entry = unsafe { Entry::load().unwrap() };
        match unsafe { entry.try_enumerate_instance_version() } {
            Ok(v) => {
                if v.is_none() {
                    warn!("failed to enumerate vulkan loader version");
                }
                else {
                    let version = Version::from(v.unwrap());
                    info!("Vulkan loader version: {}.{}.{}",
                        version.major(),
                        version.minor(),
                        version.patch(),
                    );

                }
            },
            Err(result) => {
                warn!("failed to enumerate vulkan loader version {}", result);
            },
        };
        let app_name = CString
            ::new(attributes.app_name
                .chars()
                .filter(|&c| c != '\0').collect::<CompactString>()
            ).unwrap();
        let engine_name = CString::new("nox").unwrap();
        let application_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            api_version: vk::API_VERSION_1_3,
            p_application_name: app_name.as_ptr(),
            application_version: attributes.app_version.as_u32(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let mut instance_extensions = GlobalVec::<(*const i8, bool)>
            ::with_capacity(8);
        let mut found_instance_extensions = GlobalVec::<*const i8>
            ::with_capacity(8);
        let mut found_instance_extensions_hashed = FxHashSet::default();
        let mut layers = GlobalVec::<(*const i8, bool)>
            ::with_capacity(8);
        let mut found_layers = GlobalVec::<*const i8>
            ::with_capacity(8);
        let mut found_layers_hashed = FxHashSet::default();
        get_required_instance_extensions(event_loop, &mut instance_extensions)?;
        let validation_layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        if let Some(required) = attributes.contains_extension(Extension::DebugUtils) {
            instance_extensions.push((ash::ext::debug_utils::NAME.as_ptr(), required));
        }
        if let Some(required) = attributes.contains_layer(Layer::KhronosValidation) {
            layers.push((validation_layer_name.as_ptr(), required));
        }
        verify_instance_extensions(
            &entry, &instance_extensions,
            &mut found_instance_extensions, &mut found_instance_extensions_hashed
        )?;
        verify_instance_layers(
            &entry, &layers,
            &mut found_layers, &mut found_layers_hashed,
        )?;
        let mut enabled_extensions = 0;
        let mut enabled_layers = 0;
        if found_instance_extensions_hashed.contains(ash::ext::debug_utils::NAME) {
            enabled_extensions |= Extension::DebugUtils;
        }
        if found_layers_hashed.contains(&validation_layer_name) {
            enabled_layers |= Layer::KhronosValidation;
        }
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &application_info,
            enabled_extension_count: found_instance_extensions.len() as u32,
            pp_enabled_extension_names: found_instance_extensions.as_ptr() as _,
            enabled_layer_count: found_layers.len() as u32,
            pp_enabled_layer_names: found_layers.as_ptr() as _,
            ..Default::default()
        };
        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .context("failed to create vulkan instance")?
        };
        let surface_instance = surface::Instance::new(&entry, &instance);
        let (physical_device, physical_device_info) =
            find_suitable_physical_device(&instance)
            .map_err(|err| {
                unsafe {
                    instance.destroy_instance(None);
                }
                err
            })?;
        let mut unique_device_queues = ArrayVec::<u32, 3>::new();
        let queue_family_indices = physical_device_info.queue_family_indices();
        unique_device_queues.push(queue_family_indices.graphics_index()).ok();
        if !unique_device_queues.contains(&queue_family_indices.transfer_index()) {
            unique_device_queues.push(queue_family_indices.transfer_index()).unwrap();
        }
        if !unique_device_queues.contains(&queue_family_indices.compute_index()) {
            unique_device_queues.push(queue_family_indices.compute_index()).unwrap();
        }
        let mut device_queue_create_infos = ArrayVec::<vk::DeviceQueueCreateInfo, 3>::new();
        let queue_priority = 1.0;
        for queue_family_index in &unique_device_queues {
            device_queue_create_infos
                .push(
                    vk::DeviceQueueCreateInfo {
                        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                        queue_count: 1,
                        queue_family_index: *queue_family_index,
                        p_queue_priorities: &queue_priority,
                        ..Default::default()
                }).unwrap();
        }
        const ENABLED_DEVICE_EXTENSION_NAMES: [*const i8; 3] = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::timeline_semaphore::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            sample_rate_shading: vk::TRUE,
            sampler_anisotropy: vk::TRUE,
            fill_mode_non_solid: physical_device_info.features().sample_rate_shading,
            ..Default::default()
        };
        let features_13 = vk::PhysicalDeviceVulkan13Features {
            dynamic_rendering: vk::TRUE,
            ..Default::default()
        };
        let features_12 = vk::PhysicalDeviceVulkan12Features {
            p_next: (&features_13 as *const _) as _,
            timeline_semaphore: vk::TRUE,
            descriptor_binding_update_unused_while_pending: vk::TRUE,
            descriptor_binding_uniform_buffer_update_after_bind: vk::TRUE,
            descriptor_binding_storage_buffer_update_after_bind: vk::TRUE,
            descriptor_binding_storage_image_update_after_bind: vk::TRUE,
            descriptor_binding_sampled_image_update_after_bind: vk::TRUE,
            ..Default::default()
        };
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: (&features_12 as *const _) as _,
            queue_create_info_count: device_queue_create_infos.len() as u32,
            p_queue_create_infos: device_queue_create_infos.as_ptr() as _,
            enabled_extension_count: ENABLED_DEVICE_EXTENSION_NAMES.len() as u32,
            pp_enabled_extension_names: ENABLED_DEVICE_EXTENSION_NAMES.as_ptr(),
            p_enabled_features: &features,
            ..Default::default()
        };
        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .context_with(|| {
                    instance.destroy_instance(None);
                    "failed to create vulkan device"
                })?
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics_index(), 0) };
        let transfer_queue = unsafe { device.get_device_queue(queue_family_indices.transfer_index(), 0) };
        let compute_queue = unsafe { device.get_device_queue(queue_family_indices.compute_index(), 0) };
        let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);
        if has_bits!(enabled_layers, Layer::KhronosValidation) {
            info!("Khronos validation enabled");
        }
        Ok(Self {
            entry,
            instance: instance,
            surface_instance: surface_instance,
            swapchain_device: swapchain_device,
            physical_device,
            physical_device_info: physical_device_info,
            device: device,
            graphics_queue,
            transfer_queue: transfer_queue,
            compute_queue: compute_queue,
            enabled_extensions,
            enabled_layers,
        })
    }

    #[inline(always)]
    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    #[inline(always)]
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    #[inline(always)]
    pub fn physical_device_info(&self) -> &PhysicalDeviceInfo {
        &self.physical_device_info
    }

    #[inline(always)]
    pub fn is_layer_enabled(&self, layer: Layer) -> bool {
        self.enabled_layers & layer == layer
    }

    #[inline(always)]
    pub fn is_extension_enabled(&self, extension: Extension) -> bool {
        self.enabled_extensions & extension == extension
    }

    #[inline(always)]
    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    #[inline(always)]
    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    #[inline(always)]
    pub fn surface_instance(&self) -> &surface::Instance {
        &self.surface_instance
    }

    #[inline(always)]
    pub fn swapchain_device(&self) -> &swapchain::Device {
        &self.swapchain_device
    }

    #[inline(always)]
    pub fn queue_family_indices(&self) -> physical_device::QueueFamilyIndices {
        self.physical_device_info.queue_family_indices()
    }

    #[inline(always)]
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    #[inline(always)]
    pub fn transfer_queue(&self) -> vk::Queue {
        self.transfer_queue
    }

    #[inline(always)]
    pub fn compute_queue(&self) -> vk::Queue {
        self.compute_queue
    }
}

impl Drop for Vulkan {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn get_required_instance_extensions<W>(
    window: &W,
    out: &mut GlobalVec::<(*const i8, bool)>,
) -> Result<()>
    where
        W: HasDisplayHandle
{
    out.push((surface::NAME.as_ptr(), true));
    let ext = match window.display_handle().unwrap().as_raw() {
        raw_window_handle::RawDisplayHandle::Wayland(_) => wayland_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Windows(_) => win32_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xlib(_) => xlib_surface::NAME.as_ptr(),
        raw_window_handle::RawDisplayHandle::Xcb(_) => xcb_surface::NAME.as_ptr(),
        _ => {
            return Err(Error::just_context("unsupported platform"));
        },
    };
    out.push((ext, true));
    Ok(())
}

fn verify_instance_layers(
    entry: &Entry,
    layers: &[(*const i8, bool)],
    found: &mut GlobalVec<*const i8>,
    found_hash: &mut FxHashSet<CString>,
) -> Result<()>
{
    let available = unsafe { entry
        .enumerate_instance_layer_properties()
        .context("failed to enumerate instance layers")?
    };
    for &(layer, required) in layers {
        let string = unsafe {
            ArrayString::<{vk::MAX_EXTENSION_NAME_SIZE}>
                ::from_ascii_ptr(layer)
                .context_with(|| ErrorContext::StringConversionError(location!()))?
        };
        if available
            .iter()
            .find(|a| {
                match ArrayString::<{vk::MAX_EXTENSION_NAME_SIZE}>::from_ascii(&a.layer_name) {
                    Ok(s) => string == s,
                    Err(_) => false,
                }
            }).is_none()
        {
            if required {
                return Err(Error::just_context(format_compact!("instance layer {string} not present")))
            } else {
                warn!("optional instance layer {string} not present");
            }
        } else {
            found.push(layer);
            found_hash.insert(CString::new(string.as_str()).unwrap());
        }
    }
    Ok(())
}

fn verify_instance_extensions(
    entry: &Entry,
    extensions: &[(*const i8, bool)],
    found: &mut GlobalVec<*const i8>,
    found_hash: &mut FxHashSet<CString>,
) -> Result<()>
{
    let available = unsafe {
        entry
            .enumerate_instance_extension_properties(None)
            .context("failed to enumerate instance extensions")?
    };
    for &(extension, required) in extensions {
        let string = unsafe {
            ArrayString::<{vk::MAX_EXTENSION_NAME_SIZE}>
                ::from_ascii_ptr(extension)
                .context_with(|| ErrorContext::StringConversionError(location!()))?
        };
        if available
            .iter()
            .find(|a| {
                match ArrayString::<{vk::MAX_EXTENSION_NAME_SIZE}>::from_ascii(&a.extension_name) {
                    Ok(s) => string == s,
                    Err(_) => false,
                }
            }).is_none()
        {
            if required {
                return Err(Error::just_context(format_compact!("instance extension {string} not present")))
            } else {
                warn!("optional instance extension {string} not present");
            }
        } else {
            found.push(extension);
            found_hash.insert(CString::from_str(string.as_str()).unwrap());
        }
    }
    Ok(())
}
