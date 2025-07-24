use ash::vk;

use nox_mem::{const_assert, size_of, CapacityError};

use super::image::ImageError;

#[derive(Clone, Debug)]
pub enum Error {
    CapacityError(CapacityError),
    VulkanError(vk::Result),
    ShadercError(String),
    OutOfDeviceMemory { size: vk::DeviceSize, align: vk::DeviceSize, avail: vk::DeviceSize },
    IncompatibleMemoryRequirements,
    ImageError(ImageError),
}

const_assert!(size_of!(shaderc::Error) == 32);

impl From<CapacityError> for Error {

    fn from(value: CapacityError) -> Self {
        Self::CapacityError(value)
    }
}

impl From<vk::Result> for Error {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}

impl From<shaderc::Error> for Error {

    fn from(value: shaderc::Error) -> Self {
        Self::ShadercError(value.to_string())
    }
}

impl From<ImageError> for Error {

    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}
