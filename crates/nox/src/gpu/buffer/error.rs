use ash::vk;

use crate::dev::error::Error;

use crate::gpu::memory_binder::MemoryBinderError;

#[derive(Debug, Error)]
pub enum BufferError {
    #[display("offset {requested_offset} and size {requested_size} was out of range of buffer size {buffer_size}")]
    OutOfRange {
        buffer_size: u64, 
        requested_offset: u64, requested_size: u64,
    },
    #[display("buffer usage mismatch, missing usage {missing_usage:?}")]
    UsageMismatch {
        missing_usage: vk::BufferUsageFlags,
    },
    #[display("unbinded buffer memory")]
    UnbindedMemory,
    #[display("memory binder error")]
    MemoryBinderError(#[from] #[source] MemoryBinderError),
}
