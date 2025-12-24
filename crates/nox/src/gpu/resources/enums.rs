use ash::vk;

use nox_mem::AsRaw;

#[repr(i32)]
#[derive(Default, Clone, Copy, AsRaw, Debug)]
pub enum IndexType {
    U16 = vk::IndexType::UINT16.as_raw(),
    #[default]
    U32 = vk::IndexType::UINT32.as_raw(),
}

impl IndexType {
    
    pub fn index_size(self) -> u64 {
        match self {
            Self::U16 => 2,
            Self::U32 => 4,
        }
    }
}

impl From<IndexType> for vk::IndexType {

    fn from(value: IndexType) -> Self {
        vk::IndexType::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, AsRaw, Debug)]
pub enum DescriptorType {
    /// A type associated with [`Sampler`].
    Sampler = vk::DescriptorType::SAMPLER.as_raw(),
    /// A type associated with a sampled [`Image`].
    SampledImage = vk::DescriptorType::SAMPLED_IMAGE.as_raw(),
    /// A type that combines both a [`Sampler`] and a sampled [`Image`].
    CombinedImageSampler = vk::DescriptorType::COMBINED_IMAGE_SAMPLER.as_raw(),
    /// A type associated with an [`Image`] that can be used for load, store and atomic operations.
    StorageImage = vk::DescriptorType::STORAGE_IMAGE.as_raw(),
    /// A type associated with a [`Buffer`] that can be used for load operations.
    UniformBuffer = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] that can be used for load, store and atomic operations.
    StorageBuffer = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] and a buffer view that can be used for image sampling
    /// operations.
    UniformTexelBuffer = vk::DescriptorType::UNIFORM_TEXEL_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] and a buffer view that can be used for image load,
    /// store and atomic operations.
    StorageTexelBuffer = vk::DescriptorType::STORAGE_TEXEL_BUFFER.as_raw(),
    /// A type similar to [`DescriptorType::UniformBuffer`] where it's storage comes directly from
    /// the shader resource rather than from a separate [`Buffer`].
    ///
    /// For this to be used when updating shader resources, [`Extension::InlineUniformBlock`] needs
    /// to be enabled.
    InlineUniformBlock = vk::DescriptorType::INLINE_UNIFORM_BLOCK.as_raw(),
}
