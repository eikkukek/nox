//! Provided by Vulkan 1.4 or otherwise by VK_EXT_pipeline_robustness.

use super::*;

use nox_ash::ext;

/// Attribute type `bool`.
pub const IS_ENABLED_ATTRIBUTE_NAME: ConstName
    = ConstName::new("pipeline_robustness enabled");

/// Attribute type `i32`, can be safely cast to [`vk::PipelineRobustnessBufferBehaviour`].
pub const DEFAULT_ROBUSTNESS_STORAGE_BUFFERS_ATTRIBUTE_NAME: ConstName
    = ConstName::new("pipeline_robustness default_robustness_storage_buffers");

/// Attribute type `i32`, can be safely cast to [`vk::PipelineRobustnessBufferBehaviour`].
pub const DEFAULT_ROBUSTNESS_UNIFORM_BUFFERS_ATTRIBUTE_NAME: ConstName
    = ConstName::new("pipeline_robustness default_robustness_uniform_buffers");

/// Attribute type `i32`, can be safely cast to [`vk::PipelineRobustnessBufferBehaviour`].
pub const DEFAULT_ROBUSTNESS_VERTEX_INPUTS_ATTRIBUTE_NAME: ConstName
    = ConstName::new("pipeline_robustness default_robustness_vertex_inputs");

/// Attribute type `i32`, can be safely cast to [`vk::PipelineRobustnessImageBehaviour`].
pub const DEFAULT_ROBUSTNESS_IMAGE_ATTRIBUTE_NAME: ConstName
    = ConstName::new("pipeline_robustness default_robustness_image_attribute_name");

#[derive(Clone, Copy)]
pub struct Extension;

unsafe impl DeviceExtension for Extension {

    fn get_info(&self, _attributes: &GpuAttributes) -> Option<DeviceExtensionInfo> {
        Some(DeviceExtensionInfo {
            name: ext::pipeline_robustness::NAME,
            deprecation_version: Version::VULKAN_API_VERSION_1_4,
            precondition: Precondition::new(|context| {
                let mut features = vk::PhysicalDevicePipelineRobustnessFeatures::default();
                context.get_features(&mut features);
                (features.pipeline_robustness == 0).then(||
                    MissingDeviceFeatureError::new("pipeline robustness")
                )
            }),
        })
    }

    fn register(
        &self,
        context: &mut PhysicalDeviceContext<'_>,
    ) -> Option<vk::ExtendsDeviceCreateInfoObj> {
        let mut properties = vk::PhysicalDevicePipelineRobustnessProperties::default();
        context.get_properties(&mut properties);
        context.register_attribute(DeviceAttribute::new_i32(
            DEFAULT_ROBUSTNESS_STORAGE_BUFFERS_ATTRIBUTE_NAME,
            properties.default_robustness_storage_buffers.as_raw(),
        ));
        context.register_attribute(DeviceAttribute::new_i32(
            DEFAULT_ROBUSTNESS_UNIFORM_BUFFERS_ATTRIBUTE_NAME,
            properties.default_robustness_uniform_buffers.as_raw(),
        ));
        context.register_attribute(DeviceAttribute::new_i32(
            DEFAULT_ROBUSTNESS_VERTEX_INPUTS_ATTRIBUTE_NAME,
            properties.default_robustness_vertex_inputs.as_raw(),
        ));
        context.register_attribute(DeviceAttribute::new_i32(
            DEFAULT_ROBUSTNESS_IMAGE_ATTRIBUTE_NAME,
            properties.default_robustness_images.as_raw(),
        ));
        Some(create_extends_device_create_info_obj(vk::PhysicalDevicePipelineRobustnessFeatures
            ::default().pipeline_robustness(true)
        ))
    }

    fn boxed(&self) -> Box<dyn DeviceExtension> {
        Box::new(*self)
    }
}
