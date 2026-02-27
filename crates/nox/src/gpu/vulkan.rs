mod device;

pub mod nox_ash {

    pub use nox_ash::*;

    pub use super::device::Device;
}

use std::ffi::{CString, CStr};
use raw_window_handle::{HasDisplayHandle};
use compact_str::{format_compact, CompactString};
use ahash::AHashSet;

use nox_mem::{
    vec::{ArrayVec, Vec32, Vector}, vec32
};

use nox_log::{warn, info};

use nox_ash::{
    khr::{
        surface, swapchain,
        get_surface_capabilities2,
        wayland_surface, win32_surface,
        xcb_surface, xlib_surface,
        present_wait2,
    },
    ext::{
        debug_utils,
    },
    vk,
    Entry
};

use super::{
    physical_device::{self, find_suitable_physical_device, PhysicalDeviceInfo},
    GpuAttributes, Layer, InstanceExtension, ext,
    BaseDeviceFeatures,
};

use crate::dev::{
    error::{Context, Error, Result},
    prelude::Version,
    has_bits,
};

pub struct Vulkan {
    entry: nox_ash::Entry,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,
    instance: nox_ash::Instance,
    device: device::Device,
    surface_instance: surface::Instance,
    get_surface_capabilities2_instance: get_surface_capabilities2::Instance,
    swapchain_device: swapchain::Device,
    present_wait2_device: present_wait2::Device,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
    compute_queue: vk::Queue,
    max_memory_allocation_size: vk::DeviceSize,
    frame_timeout: u64,
    enabled_layers: u32,
    enabled_instance_extensions: u32,
    enabled_device_extensions: ext::EnabledDeviceExtensions,
    base_device_features: BaseDeviceFeatures,
}

impl Vulkan {

    pub(crate) fn new<H: HasDisplayHandle>(
        display_handle: &H,
        attributes: &GpuAttributes,
    ) -> Result<Vulkan> {
        let entry = unsafe { Entry::load().context("failed to load vulkan")? };
        match unsafe { entry.try_enumerate_instance_version() } {
            Ok(version) => {
                if let Some(version) = version {
                    let version = Version::from(version);
                    info!("Vulkan loader version: {}.{}.{}",
                        version.major(),
                        version.minor(),
                        version.patch(),
                    );
                } else {
                    warn!("failed to enumerate vulkan loader version");
                }
            },
            Err(result) => {
                warn!("failed to enumerate vulkan loader version {}", result);
            },
        };
        let app_name = CString
            ::new(attributes.app_name
                .chars()
                .filter(|&c| c != '\0')
                .collect::<CompactString>()
            ).unwrap();
        let engine_name = CString::new("nox").unwrap();
        let application_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_application_name: app_name.as_ptr(),
            application_version: attributes.app_version.as_u32(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_4,
            ..Default::default()
        };
        const VK_LAYER_KHRONOS_VALIDATION: &CStr = c"VK_LAYER_KHRONOS_validation";
        let mut instance_extensions = Vec32::<(&CStr, bool)>
            ::with_capacity(8);
        let mut found_instance_extensions = Vec32::<*const i8>
            ::with_capacity(8);
        let mut found_instance_extensions_hashed = AHashSet::default();
        let mut layers = Vec32::<(&CStr, bool)>
            ::with_capacity(8);
        let mut found_layers = Vec32::<*const i8>
            ::with_capacity(8);
        let mut found_layers_hashed = AHashSet::default();
        get_required_instance_extensions(display_handle, &mut instance_extensions)?;
        if let Some(required) = attributes.contains_instance_extension(InstanceExtension::DebugUtils) {
            instance_extensions.push((debug_utils::NAME, required));
        }
        if let Some(required) = attributes.contains_layer(Layer::KhronosValidation) {
            layers.push((VK_LAYER_KHRONOS_VALIDATION, required));
        }
        verify_instance_extensions(
            &entry, &instance_extensions,
            &mut found_instance_extensions, &mut found_instance_extensions_hashed
        )?;
        verify_instance_layers(
            &entry, &layers,
            &mut found_layers, &mut found_layers_hashed,
        )?;
        let mut enabled_instance_extensions = 0;
        let mut enabled_layers = 0;
        if found_instance_extensions_hashed.contains(debug_utils::NAME) {
            enabled_instance_extensions |= InstanceExtension::DebugUtils;
        }
        if found_layers_hashed.contains(VK_LAYER_KHRONOS_VALIDATION) {
            enabled_layers |= Layer::KhronosValidation;
        }
        let version = unsafe {
            let Some(version) = entry
                .try_enumerate_instance_version()
                .context_with(|| format_compact!("failed to enumerate Vulkan instance version"))?
            else {
                return Err(Error::just_context(
                    "Nox requires at least Vulkan version 1.1, enumerated version was 1.0"
                ))
            };
            version
        };
        if version < vk::API_VERSION_1_1 {
            return Err(Error::just_context(format_compact!(
                "Nox requires at least Vulkan version 1.1, enumerated version was {}",
                Version(version),
            )))
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
        let mut device_extensions = Vec32::with_capacity(attributes.device_extensions.len());
        device_extensions.extend(ext::core_extensions());
        device_extensions.extend(attributes.device_extensions.iter().cloned());
        let mut device_extension_infos = vec32![];
        device_extension_infos.extend(device_extensions
            .iter().filter_map(|ext| ext.get_info(attributes))
        );
        let (physical_device, physical_device_info) =
            find_suitable_physical_device(
                &instance,
                attributes,
                &device_extension_infos,
            ).inspect_err(|_| {
                unsafe {
                    instance.destroy_instance(None);
                }
            })?;
        let mut unique_device_queues = ArrayVec::<u32, 3>::new();
        let queue_family_indices = physical_device_info.queue_family_indices();
        unique_device_queues.push(queue_family_indices.graphics_index());
        if !unique_device_queues.contains(&queue_family_indices.transfer_index()) {
            unique_device_queues.push(queue_family_indices.transfer_index());
        }
        if !unique_device_queues.contains(&queue_family_indices.compute_index()) {
            unique_device_queues.push(queue_family_indices.compute_index());
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
                });
        }
        let enabled_device_extension_names: Vec32<_> = device_extension_infos
            .iter()
            .map(|ext| ext.name.as_ptr())
            .collect();
        let mut vulkan_12_features = None;
        let mut vulkan_14_features = None;
        let mut enabled_device_extensions = ext::EnabledDeviceExtensions::new();
        let mut physical_device_context = unsafe { ext::PhysicalDeviceContext::new(
            &instance,
            &mut vulkan_12_features,
            &mut vulkan_14_features,
            Some(&mut enabled_device_extensions),
            physical_device,
            physical_device_info.api_version(),
        ) };
        let mut enable_features = Vec32::<vk::ExtendsDeviceCreateInfoObj>::with_capacity(device_extensions.len());
        for ext in device_extensions {
            if let Some(feature) = ext.register(&mut physical_device_context) {
                let s_type = feature.s_type();
                if enable_features.iter().any(|f| f.s_type() == s_type) {
                    return Err(Error::just_context(format_compact!(
                        "device feature with structure type {:?} included twice when resolving device extensions",
                        feature.s_type(),
                    )))
                }
                enable_features.push(feature);
            }
        }
        let features = attributes.required_features.into();
        let mut device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            queue_create_info_count: device_queue_create_infos.len() as u32,
            p_queue_create_infos: device_queue_create_infos.as_ptr() as _,
            enabled_extension_count: enabled_device_extension_names.len() as u32,
            pp_enabled_extension_names: enabled_device_extension_names.as_ptr(),
            p_enabled_features: &features,
            ..Default::default()
        };
        for enable_feature in &enable_features {
            device_create_info = device_create_info.push_next(enable_feature.as_mut());
        }
        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .context_with(|| {
                    instance.destroy_instance(None);
                    "failed to create vulkan device"
                })?
        };
        let get_surface_capabilities2_instance = get_surface_capabilities2::Instance::new(
            &entry, &instance
        );
        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics_index(), 0) };
        let transfer_queue = unsafe { device.get_device_queue(queue_family_indices.transfer_index(), 0) };
        let compute_queue = unsafe { device.get_device_queue(queue_family_indices.compute_index(), 0) };
        let swapchain_device = swapchain::Device::new(&instance, &device);
        let present_wait2_device = present_wait2::Device::new(&instance, &device);
        let mut properties_11 = vk::PhysicalDeviceVulkan11Properties::default();
        physical_device_context.get_properties(&mut properties_11);
        let max_memory_allocation_size = properties_11.max_memory_allocation_size;
        if has_bits!(enabled_layers, Layer::KhronosValidation) {
            info!("Khronos validation enabled");
        }
        Ok(Self {
            entry,
            instance,
            surface_instance,
            get_surface_capabilities2_instance,
            swapchain_device,
            present_wait2_device,
            physical_device,
            physical_device_info,
            device: device::Device::new(&instance, device, &physical_device_info),
            graphics_queue,
            transfer_queue,
            compute_queue,
            frame_timeout: attributes.frame_timeout.as_nanos() as u64,
            max_memory_allocation_size,
            enabled_layers,
            enabled_instance_extensions,
            enabled_device_extensions,
            base_device_features: attributes.required_features,
        })
    }

    #[inline(always)]
    pub fn entry(&self) -> &nox_ash::Entry {
        &self.entry
    }

    #[inline(always)]
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    #[inline(always)]
    pub fn physical_device_info(&self) -> &PhysicalDeviceInfo<'static> {
        &self.physical_device_info
    }

    #[inline(always)]
    pub fn is_layer_enabled(&self, layer: Layer) -> bool {
        self.enabled_layers & layer == layer
    }

    #[inline(always)]
    pub fn is_instance_extension_enabled(&self, extension: InstanceExtension) -> bool {
        self.enabled_instance_extensions & extension == extension
    }

    #[inline(always)]
    pub fn base_device_features(&self) -> &BaseDeviceFeatures {
        &self.base_device_features
    }

    #[inline(always)]
    pub fn enabled_device_extensions(&self) -> &ext::EnabledDeviceExtensions {
        &self.enabled_device_extensions
    }

    #[inline(always)]
    pub fn instance(&self) -> &nox_ash::Instance {
        &self.instance
    }

    #[inline(always)]
    pub fn device(&self) -> &device::Device {
        &self.device
    }

    #[inline(always)]
    pub fn surface_instance(&self) -> &surface::Instance {
        &self.surface_instance
    }

    #[inline(always)]
    pub fn get_surface_capabilities2_instance(&self) -> &get_surface_capabilities2::Instance {
        &self.get_surface_capabilities2_instance
    }

    #[inline(always)]
    pub fn swapchain_device(&self) -> &swapchain::Device {
        &self.swapchain_device
    }

    #[inline(always)]
    pub fn present_wait2_device(&self) -> &present_wait2::Device {
        &self.present_wait2_device
    }

    #[inline(always)]
    pub fn max_memory_allocation_size(&self) -> vk::DeviceSize {
        self.max_memory_allocation_size
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

    #[inline(always)]
    pub fn frame_timeout(&self) -> u64 {
        self.frame_timeout
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

fn get_required_instance_extensions<H>(
    handle: &H,
    out: &mut Vec32::<(&CStr, bool)>,
) -> Result<()>
    where
        H: HasDisplayHandle
{
    out.push((surface::NAME, true));
    let ext = match handle.display_handle().unwrap().as_raw() {
        raw_window_handle::RawDisplayHandle::Wayland(_) => wayland_surface::NAME,
        raw_window_handle::RawDisplayHandle::Windows(_) => win32_surface::NAME,
        raw_window_handle::RawDisplayHandle::Xlib(_) => xlib_surface::NAME,
        raw_window_handle::RawDisplayHandle::Xcb(_) => xcb_surface::NAME,
        _ => {
            return Err(Error::just_context("unsupported platform"));
        },
    };
    out.push((ext, true));
    out.push((get_surface_capabilities2::NAME, true));
    Ok(())
}

fn verify_instance_layers<'a>(
    entry: &Entry,
    layers: &[(&'a CStr, bool)],
    found: &mut Vec32<*const i8>,
    found_hash: &mut AHashSet<&'a CStr>,
) -> Result<()>
{
    let available = unsafe { entry
        .enumerate_instance_layer_properties()
        .context("failed to enumerate instance layers")?
    };
    for &(layer, required) in layers {
        if !available
            .iter()
            .any(|a| {
                layer == a.layer_name_as_c_str().unwrap_or_default()
            })
        {
            if required {
                return Err(Error::just_context(format_compact!("instance layer {layer:?} not present")))
            } else {
                warn!("optional instance layer {layer:?} not present");
            }
        } else {
            found.push(layer.as_ptr());
            found_hash.insert(layer);
        }
    }
    Ok(())
}

fn verify_instance_extensions<'a>(
    entry: &Entry,
    extensions: &[(&'a CStr, bool)],
    found: &mut Vec32<*const i8>,
    found_hash: &mut AHashSet<&'a CStr>,
) -> Result<()>
{
    let available = unsafe {
        entry
            .enumerate_instance_extension_properties(None)
            .context("failed to enumerate instance extensions")?
    };
    for &(extension, required) in extensions {
        if !available
            .iter()
            .any(|a| {
                extension == a.extension_name_as_c_str().unwrap_or_default()
            })
        {
            if required {
                return Err(Error::just_context(format_compact!("instance extension {extension:?} not present")))
            } else {
                warn!("optional instance extension {extension:?} not present");
            }
        } else {
            found.push(extension.as_ptr());
            found_hash.insert(extension);
        }
    }
    Ok(())
}
