use core::ffi::CStr;

use compact_str::CompactString;

use nox_mem::{
    vec::{Vector, Vec32},
    string::ArrayString,
    vec32,
    Display,
    borrow::CowMut,
};

use nox_ash::vk;

use crate::{
    expand_warn, log, 
};

use crate::dev::{
    has_bits, has_not_bits,
    prelude::Version,
    error::{Result, Error, Context, ErrorContext, location},
};

use super::DeviceName;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: QueueFamilyIndex,
    pub transfer: QueueFamilyIndex,
    pub compute:  QueueFamilyIndex,
}

impl QueueFamilyIndices {

    pub fn graphics_index(&self) -> u32 {
        self.graphics.index
    }

    pub fn transfer_index(&self) -> u32 {
        self.transfer.index
    }

    pub fn compute_index(&self) -> u32 {
        self.compute.index
    }
}

pub struct PhysicalDeviceInfo<'a> {
    features: vk::PhysicalDeviceFeatures,
    extended_features: CowMut<'a, [vk::ExtendsPhysicalDeviceFeatures2Obj]>,
    properties: vk::PhysicalDeviceProperties,
    extended_properties: CowMut<'a, [vk::ExtendsPhysicalDeviceProperties2Obj]>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_family_indices: QueueFamilyIndices,
    api_version: Version,
    device_name: ArrayString<{vk::MAX_PHYSICAL_DEVICE_NAME_SIZE}>,
}

impl<'a> PhysicalDeviceInfo<'a> {

    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        extended_features: &'a mut [vk::ExtendsPhysicalDeviceFeatures2Obj],
        extended_properties: &'a mut [vk::ExtendsPhysicalDeviceProperties2Obj],
    ) -> Result<Option<Self>>
    {
        let queue_family_indices =
            match QueueFamilyIndices::new(physical_device, instance) {
                Ok(queue_family_indices) => {
                    match queue_family_indices {
                        Some(queue_family_indices) => queue_family_indices,
                        None => return Ok(None),
                    }
                }
                Err(err) => return Err(err),
            };
        let mut features = vk::PhysicalDeviceFeatures2::default();
        for feature in extended_features.iter_mut() {
            feature.clear_p_next();
            features = features.push_next(feature.as_vk_mut());
        }
        unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
        let features = features.features;
        let mut properties = vk::PhysicalDeviceProperties2::default();
        for prop in extended_properties.iter_mut() {
            prop.clear_p_next();
            properties = properties.push_next(prop.as_vk_mut());
        }
        unsafe { instance.get_physical_device_properties2(physical_device, &mut properties) };
        let properties = properties.properties;
        let device_name = ArrayString
            ::from_c_char_slice(&properties.device_name)
            .context_with(|| ErrorContext::StringConversionError(location!()))?;
        let api_version = Version::from(properties.api_version);
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
        Ok(Some(
            Self {
                features,
                extended_features: CowMut::Borrowed(extended_features),
                properties,
                extended_properties: CowMut::Borrowed(extended_properties),
                memory_properties,
                queue_family_indices,
                api_version,
                device_name,
            }
        ))
    }

    #[inline(always)]
    pub fn api_version(&self) -> Version {
        self.api_version
    }

    #[inline(always)]
    pub fn device_name(&self) -> &DeviceName {
        &self.device_name
    }

    #[inline(always)]
    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices
    }

    #[inline(always)]
    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    #[inline(always)]
    pub fn extended_feature(&self, s_type: vk::StructureType) -> Option<&vk::ExtendsPhysicalDeviceFeatures2Obj> {
        self.extended_features.iter().find(|feature| {
            feature.s_type() == s_type
        })
    }

    #[inline(always)]
    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    #[inline(always)]
    pub fn extended_property(&self, s_type: vk::StructureType) -> Option<&vk::ExtendsPhysicalDeviceProperties2Obj> {
        self.extended_properties.iter().find(|prop| {
            prop.s_type() == s_type
        })
    }

    #[inline(always)]
    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    #[inline(always)]
    pub fn into_owned(self) -> PhysicalDeviceInfo<'static> {
        PhysicalDeviceInfo {
            features: self.features,
            extended_features: CowMut::Owned(self.extended_features.into_owned()),
            properties: self.properties,
            extended_properties: CowMut::Owned(self.extended_properties.into_owned()),
            memory_properties: self.memory_properties,
            queue_family_indices: self.queue_family_indices,
            api_version: self.api_version,
            device_name: self.device_name,
        }
    }
}

impl QueueFamilyIndices {

    pub fn new(
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> Result<Option<Self>>
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
                has_bits!(property.queue_flags, vk::QueueFlags::GRAPHICS)
            {
                graphics = (i as u32, true); 
            }
            if !transfer.1 &&
                has_bits!(property.queue_flags, vk::QueueFlags::TRANSFER) 
            {
                if has_not_bits!(property.queue_flags, vk::QueueFlags::GRAPHICS) {
                    transfer = (i as u32, true);
                    continue;
                }
                else if !transfer_secondary.1 {
                    transfer_secondary = (i as u32, true);
                }
            }
            if !compute.1 &&
                has_bits!(property.queue_flags, vk::QueueFlags::COMPUTE)
            {
                if has_not_bits!(property.queue_flags, vk::QueueFlags::GRAPHICS) {
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
                has_bits!(property.queue_flags, vk::QueueFlags::TRANSFER) {
                transfer = (i as u32, true);
            }
            if !compute.1 &&
                has_bits!(property.queue_flags, vk::QueueFlags::COMPUTE) {
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

#[derive(Display)]
enum PhysicalDeviceRating<'a> {
    Ok(i32),
    #[display("of missing features: {0}")]
    MissingFeatures(CompactString),
    #[display("{0}")]
    UnsuitableProperties(CompactString),
    #[display("of missing extensions: {0:?}")]
    MissingExtensions(Vec32<&'a CStr>),
    #[display("Nox requires at least Vulkan version 1.1 (device version was {0})")]
    OldVersion(Version),
}

pub fn rate_physical_device<'a>(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    physical_device_info: &PhysicalDeviceInfo,
    required_extensions: &[(&'a CStr, Version)],
    base_feature_check: &impl Fn(&vk::PhysicalDeviceFeatures) -> Option<CompactString>,
    extended_feature_check: &impl Fn(&vk::ExtendsPhysicalDeviceFeatures2Obj) -> Option<CompactString>,
    base_property_check: &impl Fn(&vk::PhysicalDeviceProperties) -> Option<CompactString>,
    extended_property_check: &impl Fn(&vk::ExtendsPhysicalDeviceProperties2Obj) -> Option<CompactString>,
) -> Result<PhysicalDeviceRating<'a>>
{
    if let Some(msg) = base_feature_check(physical_device_info.features()) {
        return Ok(PhysicalDeviceRating::MissingFeatures(msg))
    }
    for extended in physical_device_info.extended_features.iter() {
        if let Some(msg) = extended_feature_check(extended) {
            return Ok(PhysicalDeviceRating::MissingFeatures(msg))
        }
    }
    if let Some(msg) = base_property_check(physical_device_info.properties()) {
        return Ok(PhysicalDeviceRating::UnsuitableProperties(msg))
    }
    for extended in physical_device_info.extended_properties.iter() {
        if let Some(msg) = extended_property_check(extended) {
            return Ok(PhysicalDeviceRating::UnsuitableProperties(msg))
        }
    }
    let api_version = physical_device_info.api_version;
    if api_version < vk::API_VERSION_1_1 {
        return Ok(PhysicalDeviceRating::OldVersion(api_version))
    }
    let mut check_ext = vec32![];
    for &(name, version) in required_extensions {
        if version > api_version {
            check_ext.push(name);
        }
    }
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .context("failed to enumerate vulkan device extensions")?
    };

    let missing_extensions: Vec32<_> = check_ext.iter().filter_map(|&ext| {
        (!available_extensions.iter().any(|a| {
            a.extension_name_as_c_str().unwrap_or_default() == ext
        })).then_some(ext)
    }).collect();

    if !missing_extensions.is_empty() {
        return Ok(PhysicalDeviceRating::MissingExtensions(missing_extensions))
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
    
    Ok(PhysicalDeviceRating::Ok(score))
}

pub fn find_suitable_physical_device<'a>(
    instance: &ash::Instance,
    required_extensions: &[(&CStr, Version)],
    base_feature_check: impl Fn(&vk::PhysicalDeviceFeatures) -> Option<CompactString>,
    extended_features: &mut [vk::ExtendsPhysicalDeviceFeatures2Obj],
    extended_feature_check: impl Fn(&vk::ExtendsPhysicalDeviceFeatures2Obj) -> Option<CompactString>,
    base_property_check: impl Fn(&vk::PhysicalDeviceProperties) -> Option<CompactString>,
    extended_properties: &mut [vk::ExtendsPhysicalDeviceProperties2Obj],
    extended_property_check: impl Fn(&vk::ExtendsPhysicalDeviceProperties2Obj) -> Option<CompactString>,
) -> Result<(vk::PhysicalDevice, PhysicalDeviceInfo<'static>)>
{
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .context("failed to enumerate vulkan devices")?
    };
    let mut best_physical_device : Option<(vk::PhysicalDevice, PhysicalDeviceInfo)> = None;
    let mut unsuitable = vec32![];
    let mut best_score = -1i32;
    for physical_device in physical_devices {
        let physical_device_info =
            match PhysicalDeviceInfo
                ::new(instance, physical_device, extended_features, extended_properties)
                .context("failed to get vulkan physical device info")
            {
                Ok(device_info) => {
                    match device_info {
                        Some(device_info) => device_info,
                        None => continue,
                    }
                }
                Err(err) => {
                    expand_warn!(err);
                    continue;
                },
            };
        let rating =
            match rate_physical_device(
                instance,
                physical_device,
                &physical_device_info,
                required_extensions,
                &base_feature_check,
                &extended_feature_check,
                &base_property_check,
                &extended_property_check,
            )
                .context("failed to rate vulkan physical device")
            {
                Ok(score) => score,
                Err(err) => {
                    expand_warn!(err);
                    continue;
                },
            };
        match rating {
            PhysicalDeviceRating::Ok(score) => {
                if score > best_score {
                    best_physical_device = Some((physical_device, physical_device_info.into_owned()));
                    best_score = score;
                }
            },
            _ => unsuitable.push((rating, physical_device_info.device_name)),
        };
    }
    if !unsuitable.is_empty() {
        for (reason, name) in unsuitable {
            log::warn!("GPU {} was unsuitable because {}", name, reason);
        }
    }
    best_physical_device
        .ok_or_else(||
            Error::just_context("failed to find suitable vulkan physical device")
        )
}
