use std::ffi::{CString, CStr};
use winit::event_loop::ActiveEventLoop;
use raw_window_handle::{HasDisplayHandle};
use compact_str::{format_compact, CompactString};
use ahash::{AHashSet, AHashMap};

use nox_mem::{
    num::NonZeroInteger, vec::{ArrayVec, Vec32, Vector}, vec32
};

use nox_log::{warn, info};

use nox_ash::{
    khr::{
        surface, swapchain, timeline_semaphore,
        get_surface_capabilities2,
        synchronization2, dynamic_rendering,
        create_renderpass2, depth_stencil_resolve, // required for dynamic rendering
        wayland_surface, win32_surface,
        xcb_surface, xlib_surface,
        maintenance4, // device buffer and image memory requirements
        maintenance5, // vkCmdBindIndexBuffer2KHR
        push_descriptor,
        present_wait2, present_id2,
    },
    ext::{
        extended_dynamic_state, // vkCmdBindVertexBuffers2KHR
        robustness2, // null descriptors
        debug_utils,
    },
    vk::{self, TaggedStructure, ExtendsStructureExt},
    Entry
};

use super::{
    physical_device::{self, find_suitable_physical_device, PhysicalDeviceInfo},
    GpuAttributes, Layer, InstanceExtension, DeviceExtension,
};

use crate::dev::{
    error::{Context, Error, Result},
    prelude::Version,
    has_bits,
};

pub(crate) struct Vulkan {
    entry: nox_ash::Entry,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo<'static>,
    instance: nox_ash::Instance,
    device: nox_ash::Device,
    surface_instance: surface::Instance,
    get_surface_capabilities2_instance: get_surface_capabilities2::Instance,
    swapchain_device: swapchain::Device,
    present_wait2_device: present_wait2::Device,
    timeline_semaphore_device: timeline_semaphore::Device,
    dynamic_rendering_device: dynamic_rendering::Device,
    synchronization2_device: synchronization2::Device,
    maintenance4_device: maintenance4::Device,
    maintenance5_device: maintenance5::Device,
    push_descriptor_device: push_descriptor::Device,
    extended_dynamic_state_device: extended_dynamic_state::Device,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
    compute_queue: vk::Queue,
    max_memory_allocation_size: vk::DeviceSize,
    max_push_descriptors: Option<u32>,
    frame_timeout: u64,
    enabled_layers: u32,
    enabled_instance_extensions: u32,
    enabled_device_extensions: u32,
}

impl Vulkan {

    pub fn new(
        event_loop: &ActiveEventLoop,
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
            api_version: vk::API_VERSION_1_3,
            p_application_name: app_name.as_ptr(),
            application_version: attributes.app_version.as_u32(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
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
        get_required_instance_extensions(event_loop, &mut instance_extensions)?;
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
        let mut required_device_extensions = vec32![
            (create_renderpass2::NAME, Version::new(1, 2, 0)),
            (depth_stencil_resolve::NAME, Version::new(1, 2, 0)),
            (timeline_semaphore::NAME, Version::new(1, 2, 0)),
            (dynamic_rendering::NAME, Version::new(1, 3, 0)),
            (synchronization2::NAME, Version::new(1, 3, 0)),
            (extended_dynamic_state::NAME, Version::new(1, 3, 0)),
            (maintenance4::NAME, Version::new(1, 3, 0)),
            (maintenance5::NAME, Version::new(1, 3, 0)),
            (present_id2::NAME, Version::MAX),
            (present_wait2::NAME, Version::MAX),
            (swapchain::NAME, Version::MAX),
            (robustness2::NAME, Version::MAX),
        ];
        let mut extended_device_features = [
            vk::PhysicalDeviceTimelineSemaphoreFeatures::boxed_default(),
            vk::PhysicalDeviceDynamicRenderingFeatures::boxed_default(),
            vk::PhysicalDeviceSynchronization2Features::boxed_default(),
            vk::PhysicalDeviceRobustness2FeaturesEXT::boxed_default(),
            vk::PhysicalDevicePresentId2FeaturesKHR::boxed_default(),
            vk::PhysicalDevicePresentWait2FeaturesKHR::boxed_default(),
        ];
        let mut device_feature_check = AHashMap::<
            vk::StructureType,
            &dyn Fn(&vk::ExtendsPhysicalDeviceFeatures2Obj
            ) -> Option<CompactString>>::default();
        device_feature_check.entry(vk::PhysicalDeviceDynamicRenderingFeatures::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDeviceDynamicRenderingFeatures::as_ref(features)
                .unwrap()
            };
            (features.dynamic_rendering == 0).then(|| {
                CompactString::new("dynamic rendering")
            })
        });
        device_feature_check.entry(vk::PhysicalDeviceTimelineSemaphoreFeatures::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDeviceTimelineSemaphoreFeatures::as_ref(features)
                .unwrap()
            };
            (features.timeline_semaphore == 0).then(|| {
                CompactString::new("timeline semaphore")
            })
        });
        device_feature_check.entry(vk::PhysicalDeviceSynchronization2Features::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDeviceSynchronization2Features::as_ref(features)
                .unwrap()
            };
            (features.synchronization2 == 0).then(|| {
                CompactString::new("synchronization2")
            })
        });
        device_feature_check.entry(vk::PhysicalDeviceRobustness2FeaturesEXT::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDeviceRobustness2FeaturesEXT::as_ref(features)
                .unwrap()
            };
            (features.null_descriptor == 0).then(|| {
                CompactString::new("null descriptor")
            })
        });
        device_feature_check.entry(vk::PhysicalDevicePresentId2FeaturesKHR::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDevicePresentId2FeaturesKHR::as_ref(features)
                .unwrap()
            };
            (features.present_id2 == 0).then(|| {
                CompactString::new("present id2")
            })
        });
        device_feature_check.entry(vk::PhysicalDevicePresentWait2FeaturesKHR::STRUCTURE_TYPE)
        .or_insert(&|features: &vk::ExtendsPhysicalDeviceFeatures2Obj| {
            let features = unsafe {
                vk::PhysicalDevicePresentWait2FeaturesKHR::as_ref(features)
                .unwrap()
            };
            (features.present_wait2 == 0).then(|| {
                CompactString::new("present wait2")
            })
        });
        let mut extended_device_properties = vec32![
            vk::PhysicalDeviceVulkan11Properties::boxed_default(),
        ];
        let device_property_check = AHashMap::<
            vk::StructureType,
            &dyn Fn(&vk::ExtendsPhysicalDeviceProperties2Obj
        ) -> Option<CompactString>>::default();
        if attributes.contains_device_extension(DeviceExtension::PushDescriptor) {
            extended_device_properties.push(vk::PhysicalDevicePushDescriptorPropertiesKHR::boxed_default());
            required_device_extensions.push((push_descriptor::NAME, Version::new(1, 4, 0)));
        }
        let (physical_device, physical_device_info) =
            find_suitable_physical_device(
                &instance,
                &required_device_extensions,
                |features| {
                    if features.sampler_anisotropy == 0 {
                        Some(CompactString::new("sampler anisotropy"))
                    } else if features.sample_rate_shading == 0 {
                        Some(CompactString::new("sample rate shading"))
                    } else {
                        None
                    }
                },
                extended_device_features.as_mut_slice(),
                |features| {
                    if let Some(f) = device_feature_check.get(&features.s_type()) {
                        f(features)
                    } else {
                        None
                    }
                },
                |properties| {
                    (properties.api_version < vk::API_VERSION_1_1).then(|| {
                        format_compact!(
                            "Nox requires at least Vulkan version 1.1, found version was {}",
                            Version::from_u32(properties.api_version)
                        )
                    })
                },
                extended_device_properties.as_mut_slice(),
                |properties| {
                    if let Some(f) = device_property_check.get(&properties.s_type()) {
                        f(properties)
                    } else {
                        None
                    }
                },
            )
            .inspect_err(|_| {
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
        let enabled_device_extension_names: Vec32<_> = required_device_extensions
            .iter()
            .map(|(ext, _)| ext.as_ptr())
            .collect();
        let features = vk::PhysicalDeviceFeatures {
            sample_rate_shading: vk::TRUE,
            sampler_anisotropy: vk::TRUE,
            fill_mode_non_solid: physical_device_info.features().fill_mode_non_solid,
            ..Default::default()
        };
        let mut timeline_semaphore = vk::PhysicalDeviceTimelineSemaphoreFeatures {
            s_type: vk::PhysicalDeviceTimelineSemaphoreFeatures::STRUCTURE_TYPE,
            timeline_semaphore: vk::TRUE,
            ..Default::default()
        };
        let mut synchronization2 = vk::PhysicalDeviceSynchronization2Features {
            s_type: vk::PhysicalDeviceSynchronization2Features::STRUCTURE_TYPE,
            synchronization2: vk::TRUE,
            ..Default::default()
        };
        let mut dynamic_rendering = vk::PhysicalDeviceDynamicRenderingFeatures {
            s_type: vk::PhysicalDeviceDynamicRenderingFeatures::STRUCTURE_TYPE,
            dynamic_rendering: vk::TRUE,
            ..Default::default()
        };
        let mut robustness2 = vk::PhysicalDeviceRobustness2FeaturesEXT {
            s_type: vk::PhysicalDeviceRobustness2FeaturesEXT::STRUCTURE_TYPE,
            null_descriptor: vk::TRUE,
            ..Default::default()
        };
        let mut present_id2 = vk::PhysicalDevicePresentId2FeaturesKHR {
            present_id2: vk::TRUE,
            ..Default::default()
        };
        let mut present_wait2 = vk::PhysicalDevicePresentWait2FeaturesKHR {
            present_wait2: vk::TRUE,
            ..Default::default()
        };
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            queue_create_info_count: device_queue_create_infos.len() as u32,
            p_queue_create_infos: device_queue_create_infos.as_ptr() as _,
            enabled_extension_count: enabled_device_extension_names.len() as u32,
            pp_enabled_extension_names: enabled_device_extension_names.as_ptr(),
            p_enabled_features: &features,
            ..Default::default()
        }.push_next(&mut timeline_semaphore).push_next(&mut synchronization2)
        .push_next(&mut dynamic_rendering).push_next(&mut robustness2)
        .push_next(&mut present_id2).push_next(&mut present_wait2);
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
        let dynamic_rendering_device = dynamic_rendering::Device::new(&instance, &device);
        let timeline_semaphore_device = timeline_semaphore::Device::new(&instance, &device);
        let synchronization2_device = synchronization2::Device::new(&instance, &device);
        let extended_dynamic_state_device = extended_dynamic_state::Device::new(&instance, &device);
        let maintenance4_device = maintenance4::Device::new(&instance, &device);
        let maintenance5_device = maintenance5::Device::new(&instance, &device);
        let push_descriptor_device = push_descriptor::Device::new(&instance, &device);
        let max_push_descriptors = unsafe {
            physical_device_info.extended_property(vk::PhysicalDevicePushDescriptorPropertiesKHR::STRUCTURE_TYPE)
                .map(|prop| {
                    vk::PhysicalDevicePushDescriptorPropertiesKHR::as_ref(prop)
                    .unwrap()
                    .max_push_descriptors
                })
        };
        let max_memory_allocation_size = unsafe {
            let obj = physical_device_info.extended_property(
                vk::PhysicalDeviceVulkan11Properties::STRUCTURE_TYPE
            ).unwrap();
            vk::PhysicalDeviceVulkan11Properties::as_ref(obj).unwrap()
            .max_memory_allocation_size
        };
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
            dynamic_rendering_device,
            synchronization2_device,
            extended_dynamic_state_device,
            timeline_semaphore_device,
            maintenance4_device,
            maintenance5_device,
            push_descriptor_device,
            physical_device,
            physical_device_info,
            device,
            graphics_queue,
            transfer_queue,
            compute_queue,
            frame_timeout: attributes.frame_timeout.as_nanos() as u64,
            max_memory_allocation_size,
            max_push_descriptors,
            enabled_layers,
            enabled_instance_extensions,
            enabled_device_extensions: attributes.device_extensions,
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
    pub fn is_device_extension_enabled(&self, extension: DeviceExtension) -> bool {
        self.enabled_device_extensions & extension == extension
    }

    #[inline(always)]
    pub fn instance(&self) -> &nox_ash::Instance {
        &self.instance
    }

    #[inline(always)]
    pub fn device(&self) -> &nox_ash::Device {
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
    pub fn timeline_semaphore_device(&self) -> &timeline_semaphore::Device {
        &self.timeline_semaphore_device
    }

    #[inline(always)]
    pub fn dynamic_rendering_device(&self) -> &dynamic_rendering::Device {
        &self.dynamic_rendering_device
    }

    #[inline(always)]
    pub fn synchronization2_device(&self) -> &synchronization2::Device {
        &self.synchronization2_device
    }

    #[inline(always)]
    pub fn maintenance4_device(&self) -> &maintenance4::Device {
        &self.maintenance4_device
    }

    #[inline(always)]
    pub fn maintenance5_device(&self) -> &maintenance5::Device {
        &self.maintenance5_device
    }

    #[inline(always)]
    pub fn push_descriptor_device(&self) -> &push_descriptor::Device {
        &self.push_descriptor_device
    }

    #[inline(always)]
    pub fn extended_dynamic_state_device(&self) -> &extended_dynamic_state::Device {
        &self.extended_dynamic_state_device
    }

    #[inline(always)]
    pub fn max_memory_allocation_size(&self) -> vk::DeviceSize {
        self.max_memory_allocation_size
    }

    #[inline(always)]
    pub fn max_push_descriptors(&self) -> Option<u32> {
        self.max_push_descriptors
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

fn get_required_instance_extensions<W>(
    window: &W,
    out: &mut Vec32::<(&CStr, bool)>,
) -> Result<()>
    where
        W: HasDisplayHandle
{
    out.push((surface::NAME, true));
    let ext = match window.display_handle().unwrap().as_raw() {
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
