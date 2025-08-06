use ash::vk;

#[derive(Clone, Debug)]
pub enum BufferError {
    InvalidCopy {
        buffer_size: u64, 
        copy_offset: u64, copy_size: u64,
    },
    UsageMismatch {
        missing_usage: vk::BufferUsageFlags,
    }
}
