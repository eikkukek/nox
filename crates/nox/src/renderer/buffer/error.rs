use ash::vk;

use crate::memory_binder::MemoryBinderError;

#[derive(Debug)]
pub enum BufferError {
    OutOfRange {
        buffer_size: u64, 
        requested_offset: u64, requested_size: u64,
    },
    UsageMismatch {
        missing_usage: vk::BufferUsageFlags,
    },
    UnbindedMemory,
    MemoryBinderError(MemoryBinderError),
}

impl core::fmt::Display for BufferError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfRange { buffer_size, requested_offset, requested_size } =>
                write!(f, "offset {requested_offset} and size {requested_size} out of range with buffer size {buffer_size}"),
            Self::UsageMismatch { missing_usage } =>
                write!(f, "usage mismatch, missing usage {:?}", missing_usage),
            Self::UnbindedMemory => write!(f, "unbinded memory"),
            Self::MemoryBinderError(err) => write!(f, "{err}"),
        }
    }
}

impl core::error::Error for BufferError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MemoryBinderError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<MemoryBinderError> for BufferError {

    fn from(value: MemoryBinderError) -> Self {
        Self::MemoryBinderError(value)
    }
}
