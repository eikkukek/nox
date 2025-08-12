use ash::vk;

use nox_mem::{const_assert, size_of, CapacityError, slot_map::SlotMapError};

use crate::renderer::{
    ImageError,
    BufferError
};

#[derive(Clone, Debug)]
pub enum Error {
    CapacityError(CapacityError),
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    ShaderError(String),
    OutOfDeviceMemory { size: u64, align: u64, avail: u64, },
    DescriptorPoolFull { max_sets: u32, allocated_sets: u32 },
    InvalidHostCopy { copy_size: u64, host_buffer_size: usize, },
    IncompatibleMemoryRequirements,
    ImageError(ImageError),
    BufferError(BufferError),
    IoError(String),
    UserError(String),
}

const_assert!(size_of!(shaderc::Error) == 32);

impl From<CapacityError> for Error {

    fn from(value: CapacityError) -> Self {
        Self::CapacityError(value)
    }
}

impl From<SlotMapError> for Error {

    fn from(value: SlotMapError) -> Self {
        Self::SlotMapError(value)
    }
}

impl From<vk::Result> for Error {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl From<shaderc::Error> for Error {

    fn from(value: shaderc::Error) -> Self {
        Self::ShaderError("shaderc error: ".to_string() + &value.to_string())
    }
}

impl From<rspirv_reflect::ReflectError> for Error {

    fn from(value: rspirv_reflect::ReflectError) -> Self {
        Self::ShaderError("spirv cross error: ".to_string() + &value.to_string())
    }
}

impl From<ImageError> for Error {

    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl From<BufferError> for Error {

    fn from(value: BufferError) -> Self {
        Self::BufferError(value)
    }
}

impl From<std::io::Error> for Error {

    fn from(value: std::io::Error) -> Self {
        Self::IoError(value.to_string())
    }
}
