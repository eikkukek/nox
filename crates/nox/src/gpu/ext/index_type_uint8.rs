use super::*;

use nox_ash::khr;

/// Attribute type `bool`.
pub const IS_ENABLED_ATTRIBUTE_NAME: ConstName = ConstName::new("index_type_uint8");

/// The extension type.
#[derive(Clone, Copy)]
pub struct DeviceExtensionIndexTypeUint8;

unsafe impl DeviceExtension for DeviceExtensionIndexTypeUint8 {

    fn get_info(&self, _: &GpuAttributes) -> Option<DeviceExtensionInfo> {
        Some(DeviceExtensionInfo {
            name: khr::index_type_uint8::NAME,
            deprecation_version: Version::VULKAN_API_VERSION_1_4,
            precondition: Precondition::new(|context| {
                let mut features = vk::PhysicalDeviceIndexTypeUint8Features::default();
                context.get_features(&mut features);
                (features.index_type_uint8 == 0).then(|| MissingDeviceFeatureError::new(
                    "index type uint8"
                ))
            }),
        })
    }

    fn register(
        &self,
        context: &mut PhysicalDeviceContext<'_>,
    ) -> Option<vk::ExtendsDeviceCreateInfoObj> {
        context.register_attribute(DeviceAttribute::new_bool(IS_ENABLED_ATTRIBUTE_NAME, true));
        Some(vk::PhysicalDeviceIndexTypeUint8Features
            ::default()
            .index_type_uint8(true)
            .into_obj()
        )
    }

    fn boxed(&self) -> Box<dyn DeviceExtension> {
        Box::new(*self)
    }
}
