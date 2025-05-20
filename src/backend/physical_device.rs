use super::SmallError;

use crate::{
    string::String,
    version::Version,
    utility::{has_bit, has_not_bit},
};

use ash::{ext::graphics_pipeline_library, khr::{self, surface}, vk};
use arrayvec::ArrayVec;

pub struct QueueFamilyIndex {
    pub index: u32,
    pub is_unique: bool,
}

impl QueueFamilyIndex {

    pub fn new(index: u32, is_unique: bool) -> Self {
        Self {
            index,
            is_unique,
        }
    }
}

pub struct QueueFamilyIndices {
    pub graphics: QueueFamilyIndex,
    pub transfer: QueueFamilyIndex,
    pub compute:  QueueFamilyIndex,
}

impl QueueFamilyIndices {

    pub fn get_graphics_index(&self) -> u32 {
        self.graphics.index
    }

    pub fn get_transfer_index(&self) -> u32 {
        self.transfer.index
    }

    pub fn get_compute_index(&self) -> u32 {
        self.compute.index
    }
}

pub struct PhysicalDeviceInfo {
    pub api_version: Version,
    pub device_name: String<{vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>,
    pub queue_family_indices: QueueFamilyIndices,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
}

impl PhysicalDeviceInfo {

    pub fn new(
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface_loader: &surface::Instance,
        surface_khr: vk::SurfaceKHR,
    ) -> Result<Option<Self>, SmallError>
    {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_name =
            match String::from_ascii(properties.device_name) {
                Ok(device_name) => device_name,
                Err(_) => return Err(String::from_str("failed to convert device name to string")),
            };
        let api_version = Version::from(properties.api_version);
        let queue_family_indices =
            match QueueFamilyIndices::new(physical_device, instance, surface_loader, surface_khr) {
                Ok(queue_family_indices) => {
                    match queue_family_indices {
                        Some(queue_family_indices) => queue_family_indices,
                        None => return Ok(None),
                    }
                }
                Err(err) => return Err(err),
            };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        Ok(
            Some(
                Self {
                    api_version,
                    device_name,
                    queue_family_indices,
                    properties,
                    features,
                }
            )
        )
    }

    pub fn get_queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_family_indices
    }
}

impl QueueFamilyIndices {

    pub fn new(
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface_loader: &surface::Instance,
        surface_khr: vk::SurfaceKHR
    ) -> Result<Option<Self>, SmallError>
    {
        let properties = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics: (u32, bool) = (0, false);
        let mut transfer: (u32, bool) = (0, false);
        let mut compute:  (u32, bool) = (0, false);

        let mut transfer_secondary: (u32, bool) = (0, false);
        let mut compute_secondary: (u32, bool) = (0, false);

        for (i, property) in properties.iter().enumerate() {
            if graphics.1 && transfer.1 && compute.1 {
                break;
            }
            if !graphics.1 &&
                has_bit!(property.queue_flags, vk::QueueFlags::GRAPHICS)
            {
                let present_supported = unsafe {
                    match surface_loader.get_physical_device_surface_support(physical_device, i as u32, surface_khr) {
                        Ok(present_supported) => present_supported,
                        Err(result) =>  {
                            return Err(
                                String::format(format_args!("failed to get physical device surface support {:?}", result))
                            )
                        },
                    }
                };
                if present_supported {
                    graphics = (i as u32, true);
                    continue;
                }
            }
            if !transfer.1 &&
                has_bit!(property.queue_flags, vk::QueueFlags::TRANSFER) 
            {
                if has_not_bit!(property.queue_flags, vk::QueueFlags::GRAPHICS) {
                    transfer = (i as u32, true);
                    continue;
                }
                else if !transfer_secondary.1 {
                    transfer_secondary = (i as u32, true);
                }
            }
            if !compute.1 &&
                has_bit!(property.queue_flags, vk::QueueFlags::COMPUTE)
            {
                if has_not_bit!(property.queue_flags, vk::QueueFlags::GRAPHICS) {
                    compute = (i as u32, true);
                }
                else if !compute_secondary.1 {
                    compute_secondary = (i as u32, true);
                }
            }
        }
        if !graphics.1 {
            return Ok(None)
        }
        // check preferred queues
        if transfer.1 && compute.1 {
            return Ok(
                Some(
                    Self {
                        graphics: QueueFamilyIndex::new(graphics.0, true),
                        transfer: QueueFamilyIndex::new(transfer.0, true),
                        compute:  QueueFamilyIndex::new(compute.0,  true),
                    }
                )
            )
        }
        // fall back to accepting secondary (possibly shared queues)
        if !transfer.1 && transfer_secondary.1 {
            transfer = transfer_secondary;
        }
        if !compute.1 && compute_secondary.1 {
            compute = compute_secondary;
        }
        if transfer.1 && compute.1 {
            let (q, t, c) = (graphics.0, transfer.0, compute.0);
            return Ok(
                Some(
                    Self {
                        graphics: QueueFamilyIndex::new(q, true), // graphics should always be unique here
                        transfer: QueueFamilyIndex::new(t, t != c),
                        compute:  QueueFamilyIndex::new(c, c != t),
                    }
                )
            )
        }
        // fall back to accepting shared queues
        for (i, property) in properties.iter().enumerate() {
            if !transfer.1 &&
                has_bit!(property.queue_flags, vk::QueueFlags::TRANSFER) {
                transfer = (i as u32, true);
            }
            if !compute.1 &&
                has_bit!(property.queue_flags, vk::QueueFlags::COMPUTE) {
                compute  = (i as u32, true);
            }
            if transfer.1 && compute.1 {
                break;
            }
        }
        if transfer.1 && compute.1 {
            let (q, t, c) = (graphics.0, transfer.0, compute.0);
            return Ok(
                Some(
                    Self {
                        graphics: QueueFamilyIndex::new(q, q != t && q != c),
                        transfer: QueueFamilyIndex::new(t, t != q && t != c),
                        compute:  QueueFamilyIndex::new(c, c != q && c != t),
                    }
                )
            )
        }
        Ok(None)
    }
}

pub fn rate_physical_device(
    physical_device: vk::PhysicalDevice,
    physical_device_info: &PhysicalDeviceInfo,
    instance: &ash::Instance
) -> Result<i32, SmallError>
{
    if physical_device_info.features.sample_rate_shading == vk::FALSE ||
        physical_device_info.features.sampler_anisotropy == vk::FALSE {
        return Ok(-1)
    }
    let mut required_extensions = ArrayVec::<String::<{vk::MAX_EXTENSION_NAME_SIZE}>, 3>::new();
    required_extensions.push(
        String::from_str(
            match khr::swapchain::NAME.to_str() {
                Ok(s) => s,
                Err(_) => return Err(SmallError::from_str("failed to convert extension name to str"))
            }
        )
    );
    if physical_device_info.api_version.as_u32() < vk::API_VERSION_1_2 {
        required_extensions.push(
            String::from_str(
                match khr::dynamic_rendering::NAME.to_str() {
                    Ok(s) => s,
                    Err(_) => return Err(SmallError::from_str("failed to convert extension name to str")),
                }
            )
        );
        required_extensions.push(
            String::from_str(
                match khr::timeline_semaphore::NAME.to_str() {
                    Ok(s) => s,
                    Err(_) => return Err(SmallError::from_str("failed to convert extension name to str")),
                }
            )
        );
    }
    else if physical_device_info.api_version.as_u32() < vk::API_VERSION_1_3 {
        required_extensions.push(
            String::from_str(
                match khr::dynamic_rendering::NAME.to_str() {
                    Ok(s) => s,
                    Err(_) => return Err(SmallError::from_str("failed to convert extension name to str")),
                }
            )
        );
    }
    let available_extensions = unsafe {
        match instance.enumerate_device_extension_properties(physical_device) {
            Ok(available_extension) => available_extension,
            Err(result) => return Err(SmallError::format(format_args!("failed to enumerate device extensions: {:?}", result)))
        }
    };
    let mut found_extensions = false;
    for extension in &required_extensions {
        found_extensions = false;
        for available_extension in &available_extensions {
            let other = String::<{vk::MAX_EXTENSION_NAME_SIZE}>::from_ascii(available_extension.extension_name)?;
            if extension == &other {
                found_extensions = true;
                break;
            }
        }
        if !found_extensions {
            break;
        }
    }
    if !found_extensions {
        return Ok(-1)
    }

    let mut score = 0i32;

    if physical_device_info.properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 1000;
    }
    else if physical_device_info.properties.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU {
        score += 500;
    }
    else {
        score += 100;
    }

    if physical_device_info.features.fill_mode_non_solid == vk::TRUE {
        score += 100;
    }

    if physical_device_info.queue_family_indices.graphics.is_unique { score += 100 };
    if physical_device_info.queue_family_indices.transfer.is_unique { score += 100 };
    if physical_device_info.queue_family_indices.compute.is_unique { score += 100 };

    score += (physical_device_info.properties.limits.sampled_image_color_sample_counts.as_raw() << 1) as i32;
    score += physical_device_info.properties.limits.sampled_image_depth_sample_counts.as_raw() as i32;

    score += (physical_device_info.api_version.major() * 50) as i32;
    score += (physical_device_info.api_version.minor() * 10) as i32;
    
    Ok(score)
}

pub fn find_suitable_physical_device(
    instance: &ash::Instance,
    surface_loader: &surface::Instance,
    surface_khr: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, PhysicalDeviceInfo), SmallError>
{
    let physical_devices = unsafe {
        match instance.enumerate_physical_devices() {
            Ok(physical_devices) => physical_devices,
            Err(result) => {
                return Err(String::format(format_args!("failed to enumerate physical devices {:?}", result)));
            },
        }
    };
    let mut best_physical_device : Option<(vk::PhysicalDevice, PhysicalDeviceInfo)> = None;
    let mut best_score = -1i32;
    for physical_device in physical_devices {
        let physical_device_info =
            match PhysicalDeviceInfo::new(physical_device, instance, surface_loader, surface_khr) {
                Ok(device_info) => {
                    match device_info {
                        Some(device_info) => device_info,
                        None => continue,
                    }
                }
                Err(err) => {
                    println!("Nox backend error: {}", err);
                    continue;
                },
            };
        let score =
            match rate_physical_device(physical_device, &physical_device_info, instance) {
                Ok(score) => score,
                Err(err) => {
                    println!("Nox backend error: {}", err);
                    continue;
                },
            };
        if score > best_score {
            best_physical_device = Some((physical_device, physical_device_info));
            best_score = score;
        }
    }
    match best_physical_device {
        Some(physical_device) => Ok(physical_device),
        None => Err(SmallError::from_str("failed to find suitable physical device")),
    }
}
