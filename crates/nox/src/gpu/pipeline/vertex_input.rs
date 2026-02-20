use nox_ash::vk;

use nox_mem::AsRaw;

use crate::gpu::VkFormat;

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum VertexInputRate {
    #[default]
    Vertex = vk::VertexInputRate::VERTEX.as_raw(),
    Instance = vk::VertexInputRate::INSTANCE.as_raw(),
}

impl From<VertexInputRate> for vk::VertexInputRate {

    fn from(value: VertexInputRate) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexInputAttribute {
    pub location: u32,
    pub format: VkFormat,
    pub offset: u32,
}

impl VertexInputAttribute {

    pub(crate) fn into_internal(self, binding: u32) -> VertexInputAttributeInternal {
        VertexInputAttributeInternal {
            location: self.location,
            binding,
            format: self.format,
            offset: self.offset,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VertexInputAttributeInternal {
    pub location: u32,
    pub binding: u32,
    pub format: vk::Format,
    pub offset: u32,
}

impl From<VertexInputAttributeInternal> for vk::VertexInputAttributeDescription {

    fn from(value: VertexInputAttributeInternal) -> Self {
        Self {
            location: value.location,
            binding: value.binding,
            format: value.format,
            offset: value.offset,
        }
    }
}

/// A trait for reflecting vulkan vertex input attributes from a Rust struct.
///
/// To implement this for a repr(C) struct, you can use the [`VertexInput`] derive macro.
pub trait VertexInput<const N_ATTRIBUTES: usize> {

    fn get_attributes(first_location: u32) -> [VertexInputAttribute; N_ATTRIBUTES];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct VertexInputBinding {
    pub binding: u32,
    pub input_rate: VertexInputRate,
    pub stride: u32,
}

impl VertexInputBinding {

    pub fn new(
        binding: u32,
        input_rate: VertexInputRate,
        stride: u32
    ) -> Self
    {
        Self {
            binding,
            input_rate,
            stride,
        }
    }
}

impl From<VertexInputBinding> for vk::VertexInputBindingDescription {

    fn from(value: VertexInputBinding) -> Self {
        Self {
            binding: value.binding,
            stride: value.stride,
            input_rate: value.input_rate.into(),
        }
    }
}
