use core::{
    str::Utf8Error,
    fmt::Display,
};

use ash::vk;

use compact_str::CompactString;

use nox_mem::{
    vec_types::VecError,
    slot_map::SlotMapError,
    string_types::StringError,
};

use crate::renderer::{
    ResourceError,
    PipelineError,
    memory_binder::MemoryBinderError,
    ShaderError,
    ImageError,
    BufferError,
    FrameGraphError,
    CommandError,
};

#[derive(Debug)]
pub enum Error {
    VecError(VecError),
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    ResourceError(ResourceError),
    PipelineError(PipelineError),
    MemoryBinderError(MemoryBinderError),
    ShaderError(ShaderError),
    ImageError(ImageError),
    BufferError(BufferError),
    FrameGraphError(FrameGraphError),
    CommandError(CommandError),
    IoError(std::io::Error),
    Other(Box<dyn core::error::Error>),
}

impl Error {

    pub fn other(err: impl core::error::Error + 'static) -> Self {
        Self::Other(Box::new(err))
    }
}

impl Display for Error {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::SlotMapError(_) => write!(f, "slot map error"),
            Self::VulkanError(_) => write!(f, "vulkan error"),
            Self::ResourceError(_) => write!(f, "resource error"),
            Self::PipelineError(_) => write!(f, "pipeline error"),
            Self::MemoryBinderError(_) => write!(f, "memory binder error"),
            Self::ShaderError(_) => write!(f, "shader error"),
            Self::ImageError(_) => write!(f, "image error"),
            Self::BufferError(_) => write!(f, "buffer error"),
            Self::FrameGraphError(_) => write!(f, "frame graph error"),
            Self::CommandError(_) => write!(f, "command error"),
            Self::IoError(_) => write!(f, "io error"),
            Self::Other(_) => write!(f, "other error"),
        }
    }
}

impl core::error::Error for Error {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::SlotMapError(err) => Some(err),
            Self::VulkanError(err) => Some(err),
            Self::ResourceError(err) => Some(err),
            Self::PipelineError(err) => Some(err),
            Self::MemoryBinderError(err) => Some(err),
            Self::ShaderError(err) => Some(err),
            Self::ImageError(err) => Some(err),
            Self::BufferError(err) => Some(err),
            Self::FrameGraphError(err) => Some(err),
            Self::CommandError(err) => Some(err),
            Self::IoError(err) => Some(err),
            Self::Other(err) => Some(&**err),
        }
    }
}

impl From<VecError> for Error {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
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

impl From<ResourceError> for Error {

    fn from(value: ResourceError) -> Self {
        Self::ResourceError(value)
    }
}

impl From<PipelineError> for Error {

    fn from(value: PipelineError) -> Self {
        Self::PipelineError(value)
    }
}

impl From<MemoryBinderError> for Error {

    fn from(value: MemoryBinderError) -> Self {
        Self::MemoryBinderError(value)
    }
}

impl From<ShaderError> for Error {

    fn from(value: ShaderError) -> Self {
        Self::ShaderError(value)
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

impl From<FrameGraphError> for Error {

    fn from(value: FrameGraphError) -> Self {
        Self::FrameGraphError(value)
    }
}

impl From<CommandError> for Error {
    
    fn from(value: CommandError) -> Self {
        Self::CommandError(value)
    }
}

impl From<std::io::Error> for Error {

    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug)]
pub enum InitError {
    VecError(VecError),
    StringError(StringError),
    Utf8Error(Utf8Error),
    InstanceLayerNotPresent(CompactString),
    InstanceExtensionNotPresent(CompactString),
    InstanceCreateError(vk::Result),
    SurfaceCreateError(vk::Result),
    FailedToQuerySurfaceSupport(vk::Result),
    FailedToEnumeratePhysicalDevices(vk::Result),
    SuitablePhysicalDeviceNotFound,
    FailedToCreateDevice(vk::Result),
    UnexpectedVulkanError(vk::Result),
    GlobalAllocFailed,
    UnsupportedPlatform,
    SwapchainPassError(Box<Error>),
}

impl Display for InitError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::StringError(_) => write!(f, "string error"),
            Self::Utf8Error(_) => write!(f, "utf8 error"),
            Self::InstanceLayerNotPresent(layer) => write!(f, "vulkan instance layer {} was not present", layer),
            Self::InstanceExtensionNotPresent(ext) => write!(f, "vulkan instance extension {} was not present", ext),
            Self::InstanceCreateError(_) => write!(f, "failed to create vulkan instance"),
            Self::SurfaceCreateError(_) => write!(f, "failed to create vulkan surface"),
            Self::FailedToQuerySurfaceSupport(_) => write!(f, "failed to query vulkan surface support"),
            Self::FailedToEnumeratePhysicalDevices(_) => write!(f, "failed to enumerate GPUs"),
            Self::SuitablePhysicalDeviceNotFound => write!(f, "failed to find suitable GPU"),
            Self::FailedToCreateDevice(_) => write!(f, "failed to create device"),
            Self::UnexpectedVulkanError(_) => write!(f, "unexpected vulkan error"),
            Self::GlobalAllocFailed => write!(f, "global alloc failed"),
            Self::UnsupportedPlatform => write!(f, "unsupported platform"),
            Self::SwapchainPassError(_) => write!(f, "failed to create swapchain pass"),
        }
    }
}

impl core::error::Error for InitError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::StringError(err) => Some(err),
            Self::Utf8Error(err) => Some(err),
            Self::InstanceLayerNotPresent(_) => None,
            Self::InstanceExtensionNotPresent(_) => None,
            Self::InstanceCreateError(err) => Some(err),
            Self::SurfaceCreateError(err) => Some(err),
            Self::FailedToQuerySurfaceSupport(err) => Some(err),
            Self::FailedToEnumeratePhysicalDevices(err) => Some(err),
            Self::SuitablePhysicalDeviceNotFound => None,
            Self::FailedToCreateDevice(err) => Some(err),
            Self::UnexpectedVulkanError(err) => Some(err),
            Self::GlobalAllocFailed => None,
            Self::UnsupportedPlatform => None,
            Self::SwapchainPassError(err) => Some(err),
        }
    }
}

impl From<VecError> for InitError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<StringError> for InitError {

    fn from(value: StringError) -> Self {
        Self::StringError(value)
    }
}

impl From<Utf8Error> for InitError {

    fn from(value: Utf8Error) -> Self {
        Self::Utf8Error(value)
    }
}
