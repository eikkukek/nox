//! Vulkan extensions.

mod core;
pub mod push_descriptor;
pub mod index_type_uint8;
pub mod robust_image_access;
pub mod robustness2;
pub mod pipeline_robustness;

pub(crate) use core::core_extensions;

use {
    ::core::{
        ffi::CStr,
        hash::{self, Hash},
        borrow::Borrow,
        ops::Deref,
        any::Any,
        mem,
        ptr::NonNull,
    },
    nox_mem::{
        Display,
        option::OptionExt,
        collections::EntryExt,
    },
    ahash::{AHashSet, AHashMap},
    compact_str::CompactString,
    nox_ash::{
        vk::{
            self,
            ExtendsStructureExt,
        },
    },
    compact_str::format_compact,
    crate::{
        Version,
        gpu::{Vulkan, PhysicalDeviceInfo, GpuAttributes},
        log,
    },
};

#[derive(Clone, Copy)]
pub enum RobustAccessRequirements {
    NotRequired,
    Required,
    Enabled,
}

impl RobustAccessRequirements {

    #[inline(always)]
    pub fn is_required(self) -> bool {
        matches!(self, Self::Required | Self::Enabled)
    }
}

/// Utility for creating [`vk::ExtendsDeviceCreateInfoObj`].
#[inline(always)]
pub fn create_extends_device_create_info_obj<T>(
    value: T
) -> vk::ExtendsDeviceCreateInfoObj
    where T: ExtendsStructureExt<
        dyn vk::ExtendsDeviceCreateInfo,
        ExtendsObj = vk::ExtendsDeviceCreateInfoObj
    >
{
    value.into_obj()
}

#[derive(Display)] #[display("{missing}")]
pub struct MissingDeviceFeatureError {
    missing: CompactString,
}

impl MissingDeviceFeatureError {

    #[inline(always)]
    pub fn new<S>(missing_features: S) -> Self
        where S: AsRef<str>
    {
        Self {
            missing: CompactString::new(missing_features),
        }
    }
}

#[derive(Clone, Copy, Display)] #[display("{name}")]
pub struct ConstName {
    name: &'static str,
    hash: u64,
}

impl ConstName {

    /// Uses 64-bit FNV-1 hash.
    pub const fn new(name: &'static str) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        let len = name.len();
        let bytes = name.as_bytes();
        let mut i = 0;
        while i < len {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(0x00000100000001b3u64);
            i += 1;
        }
        Self {
            name,
            hash,
        }
    }
}

impl Hash for ConstName {

    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for ConstName {

    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ConstName {}

enum AttributeType {
    None,
    Bool(bool),
    U32(u32),
    I32(i32),
    DeviceSize(vk::DeviceSize),
}

pub struct DeviceAttribute {
    name: ConstName,
    ty: AttributeType,
}

impl DeviceAttribute {

    #[inline(always)]
    const fn empty() -> Self {
        Self {
            name: ConstName::new(""),
            ty: AttributeType::None,
        }
    }

    #[inline(always)]
    pub fn new_bool(name: ConstName, value: bool) -> Self {
        Self {
            name: name.into(),
            ty: AttributeType::Bool(value),
        }
    }

    #[inline(always)]
    pub fn new_u32(name: ConstName, value: u32) -> Self {
        Self {
            name: name.into(),
            ty: AttributeType::U32(value),
        }
    }

    #[inline(always)]
    pub fn new_i32(name: ConstName, value: i32) -> Self {
        Self {
            name: name.into(),
            ty: AttributeType::I32(value),
        }
    }

    #[inline(always)]
    pub fn new_device_size(name: ConstName, value: vk::DeviceSize) -> Self {
        Self {
            name: name.into(),
            ty: AttributeType::DeviceSize(value),
        }
    }

    #[inline(always)]
    pub fn bool(&self) -> Option<bool> {
        match self.ty {
            AttributeType::Bool(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn u32(&self) -> Option<u32> {
        match self.ty {
            AttributeType::U32(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn device_size(&self) -> Option<vk::DeviceSize> {
        match self.ty {
            AttributeType::DeviceSize(value) => Some(value),
            _ => None,
        }
    }
}

impl Default for &DeviceAttribute {

    #[inline(always)]
    fn default() -> Self {
        const DEFAULT: DeviceAttribute = DeviceAttribute::empty();
        &DEFAULT
    }
}

impl PartialEq for DeviceAttribute {
    
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DeviceAttribute {}

impl Hash for DeviceAttribute {

    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Borrow<ConstName> for DeviceAttribute {
    
    #[inline(always)]
    fn borrow(&self) -> &ConstName {
        &self.name
    }
}

pub unsafe trait AnyExtensionDevice: Any + Send + Sync {

    fn boxed(&self) -> Box<dyn AnyExtensionDevice>;
}

pub struct ExtensionDeviceObj(Box<dyn AnyExtensionDevice>);

impl Deref for ExtensionDeviceObj {

    type Target = dyn Any;
    
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Clone for ExtensionDeviceObj {

    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.boxed())
    }
}

pub unsafe trait ExtensionDevice: AnyExtensionDevice + Clone {

    /// The name hash for the extension.
    const NAME: ConstName;

    /// Precondition for using the device.
    ///
    /// Should return `true` when the precondition is met.
    fn precondition<F>(f: F) -> bool
        where F: Fn(&ConstName) -> Option<&DeviceAttribute>;

    /// Creates a new Device.
    fn new(
        instance: &nox_ash::Instance,
        device: &nox_ash::Device,
        device_info: &PhysicalDeviceInfo,
    ) -> Box<dyn AnyExtensionDevice>;
}

#[derive(Default)]
pub struct EnabledDeviceExtensions {
    attributes: AHashSet<DeviceAttribute>,
    extension_devices: AHashMap<ConstName, ExtensionDeviceObj>
}

impl EnabledDeviceExtensions {

    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
    
    #[inline(always)]
    pub fn add_attribute(&mut self, property: DeviceAttribute) {
        self.attributes.insert(property);
    }

    #[inline(always)]
    pub fn get_attribute(&self, name: ConstName) -> &DeviceAttribute {
        self.attributes
            .get(&name)
            .unwrap_or_default()
    }

    #[inline(always)]
    pub(crate) fn get_device<T: ExtensionDevice>(
        &mut self,
        vk: &Vulkan,
    ) -> Option<T> {
        let obj = self.extension_devices.entry(T::NAME)
            .or_try_insert_with(|| {
                if !T::precondition(|name| {
                    self.attributes.get(name)
                }) {
                    return Err(format_compact!(
                        "precondition for {} not met", T::NAME.name
                    ))
                }
                Ok(ExtensionDeviceObj(T::new(
                    vk.instance(),
                    vk.device().ash_device(),
                    vk.physical_device_info(),
                )))
            }).inspect_err(|err| {
                log::error!("{}", err);
            }).ok()?;
        obj.is::<T>().then(|| unsafe {
            mem::transmute::<
                &dyn AnyExtensionDevice,
                (NonNull<T>, *const ())
            >(obj.0.deref()).0
            .as_ref().clone()
        })
    }
}

pub struct PhysicalDeviceContext<'a> {
    instance: &'a nox_ash::Instance,
    vulkan_12_features: &'a mut Option<vk::PhysicalDeviceVulkan12Features<'static>>,
    vulkan_14_features: &'a mut Option<vk::PhysicalDeviceVulkan14Features<'static>>,
    enabled_extensions: Option<&'a mut EnabledDeviceExtensions>,
    physical_device: vk::PhysicalDevice,
    physical_device_api_version: Version,
}

impl<'a> PhysicalDeviceContext<'a> {

    #[inline(always)]
    pub unsafe fn new(
        instance: &'a nox_ash::Instance,
        vulkan_12_features: &'a mut Option<vk::PhysicalDeviceVulkan12Features<'static>>,
        vulkan_14_features: &'a mut Option<vk::PhysicalDeviceVulkan14Features<'static>>,
        enabled_extensions: Option<&'a mut EnabledDeviceExtensions>,
        physical_device: vk::PhysicalDevice,
        physical_device_api_version: Version,
    ) -> Self {
        Self {
            instance,
            vulkan_12_features,
            vulkan_14_features,
            enabled_extensions,
            physical_device,
            physical_device_api_version,
        }
    }

    #[inline(always)]
    pub fn api_version(&self) -> Version {
        self.physical_device_api_version
    }

    #[inline(always)]
    pub fn get_features<T>(
        &self,
        out: &mut T
    ) where T: nox_ash::vk::ExtendsPhysicalDeviceFeatures2,
    {
        let mut features = vk::PhysicalDeviceFeatures2
            ::default()
            .push_next(out);
        unsafe {
            self.instance.get_physical_device_features2(
                self.physical_device, &mut features,
            );
        }
    }

    #[inline(always)]
    pub fn get_properties<T>(
        &self,
        out: &mut T,
    ) where T: nox_ash::vk::ExtendsPhysicalDeviceProperties2,
    {
        let mut properties = vk::PhysicalDeviceProperties2
            ::default()
            .push_next(out);
        unsafe {
            self.instance.get_physical_device_properties2(
                self.physical_device,
                &mut properties
            );
        }
    }

    #[inline(always)]
    pub fn vulkan_12_features(&mut self) -> &mut vk::PhysicalDeviceVulkan12Features<'static> {
        self.vulkan_12_features.get_or_insert_default()
    }

    #[inline(always)]
    pub fn vulkan_14_features(&mut self) -> &mut vk::PhysicalDeviceVulkan14Features<'static> {
        self.vulkan_14_features.get_or_insert_default()
    }

    #[inline(always)]
    pub fn register_attribute(&mut self, attribute: DeviceAttribute) {
        self.enabled_extensions.edit(|extensions| {
            extensions.add_attribute(attribute);
        });
    }
}

pub struct Precondition(Box<dyn Fn(&PhysicalDeviceContext<'_>) -> Option<MissingDeviceFeatureError>>);

impl Precondition {

    #[inline(always)]
    pub fn new<F>(f: F) -> Option<Self>
        where F: Fn(&PhysicalDeviceContext<'_>) -> Option<MissingDeviceFeatureError> + 'static
    {
        Some(Self(Box::new(f)))
    }

    #[inline(always)]
    pub fn call(&self, context: &PhysicalDeviceContext<'_>) -> Option<MissingDeviceFeatureError> {
        (self.0)(context)
    }
}

#[derive(Default)]
pub struct DeviceExtensionInfo {
    pub name: &'static CStr,
    pub deprecation_version: Version,
    pub precondition: Option<Precondition>,
}

/// # Safety
/// You should only implement this trait if you know what you are doing.
pub unsafe trait DeviceExtension: 'static + Send + Sync {

    /// Conditionally gets info about the extension.
    fn get_info(&self, attributes: &GpuAttributes) -> Option<DeviceExtensionInfo>;

    /// Registers the extension and optionally returns a structure extending
    /// [`vk::DeviceCreateInfo`].
    fn register(
        &self,
        context: &mut PhysicalDeviceContext<'_>,
    ) -> Option<vk::ExtendsDeviceCreateInfoObj>;

    /// Clones self to a box.
    fn boxed(&self) -> Box<dyn DeviceExtension>;
}

pub struct DeviceExtensionObj(Box<dyn DeviceExtension>);

impl From<Box<dyn DeviceExtension>> for DeviceExtensionObj {

    #[inline(always)]
    fn from(value: Box<dyn DeviceExtension>) -> Self {
        Self(value)
    }
}

impl Clone for DeviceExtensionObj {

    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.boxed())
    }
}

impl Deref for DeviceExtensionObj {

    type Target = dyn DeviceExtension;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
