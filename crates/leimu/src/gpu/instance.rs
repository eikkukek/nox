use std::ffi::{
    CStr, CString,
};

use raw_window_handle::{
    HasDisplayHandle,
    RawDisplayHandle,
};

use ahash::AHashSet;

use nox_mem::{
    vec::Vec32,
    Display,
    vec32,
};
use nox_ash::{
    vk,
    khr::{
        surface,
        wayland_surface, win32_surface,
        xlib_surface, xcb_surface,
        android_surface,
        get_surface_capabilities2,
    },
};

use crate::{
    gpu::prelude::*,
    error::*,
    log::{info, warn},
    sync::*,
};

/// Khronos validation layer.
///
/// For this to be used, the Vulkan SDK needs to be installed.
pub const LAYER_KHRONOS_VALIDATION: &CStr = c"VK_LAYER_KHRONOS_validation";

#[derive(Clone, Copy, Display)]
#[display("{name:?}")]
pub struct InstanceLayer<'a> {
    name: &'a CStr,
    is_required: bool,
}

impl<'a> InstanceLayer<'a> {

    #[inline(always)]
    pub fn new(
        name: &'a CStr,
        is_required: bool,
    ) -> Self {
        Self {
            name,
            is_required,
        }
    }

    #[inline(always)]
    pub fn khronos_validation(
        is_required: bool,
    ) -> Self {
        Self {
            name: LAYER_KHRONOS_VALIDATION,
            is_required,
        }
    }
}

struct Inner {
    entry: nox_ash::Entry,
    instance: nox_ash::Instance,
    get_surface_capabilities2: get_surface_capabilities2::Instance,
    surface: surface::Instance,
}

/// Represents a [`Vulkan instance`][1].
///
/// This is used for creating [`logical devices`][2], which are needed to create [`Gpu`].
///
/// [1]: https://docs.vulkan.org/refpages/latest/refpages/source/VkInstance.html
/// [2]: LogicalDevice
#[derive(Clone)]
pub struct Instance {
    inner: Arc<Inner>,
}

impl Instance {

    pub fn new<H>(
        platform: &H,
        app_name: &str, 
        app_version: Version,
        layers: &[InstanceLayer<'_>],
    ) -> Result<Self>
        where H: HasDisplayHandle
    {
        let entry = unsafe { nox_ash::Entry::load().context("failed to create vulkan entry")? };
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
            ::new(app_name
                .chars()
                .filter(|&c| c != '\0')
                .collect::<String>()
            ).unwrap();
        let engine_name = CString::new("nox").unwrap();
        let application_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_application_name: app_name.as_ptr(),
            application_version: app_version.as_u32(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_4,
            ..Default::default()
        };
        let mut extensions = Vec32::<(&CStr, bool)>
            ::with_capacity(8);
        let mut found_extensions = Vec32::<*const i8>
            ::with_capacity(8);
        let mut found_extensions_hashed = AHashSet::default();
        get_required_instance_extensions(platform, &mut extensions)?;
        let mut found_layers = Vec32::<*const i8>
            ::with_capacity(8);
        let mut found_layers_hashed = AHashSet::default();
        verify_instance_extensions(
            &entry,
            &extensions,
            &mut found_extensions,
            &mut found_extensions_hashed
        )?;
        verify_instance_layers(
            &entry,
            layers,
            &mut found_layers,
            &mut found_layers_hashed,
        )?;
        let version = unsafe {
            let Some(version) = entry
                .try_enumerate_instance_version()
                .context("failed to enumerate Vulkan instance version")?
            else {
                return Err(Error::just_context(
                    "Nox requires at least Vulkan version 1.1, enumerated version was 1.0"
                ))
            };
            version
        };
        if version < vk::API_VERSION_1_1 {
            return Err(Error::just_context(format!(
                "Nox requires at least Vulkan version 1.1, enumerated version was {}",
                Version(version),
            )))
        }
        let instance_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &application_info,
            enabled_extension_count: found_extensions.len(),
            pp_enabled_extension_names: found_extensions.as_ptr() as _,
            enabled_layer_count: found_layers.len(),
            pp_enabled_layer_names: found_layers.as_ptr() as _,
            ..Default::default()
        };
        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .context("failed to create vulkan instance")?
        };
        let get_surface_capabilities2 = get_surface_capabilities2::Instance
            ::new(&entry, &instance);
        let surface = surface::Instance
            ::new(&entry, &instance);
        Ok(Self {
            inner: Arc::new(Inner {
                entry,
                instance,
                get_surface_capabilities2,
                surface,
            }),
        })
    }

    /// Enumerates all [`physical devices`][1] that are suitable for the given [`attributes`][2].
    ///
    /// After this, you can pick a device you want and [`create a logical device`][3].
    ///
    /// [1]: PhysicalDevice
    /// [2]: DeviceAttributes
    /// [3]: SuitablePhysicalDevices::create_logical_device
    #[inline(always)]
    pub fn enumerate_suitable_physical_devices(
        &self,
        device_attributes: DeviceAttributes,
    ) -> Result<SuitablePhysicalDevices> {
        let mut device_extensions = Vec32::with_capacity(device_attributes.device_extensions.len());
        device_extensions.extend(ext::core_extensions());
        device_extensions.extend(device_attributes.device_extensions.iter().cloned());
        let mut device_extension_infos = vec32![];
        device_extension_infos.extend(device_extensions
            .iter().filter_map(|ext| ext.get_info(&device_attributes))
        );
        let devices = find_suitable_physical_devices(
            self,
            &device_attributes,
            &device_extension_infos,
        )?;
        Ok(SuitablePhysicalDevices {
            instance: self.clone(),
            devices,
            attributes: device_attributes,
            device_extensions,
            device_extension_infos,
        })
    }

    #[inline(always)]
    pub fn entry(&self) -> &nox_ash::Entry {
        &self.inner.entry
    }

    #[inline(always)]
    pub fn ash(&self) -> &nox_ash::Instance {
        &self.inner.instance
    }

    #[inline(always)]
    pub fn surface_instance(&self) -> &surface::Instance {
        &self.inner.surface
    }

    #[inline(always)]
    pub fn get_surface_capabilities2_instance(&self) -> &get_surface_capabilities2::Instance {
        &self.inner.get_surface_capabilities2
    }
}

/// A structure returned by [`enumerate suitable physical devices`][1].
///
/// [1]: Instance::enumerate_suitable_physical_devices
pub struct SuitablePhysicalDevices {
    pub(super) instance: Instance,
    pub(super) devices: Vec32<PhysicalDevice>,
    pub(super) attributes: DeviceAttributes,
    pub(super) device_extensions: Vec32<ext::DeviceExtensionObj>,
    pub(super) device_extension_infos: Vec32<ext::DeviceExtensionInfo>,
}

impl SuitablePhysicalDevices {

    /// Returns an iterator over all suitable physical devices.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (u32, &PhysicalDevice)> {
        self.devices
        .iter().enumerate()
        .map(|(idx, d)| (idx as u32, d))
    }

    #[inline(always)]
    pub fn get(&self, index: u32) -> &PhysicalDevice {
        &self.devices[index as usize]
    }

    /// Creates a [`logical device`][1] needed for creating [`Gpu`].
    ///
    /// # Parameters
    /// - `device_idx`: the index a physical device, that originated from [`Self::iter`].
    /// - `queue_create_infos`: specifies which which queues to create
    ///
    /// # Valid usage
    /// - `device_idx` *must* be a valid index into the physical devices in this structure.
    /// - `queue_create_infos` *must* not be empty.
    /// - Each [`create info`][2] in `queue_create_infos` *must* have a valid queue family index for
    ///   the specified device and the queue index *must* be less than the queue count of that queue
    ///   family.
    ///
    /// [1]: LogicalDevice
    /// [2]: DeviceQueueCreateInfo
    #[inline(always)]
    pub fn create_logical_device(
        &self,
        device_idx: u32,
        queue_create_infos: &[DeviceQueueCreateInfo],
    ) -> Result<LogicalDevice> {
        LogicalDevice::new(self, device_idx, queue_create_infos)
    }
}

impl Drop for Inner {

    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

fn get_required_instance_extensions<H>(
    handle: &H,
    out: &mut Vec32::<(&CStr, bool)>,
) -> Result<()>
    where H: HasDisplayHandle
{
    out.push((surface::NAME, true));
    let ext = match handle.display_handle().unwrap().as_raw() {
        RawDisplayHandle::Wayland(_) => wayland_surface::NAME,
        RawDisplayHandle::Windows(_) => win32_surface::NAME,
        RawDisplayHandle::Xlib(_) => xlib_surface::NAME,
        RawDisplayHandle::Xcb(_) => xcb_surface::NAME,
        RawDisplayHandle::Android(_) => android_surface::NAME,
        _ => return Err(Error::just_context(
            "unsupported platform"
        )),
        
    };
    out.push((ext, true));
    out.push((get_surface_capabilities2::NAME, true));
    Ok(())
}

fn verify_instance_layers<'a>(
    entry: &nox_ash::Entry,
    layers: &[InstanceLayer<'a>],
    found: &mut Vec32<*const i8>,
    found_hash: &mut AHashSet<&'a CStr>,
) -> Result<()>
{
    let available = unsafe { entry
        .enumerate_instance_layer_properties()
        .context("failed to enumerate instance layers")?
    };
    for layer in layers {
        if !available
            .iter()
            .any(|a| {
                layer.name == a.layer_name_as_c_str().unwrap_or_default()
            })
        {
            if layer.is_required {
                return Err(Error::just_context(format!("instance layer {layer} not present")))
            } else {
                warn!("optional instance layer {layer} not present");
            }
        } else {
            found.push(layer.name.as_ptr());
            found_hash.insert(layer.name);
        }
    }
    Ok(())
}

fn verify_instance_extensions<'a>(
    entry: &nox_ash::Entry,
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
                return Err(Error::just_context(format!(
                    "instance extension {extension:?} not present"
                )))
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
