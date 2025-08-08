use ash::vk;

#[derive(Clone, Debug)]
pub enum BufferError {
    OutOfRange {
        buffer_size: u64, 
        requested_offset: u64, requested_size: u64,
    },
    UsageMismatch {
        missing_usage: vk::BufferUsageFlags,
    }
}
