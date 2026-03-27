//! Provided by VK_KHR_push_descriptor or Vulkan 1.4.

use {
    ::core::ffi::{
        CStr,
        c_void,
    },
    nox_ash::{
        vk,
        load_fn,
        khr,
    },
    crate::Version,
    super::*,
};


pub struct Attributes;

impl Attributes {
    /// Attribute type `bool`.
    pub const IS_ENABLED: ConstName = ConstName::new("push_descriptor");
    /// Attribute type `u32`.
    pub const MAX_PUSH_DESCRIPTORS: ConstName = ConstName::new("max_push_descriptors");
}

/// The extension type.
#[derive(Clone, Copy)]
pub struct Extension;

unsafe impl DeviceExtension for Extension {

    fn get_info(&self, _: &DeviceAttributes) -> Option<DeviceExtensionInfo> {
        Some(DeviceExtensionInfo {
            name: khr::push_descriptor::NAME,
            deprecation_version: Version::VULKAN_API_VERSION_1_4,
            precondition: Precondition::new(|ctx| {
                if ctx.api_version() >= Version::VULKAN_API_VERSION_1_4 {
                    let mut features = vk::PhysicalDeviceVulkan14Features::default();
                    ctx.get_features(&mut features);
                    (features.push_descriptor == 0).then(|| MissingDeviceFeatureError::new(
                        "push descriptor"
                    ))
                } else {
                    None
                }
            }),
        })
    }

    fn register(
        &self,
        ctx: &mut PhysicalDeviceContext<'_>,
    ) -> Option<vk::ExtendsDeviceCreateInfoObj> {
        let mut properties = vk::PhysicalDevicePushDescriptorProperties::default();
        ctx.get_properties(&mut properties);
        ctx.register_attribute(DeviceAttribute::new_u32(
            Attributes::MAX_PUSH_DESCRIPTORS,
            properties.max_push_descriptors,
        ));
        ctx.register_attribute(DeviceAttribute::new_bool(
            Attributes::IS_ENABLED,
            true,
        ));
        if ctx.api_version() >= Version::VULKAN_API_VERSION_1_4 {
            ctx.vulkan_14_features().push_descriptor = vk::TRUE;
        }
        None
    }

    fn boxed(&self) -> Box<dyn DeviceExtension> {
        Box::new(*self)
    }
}


/// Raw [`push_descriptor`] device-level function pointers.
#[derive(Clone)]
pub struct DeviceFn {
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushDescriptorSet.html>
    pub cmd_push_descriptor_set: vk::PFN_vkCmdPushDescriptorSet,
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushDescriptorSet2.html>
    pub cmd_push_descriptor_set2: vk::PFN_vkCmdPushDescriptorSet2,
}

unsafe impl Send for DeviceFn {}
unsafe impl Sync for DeviceFn {}

impl DeviceFn {

    pub fn load<F: FnMut(&CStr) -> *const c_void>(
        api_version: Version,
        mut f: F,
    ) -> Self
    {
        if api_version >= Version::VULKAN_API_VERSION_1_4 {
            unsafe { Self {
                cmd_push_descriptor_set: load_fn!(
                    fn cmd_push_descriptor_set(
                        vk::CommandBuffer,
                        vk::PipelineBindPoint,
                        vk::PipelineLayout,
                        u32,
                        u32,
                        *const vk::WriteDescriptorSet,
                    ) -> (),
                    f,
                    c"vkCmdPushDescriptorSet",
                    vk::PFN_vkCmdPushDescriptorSet,
                ),
                cmd_push_descriptor_set2: load_fn!(
                    fn cmd_push_descriptor_set2(
                        vk::CommandBuffer,
                        *const vk::PushDescriptorSetInfo,
                    ) -> (),
                    f,
                    c"vkCmdPushDescriptorSet2",
                    vk::PFN_vkCmdPushDescriptorSet2,
                ),
            } }
        } else {
            unsafe { Self {
                cmd_push_descriptor_set: load_fn!(
                    fn cmd_push_descriptor_set(
                        vk::CommandBuffer,
                        vk::PipelineBindPoint,
                        vk::PipelineLayout,
                        u32,
                        u32,
                        *const vk::WriteDescriptorSet,
                    ) -> (),
                    f,
                    c"vkCmdPushDescriptorSetKHR",
                    vk::PFN_vkCmdPushDescriptorSet,
                ),
                cmd_push_descriptor_set2: load_fn!(
                    fn cmd_push_descriptor_set2(
                        vk::CommandBuffer,
                        *const vk::PushDescriptorSetInfo,
                    ) -> (),
                    f,
                    c"vkCmdPushDescriptorSet2KHR",
                    vk::PFN_vkCmdPushDescriptorSet2,
                ),
            } }
        }
    }
}

/// [`push_descriptor`] device-level functions.
#[derive(Clone)]
pub struct Device {
    fp: DeviceFn,
    handle: vk::Device,
}

impl Device {

    #[inline(always)]
    pub fn fp(&self) -> &DeviceFn {
        &self.fp
    }

    #[inline(always)]
    pub fn device(&self) -> vk::Device {
        self.handle
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushDescriptorSet.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    #[inline(always)]
    pub unsafe fn cmd_push_descriptor_set(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        set: u32,
        descriptor_writes: &[vk::WriteDescriptorSet<'_>],
    ) {
        unsafe {
            (self.fp().cmd_push_descriptor_set)(
                command_buffer,
                pipeline_bind_point,
                layout,
                set,
                descriptor_writes.len() as u32,
                descriptor_writes.as_ptr(),
            )
        }
    }

    /// <https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdPushDescriptorSet2.html>
    ///
    /// # Safety
    /// All raw Vulkan calls are inherently unsafe, because no validation of input or usage is applied.
    pub unsafe fn cmd_push_descriptor_set2(
        &self,
        command_buffer: vk::CommandBuffer,
        push_descriptor_set_info: &vk::PushDescriptorSetInfo<'_>
    ) {
        unsafe {
            (self.fp().cmd_push_descriptor_set2)(
                command_buffer,
                push_descriptor_set_info,
            )
        }
    }
}

impl AnyExtensionDevice for Device {

    #[inline(always)]
    fn boxed(&self) -> Box<dyn AnyExtensionDevice> {
        Box::new(self.clone())
    }
}

impl ExtensionDevice for Device {

    const NAME: ConstName = ConstName::new(
        "push descriptor device"
    );

    #[inline(always)]
    fn precondition<'a, F>(f: F) -> bool
        where F: Fn(&ConstName) -> Option<&'a DeviceAttribute>
    {
        f(&Attributes::IS_ENABLED)
            .is_some_and(|value| value.bool().is_some_and(|value| value))
    }

    #[inline(always)]
    fn new(device: &LogicalDevice) -> Box<Self>
    {
        let fp = DeviceFn::load(device.api_version(), |name| unsafe {
            ::core::mem::transmute(device.get_proc_addr(name))
        });
        Box::new(Self {
            fp,
            handle: device.handle(),
        })
    }
}
