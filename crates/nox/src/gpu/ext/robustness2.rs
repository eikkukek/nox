//! Provided by VK_KHR_robustness2.

use super::*;

use nox_ash::ext;

/// Property type `bool`.
pub const IS_ROBUST_BUFFER_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("robust_buffer_access2 supported");
/// Property type `bool`.
pub const IS_ROBUST_BUFFER_ACCESS_2_ENABLED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("robust_buffer_access2 enabled");
/// Property type `bool`.
pub const IS_ROBUST_IMAGE_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("robust_image_access2 supported");
/// Property type `bool`.
pub const IS_ROBUST_IMAGE_ACCESS_2_ENABLED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("robust_image_access2 enabled");
/// Property type `bool`.
pub const IS_ROBUST_NULL_DESCRIPTOR_ENABLED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("null_descriptor");

#[derive(Clone, Copy)]
pub struct Extension {
    pub robust_buffer_access2: RobustAccessRequirements,
    pub robust_image_access2: RobustAccessRequirements,
    pub enable_null_descriptor: bool,
}

unsafe impl DeviceExtension for Extension {

    fn get_info(&self, _: &GpuAttributes) -> Option<DeviceExtensionInfo> {
        let s = *self;
        Some(DeviceExtensionInfo {
            name: c"VK_KHR_robustness2",
            deprecation_version: Version::MAX,
            precondition: Precondition::new(move |context| {
                let mut features = vk::PhysicalDeviceRobustness2FeaturesEXT::default();
                context.get_features(&mut features);
                if s.robust_buffer_access2.is_required() && features.robust_buffer_access2 == 0 {
                    Some(MissingDeviceFeatureError::new("robust buffer access2"))
                } else if s.robust_image_access2.is_required() && features.robust_image_access2 == 0 {
                    Some(MissingDeviceFeatureError::new("robust image access2"))
                } else if s.enable_null_descriptor && features.null_descriptor == 0 {
                    Some(MissingDeviceFeatureError::new("null descriptor"))
                } else {
                    None
                }
            }),
        })
    }

    fn register(
        &self,
        context: &mut PhysicalDeviceContext<'_>,
    ) -> Option<vk::ExtendsDeviceCreateInfoObj> {
        let mut supported = vk::PhysicalDeviceRobustness2FeaturesEXT::default();
        context.get_features(&mut supported);
        if supported.robust_buffer_access2 != 0 {
            context.register_attribute(DeviceAttribute::new_bool(
                IS_ROBUST_BUFFER_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME, true,
            ));
        }
        if supported.robust_image_access2 != 0 {
            context.register_attribute(DeviceAttribute::new_bool(
                IS_ROBUST_IMAGE_ACCESS_2_SUPPORTED_ATTRIBUTE_NAME, true,
            ));
        }
        let mut features = vk::PhysicalDeviceRobustness2FeaturesEXT::default();
        if matches!(self.robust_buffer_access2, RobustAccessRequirements::Enabled) {
            context.register_attribute(DeviceAttribute::new_bool(
                IS_ROBUST_BUFFER_ACCESS_2_ENABLED_ATTRIBUTE_NAME, true,
            ));
            features.robust_buffer_access2 = vk::TRUE;
        }
        if matches!(self.robust_image_access2, RobustAccessRequirements::Enabled) {
            context.register_attribute(DeviceAttribute::new_bool(
                IS_ROBUST_IMAGE_ACCESS_2_ENABLED_ATTRIBUTE_NAME, true,
            ));
            features.robust_image_access2 = vk::TRUE;
        }
        if self.enable_null_descriptor {
            context.register_attribute(DeviceAttribute::new_bool(
                IS_ROBUST_NULL_DESCRIPTOR_ENABLED_ATTRIBUTE_NAME, true,
            ));
            features.null_descriptor = vk::TRUE;
        }
        Some(create_extends_device_create_info_obj(features))
    }

    fn boxed(&self) -> Box<dyn DeviceExtension> {
        Box::new(*self)
    }
}
