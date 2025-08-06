use ash::vk;

use nox_mem::{AsRaw, size_of};

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
    pub format: vk::Format,
    pub offset: u32,
}

impl VertexInputAttribute {

    pub(crate) fn into_vk(self, binding: u32) -> vk::VertexInputAttributeDescription {
        vk::VertexInputAttributeDescription {
            location: self.location,
            binding,
            format: self.format,
            offset: self.offset,
        }
    }
}

pub unsafe trait VertexInput: Sized {

    fn get_attributes<const FIRST_LOCATION: u32>() -> &'static [VertexInputAttribute];
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexInputBinding {
    pub binding: u32,
    pub input_rate: VertexInputRate,
    pub(crate) stride: u32,
    pub(crate) attributes: &'static [VertexInputAttribute],
}

impl VertexInputBinding {

    pub fn new<const FIRST_LOCATION: u32, I: VertexInput>(
        binding: u32,
        input_rate: VertexInputRate,
    ) -> Self
    {
        Self {
            binding,
            input_rate,
            stride: size_of!(I) as u32,
            attributes: I::get_attributes::<FIRST_LOCATION>(),
        }
    }

    pub fn first_location(&self) -> u32 {
        self.attributes.first().unwrap().location
    }

    pub fn last_location(&self) -> u32 {
        self.attributes.last().unwrap().location
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
