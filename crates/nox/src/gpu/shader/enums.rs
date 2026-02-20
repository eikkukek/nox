use nox_ash::vk;

use nox_proc::Display;
use nox_mem::AsRaw;

use spirv_cross2::spirv;

#[repr(u32)]
#[derive(Display, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ShaderStage {
    #[display("unknown")]
    Unknown = 0,
    #[display("vertex")]
    Vertex = 1,
    #[display("fragment")]
    Fragment = 2,
    #[display("compute")]
    Compute = 4,
}

impl ShaderStage {

    pub(super) fn execution_model(self) -> Option<spirv::ExecutionModel> {
        match self {
            Self::Unknown => None,
            Self::Vertex => Some(spirv::ExecutionModel::Vertex),
            Self::Fragment => Some(spirv::ExecutionModel::Fragment),
            Self::Compute => Some(spirv::ExecutionModel::GLCompute),
        }
    }
}

impl From<vk::ShaderStageFlags> for ShaderStage {

    fn from(value: vk::ShaderStageFlags) -> Self {
        match value {
            vk::ShaderStageFlags::VERTEX => Self::Vertex,
            vk::ShaderStageFlags::FRAGMENT => Self::Fragment,
            vk::ShaderStageFlags::COMPUTE => Self::Compute,
            _ => Self::Unknown,
        }
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {

    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Unknown => Self::empty(),
            ShaderStage::Vertex => Self::VERTEX,
            ShaderStage::Fragment => Self::FRAGMENT,
            ShaderStage::Compute => Self::COMPUTE,
        }
    }
}

impl From<ShaderStage> for shaderc::ShaderKind {

    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Unknown => Self::AnyHit,
            ShaderStage::Vertex => Self::Vertex,
            ShaderStage::Fragment => Self::Fragment,
            ShaderStage::Compute => Self::Compute,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Display, Clone, Copy, AsRaw, PartialEq, Eq, Hash)]
pub enum DescriptorType {
    #[display("unknown")]
    Unknown = -1,
    /// A type associated with [`Sampler`].
    ///
    /// # Shader declaratios
    /// Glsl: `uniform sampler ...`
    #[display("sampler")]
    Sampler = vk::DescriptorType::SAMPLER.as_raw(),
    /// A type associated with a sampled [`Image`].
    ///
    /// # Shader declarations
    /// Glsl: `uniform texture2D ...`
    #[display("sampled image")]
    SampledImage = vk::DescriptorType::SAMPLED_IMAGE.as_raw(),
    /// A type that combines both a [`Sampler`] and a sampled [`Image`].
    ///
    /// # Shader declarations
    /// Glsl: `uniform sampler2D ...`
    #[display("combined image sampler")]
    CombinedImageSampler = vk::DescriptorType::COMBINED_IMAGE_SAMPLER.as_raw(),
    /// A type associated with an [`Image`] that can be used for load, store and atomic operations.
    ///
    /// # Shader declarations
    /// Glsl: `uniform image2D ...`
    #[display("storage image")]
    StorageImage = vk::DescriptorType::STORAGE_IMAGE.as_raw(),
    /// A type associated with a [`Buffer`] that can be used for load operations.
    ///
    /// # Shader declarations
    /// Glsl: `uniform UBO { ... }`
    #[display("uniform buffer")]
    UniformBuffer = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] that can be used for load, store and atomic operations.
    ///
    /// # Shader declarations
    /// Glsl: `buffer SSBO { ... }`
    #[display("storage buffer")]
    StorageBuffer = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] and a buffer view that can be used for image sampling
    /// operations.
    ///
    /// # Shader declarations
    /// Glsl: `uniform samplerBuffer ...`
    #[display("uniform texel buffer")]
    UniformTexelBuffer = vk::DescriptorType::UNIFORM_TEXEL_BUFFER.as_raw(),
    /// A type associated with a [`Buffer`] and a buffer view that can be used for image load,
    /// store and atomic operations.
    ///
    /// # Shader declarations
    /// Glsl: `uniform imageBuffer ...`
    #[display("storage texel buffer")]
    StorageTexelBuffer = vk::DescriptorType::STORAGE_TEXEL_BUFFER.as_raw(),
    /// A type similar to [`DescriptorType::UniformBuffer`] where it's storage comes directly from
    /// the shader resource rather than from a separate [`Buffer`].
    ///
    /// The use of inline uniform blocks in shaders is unsupported for now due to lack of support
    /// in GLSL.
    #[display("inline uniform block")]
    InlineUniformBlock = vk::DescriptorType::INLINE_UNIFORM_BLOCK.as_raw(),
}

impl DescriptorType {

    #[inline(always)]
    pub fn is_unsupported(self) -> bool {
        matches!(self,
            Self::Unknown |
            Self::InlineUniformBlock,
        )
    }

    #[inline(always)]
    pub fn is_buffer(self) -> bool {
        matches!(self,
            Self::UniformBuffer |
            Self::StorageBuffer |
            Self::UniformTexelBuffer |
            Self::StorageTexelBuffer,
        )
    }

    #[inline(always)]
    pub(crate) fn buffer_usage(self) -> Option<vk::BufferUsageFlags> {
        match self {
            Self::UniformBuffer => Some(vk::BufferUsageFlags::UNIFORM_BUFFER),
            Self::StorageBuffer => Some(vk::BufferUsageFlags::STORAGE_BUFFER),
            Self::UniformTexelBuffer => Some(vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER),
            Self::StorageTexelBuffer => Some(vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_image(self) -> bool {
        matches!(self,
            Self::Sampler |
            Self::SampledImage |
            Self::StorageImage |
            Self::CombinedImageSampler,
        )
    }

    #[inline(always)]
    pub(crate) fn image_usage(self) -> Option<vk::ImageUsageFlags> {
        match self {
            Self::Sampler => Some(vk::ImageUsageFlags::empty()),
            Self::SampledImage => Some(vk::ImageUsageFlags::SAMPLED),
            Self::StorageImage => Some(vk::ImageUsageFlags::STORAGE),
            Self::CombinedImageSampler => Some(vk::ImageUsageFlags::SAMPLED),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn requires_sampler(self) -> bool {
        matches!(self,
            Self::Sampler |
            Self::CombinedImageSampler,
        )
    }

    #[inline(always)]
    pub fn requires_image(self) -> bool {
        matches!(self,
            Self::SampledImage |
            Self::StorageImage |
            Self::CombinedImageSampler,
        )
    }
}

impl From<DescriptorType> for vk::DescriptorType {

    fn from(value: DescriptorType) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl From<vk::DescriptorType> for DescriptorType {

    fn from(value: vk::DescriptorType) -> Self {
        match value {
            vk::DescriptorType::SAMPLER => Self::Sampler,
            vk::DescriptorType::SAMPLED_IMAGE => Self::SampledImage,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER => Self::CombinedImageSampler,
            vk::DescriptorType::STORAGE_IMAGE => Self::StorageImage,
            vk::DescriptorType::UNIFORM_BUFFER => Self::UniformBuffer,
            vk::DescriptorType::STORAGE_BUFFER => Self::StorageBuffer,
            vk::DescriptorType::UNIFORM_TEXEL_BUFFER => Self::UniformTexelBuffer,
            vk::DescriptorType::STORAGE_TEXEL_BUFFER => Self::StorageTexelBuffer,
            vk::DescriptorType::INLINE_UNIFORM_BLOCK => Self::InlineUniformBlock,
            _ => Self::Unknown,
        }
    }
}
