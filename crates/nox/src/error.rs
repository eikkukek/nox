mod any;

use core::str::Utf8Error;

use ash::vk;

use compact_str::CompactString;

use nox_derive::Error;

use nox_mem::{
    vec_types::VecError,
    slot_map::SlotMapError,
    string_types::StringError,
};

pub use crate::renderer::{
    ResourceError,
    PipelineError,
    memory_binder::MemoryBinderError,
    ShaderError,
    ImageError,
    BufferError,
    FrameGraphError,
    CommandError,
};

use core::error;

pub use any::{AnyError, SomeError};

#[derive(Error, Debug)]
pub enum Error {

    #[display("vec error")]
    VecError(#[from] #[source] VecError),

    #[display("slot map error")]
    SlotMapError(#[from] #[source] SlotMapError),

    #[display("vulkan error")]
    VulkanError(#[from] #[source] vk::Result),

    #[display("resource error")]
    ResourceError(#[from] #[source] ResourceError),

    #[display("pipeline error")]
    PipelineError(#[from] #[source] PipelineError),

    #[display("memory binder error")]
    MemoryBinderError(#[from] #[source] MemoryBinderError),

    #[display("shader error")]
    ShaderError(#[from] #[source] ShaderError),

    #[display("image error")]
    ImageError(#[from] #[source] ImageError),

    #[display("buffer error")]
    BufferError(#[from] #[source] BufferError),

    #[display("frame graph error")]
    FrameGraphError(#[from] #[source] FrameGraphError),

    #[display("command error")]
    CommandError(#[from] #[source] CommandError),

    #[display("io error")]
    IoError(#[from] #[source] std::io::Error),

    #[display("{0}")]
    Other(#[source(err.source())] AnyError),
}

impl Error {

    pub fn new(desc: &str, err: impl error::Error + 'static) -> Self {
        Self::Other(AnyError::new(desc, err))
    }
}

impl<T: Into<AnyError>> From<T> for Error {

    fn from(value: T) -> Self {
        Self::Other(value.into())
    }
}

#[derive(Error, Debug)]
pub enum InitError {

    #[display("vec error")]
    VecError(#[from] #[source] VecError),

    #[display("string error")]
    StringError(#[from] #[source] StringError),

    #[display("utf8 error")]
    Utf8Error(#[from] #[source] Utf8Error),

    #[display("vulkan instance layer {0} was not present")]
    InstanceLayerNotPresent(CompactString),

    #[display("vulkan instance extension {0} was not present")]
    InstanceExtensionNotPresent(CompactString),

    #[display("failed to create vulkan instance")]
    InstanceCreateError(#[source] vk::Result),

    #[display("failed to create vulkan surface")]
    SurfaceCreateError(#[source] vk::Result),

    #[display("failed to query vulkan surface support")]
    FailedToQuerySurfaceSupport(#[source] vk::Result),

    #[display("failed to enumerate GPUs")]
    FailedToEnumeratePhysicalDevices(#[source] vk::Result),

    #[display("failed to find suitable GPU")]
    SuitablePhysicalDeviceNotFound,

    #[display("failed to create vulkan device")]
    FailedToCreateDevice(#[source] vk::Result),

    #[display("unexpected vulkan error")]
    UnexpectedVulkanError(#[source] vk::Result),

    #[display("global alloc failed")]
    GlobalAllocFailed,

    #[display("unsupported platform")]
    UnsupportedPlatform,

    #[display("{0}")]
    SwapchainPassError(#[source(err.source())] AnyError),
}
