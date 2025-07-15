use ash::vk;

use nox_mem::CapacityError;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    CapacityError(CapacityError),
    VulkanError(vk::Result),
}

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
