use core::{
    num::NonZeroU64,
    ptr::NonNull,
};

use nox_ash::vk;

use crate::gpu::prelude::{
    memory_binder::MemoryBinder,
    dev_error,
    *
};

use crate::sync::*;

/// The create info structure for buffers. See [`BufferCreateInfo::new`] for full description.
pub struct BufferCreateInfo<'a> {
    pub(crate) out: &'a mut BufferId,
    pub(crate) size: NonZeroU64,
    pub(crate) usage: BufferUsageFlags,
    pub(crate) create_flags: vk::BufferCreateFlags,
    pub(crate) memory_binder: ResourceBinder,
}

impl<'a> BufferCreateInfo<'a> {

    /// Creates new [`BufferCreateInfo`].
    ///
    /// # Parameters
    /// - `out`: A mutable reference to where the [`BufferId`] of the created buffer will be stored.
    /// - `size`: The size of the created buffer, must be non-zero.
    /// - `usage`: Specifies what the buffer *can* be used for.
    ///
    /// Returns [`None`] if buffer size is `0`.
    ///
    /// # Memory binding
    /// The default memory binder is [`ResourceBinder::default`], which always allocates a new
    /// [`vk::DeviceMemory`] object and is *not* mappable.
    ///
    /// You can specify a different memory binder by [`BufferCreateInfo::with_memory_binder`].
    #[inline(always)]
    pub fn new(
        out: &'a mut BufferId,
        size: DeviceSize,
        usage: BufferUsageFlags,
    ) -> Option<Self> {
        Some(Self {
            out,
            size: NonZeroU64::new(size)?,
            usage,
            create_flags: vk::BufferCreateFlags::empty(),
            memory_binder: Default::default(),
        })
    }

    /// Specifies where the buffer gets its memory.
    ///
    /// The default is [`ResourceBinder::default`].
    #[inline(always)]
    pub fn with_memory_binder(mut self, binder: ResourceBinder) -> Self {
        self.memory_binder = binder;
        self
    } 

    pub(crate) fn build(
        &self,
        vk: Arc<Vulkan>,
        alloc: &mut (impl MemoryBinder + ?Sized),
        bind_memory_info: &mut vk::BindBufferMemoryInfo<'static>,
    ) -> dev_error::Result<BufferMeta> {
        BufferMeta::new(vk, self, alloc, bind_memory_info)
    }
}
