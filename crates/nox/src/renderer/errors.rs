use ash::vk;

use core::{
    num::NonZeroI32,
    fmt::Display,
};

use nox_mem::vec_types::VecError;

use crate::{
    QueueFamily,
    Error,
    CommandError,
    ResourceError,
    FrameGraphError,
};

#[derive(Debug)]
pub struct QueueSubmitError(pub QueueFamily, pub vk::Result);

impl Display for QueueSubmitError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} queue failed to submit due to vulkan error {}", self.0, self.1)
    }
}

impl core::error::Error for QueueSubmitError {}

#[derive(Debug)]
pub struct ImageAcquisitionError {
    pub result: Option<NonZeroI32>,
    pub at_creation: bool,
}

impl ImageAcquisitionError {

    pub fn new(
        result: vk::Result,
        at_creation: bool
    ) -> Self
    {
        Self {
            result: NonZeroI32::new(result.as_raw()),
            at_creation,
        }
    }
}

impl Display for ImageAcquisitionError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(result) = self.result.map(|v| vk::Result::from_raw(v.get())) {
            if self.at_creation {
                write!(f, "failed to acquire swapchain images at creation due to {result}")
            } else {
                write!(f, "failed to acquire swapchain images due to {result}")
            }
        } else {
            if self.at_creation {
                write!(f, "failed to acquire swapchain images at creation")
            } else {
                write!(f, "failed to acquire swapchain images")
            }
        }
    }
}

impl core::error::Error for ImageAcquisitionError {}

#[derive(Debug)]
pub enum RenderError {
    VecError(VecError),
    AsyncTransferRequestError(Error),
    CommandError(CommandError),
    ZeroSizedSwapchain,
    UnsupportedSwapchain,
    SwapchainCreationError(vk::Result),
    ImageAcquisitionError(ImageAcquisitionError),
    QueueSubmitError(QueueSubmitError),
    FrameBufferSizeCallbackError(Error),
    ComputeError(Error),
    RenderError(Error),
    FrameGraphError(FrameGraphError),
    QueuePresentError(vk::Result),
    ResourceError(ResourceError),
    UnexpectedVulkanError(vk::Result),
}

impl Display for RenderError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::AsyncTransferRequestError(_) => write!(f, "async transfer request failed"),
            Self::CommandError(_) => write!(f, "command error"),
            Self::ZeroSizedSwapchain => write!(f, "swapchain size is zero"),
            Self::UnsupportedSwapchain => write!(f, "unsupported swapchain"),
            Self::SwapchainCreationError(_) => write!(f, "failed to create swapchain"),
            Self::ImageAcquisitionError(_) => write!(f, "failed to acquire swapchain image"),
            Self::QueueSubmitError(err) => write!(f, "{} queue submit failed", err.0),
            Self::FrameBufferSizeCallbackError(_) => write!(f, "frame buffer size callback failed"),
            Self::ComputeError(_) => write!(f, "compute callback failed"),
            Self::RenderError(_) => write!(f, "render callback failed"),
            Self::FrameGraphError(_) => write!(f, "frame graph error"),
            Self::QueuePresentError(_) => write!(f, "queue present failed"),
            Self::ResourceError(_) => write!(f, "resource error"),
            Self::UnexpectedVulkanError(_) => write!(f, "unexpected vulkan error"),
        }
    }
}

impl core::error::Error for RenderError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::AsyncTransferRequestError(err) => Some(err),
            Self::CommandError(err) => Some(err),
            Self::ZeroSizedSwapchain => None,
            Self::UnsupportedSwapchain => None,
            Self::SwapchainCreationError(err) => Some(err),
            Self::ImageAcquisitionError(err) => Some(err),
            Self::QueueSubmitError(err) => Some(err),
            Self::FrameBufferSizeCallbackError(err) => Some(err),
            Self::ComputeError(err) => Some(err),
            Self::RenderError(err) => Some(err),
            Self::FrameGraphError(err) => Some(err),
            Self::QueuePresentError(err) => Some(err),
            Self::ResourceError(err) => Some(err),
            Self::UnexpectedVulkanError(err) => Some(err),
        }
    }
}

impl From<VecError> for RenderError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<CommandError> for RenderError {

    fn from(value: CommandError) -> Self {
        Self::CommandError(value)
    }
}

impl From<ImageAcquisitionError> for RenderError {

    fn from(value: ImageAcquisitionError) -> Self {
        Self::ImageAcquisitionError(value)
    }
}

impl From<QueueSubmitError> for RenderError {

    fn from(value: QueueSubmitError) -> Self {
        Self::QueueSubmitError(value)
    }
}

impl From<FrameGraphError> for RenderError {

    fn from(value: FrameGraphError) -> Self {
        Self::FrameGraphError(value)
    }
}

impl From<ResourceError> for RenderError {

    fn from(value: ResourceError) -> Self {
        Self::ResourceError(value)
    }
}
