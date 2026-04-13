use nox_ash::{ash_style_enum, vk};

use crate::gpu::prelude::*;

ash_style_enum! {

    /// Specifies a bitmask of memory properties a [`memory binder`][1] uses when selecting
    /// memory types for its allocations.
    /// 
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkMemoryPropertyFlagBits.html>
    ///
    /// [1]: super::MemoryBinder
    #[flags(Flags32)]
    pub enum MemoryProperties {
        /// Specifies that memory allocated with this type is the most efficient for [`device`][1] access.
        ///
        /// [1]: LogicalDevice
        #[display("device local")]
        DEVICE_LOCAL = 0x00000001,
        /// Specifies that memory allocated with this type *can* be mapped for host access.
        #[display("host visible")]
        HOST_VISIBLE = 0x00000002,
        /// Specifies that host cache management commands are not needed to manage availability and
        /// visibility on the host.
        #[display("host coherent")]
        HOST_COHERENT = 0x00000004,
        /// Specifies that memory allocated with this type is cached on the host.
        #[display("host cached")]
        HOST_CACHED = 0x00000008,
        /// Specifies that the memory type only allows [`device`][1] access to memory.
        ///
        /// Additionally, the object's backing memory *may* be provided by the implementation
        /// lazily.
        ///
        /// [1]: LogicalDevice
        #[display("lazily allocated")]
        LAZILY_ALLOCATED = 0x00000010,
    }
}

impl From<MemoryProperties> for vk::MemoryPropertyFlags {
    
    #[inline]
    fn from(value: MemoryProperties) -> Self {
        Self::from_raw(value.as_raw())
    }
}
