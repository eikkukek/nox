use nox_ash::vk;

use nox_proc::Display;
use nox_mem::AsRaw;

use spirv_cross2::spirv;

use crate::gpu::prelude::*;

nox_ash::ash_style_enum!(
    /// A bitmask of [`shader stages`][1].
    ///
    /// [1]: ShaderStage
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum ShaderStageFlags {
        #[display("vertex")]
        VERTEX = vk::ShaderStageFlags::VERTEX.as_raw(),
        #[display("tessellation control")]
        TESSELLATION_CONTROL = vk::ShaderStageFlags::TESSELLATION_CONTROL.as_raw(),
        #[display("tessellation evaluation")]
        TESSELLATION_EVALUATION = vk::ShaderStageFlags::TESSELLATION_EVALUATION.as_raw(),
        #[display("geometry")]
        GEOMETRY = vk::ShaderStageFlags::GEOMETRY.as_raw(),
        #[display("fragment")]
        FRAGMENT = vk::ShaderStageFlags::FRAGMENT.as_raw(),
        #[display("compute")]
        COMPUTE = vk::ShaderStageFlags::COMPUTE.as_raw(),
    }
);

impl ShaderStageFlags {

    #[inline(always)]
    pub fn pipeline_stage_mask(self) -> vk::PipelineStageFlags2 {
        let mut mask = vk::PipelineStageFlags2::empty();
        if self.contains(Self::VERTEX) {
            mask |= vk::PipelineStageFlags2::VERTEX_SHADER;
        }
        if self.contains(Self::TESSELLATION_CONTROL) {
            mask |= vk::PipelineStageFlags2::TESSELLATION_CONTROL_SHADER;
        }
        if self.contains(Self::TESSELLATION_EVALUATION) {
            mask |= vk::PipelineStageFlags2::TESSELLATION_EVALUATION_SHADER;
        }
        if self.contains(Self::GEOMETRY) {
            mask |= vk::PipelineStageFlags2::GEOMETRY_SHADER;
        }
        if self.contains(Self::FRAGMENT) {
            mask |= vk::PipelineStageFlags2::FRAGMENT_SHADER;
        }
        if self.contains(Self::COMPUTE) {
            mask |= vk::PipelineStageFlags2::COMPUTE_SHADER;
        }
        mask
    }
}

/// An enumeration of all supported shader stages.
#[repr(u32)]
#[derive(Display, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ShaderStage {
    #[display("vertex")]
    Vertex = ShaderStageFlags::VERTEX.as_raw(),
    #[display("tessellation control")]
    TesellationControl = ShaderStageFlags::TESSELLATION_CONTROL.as_raw(),
    #[display("tessellation evaluation")]
    TesellationEvaluation = ShaderStageFlags::TESSELLATION_EVALUATION.as_raw(),
    #[display("geometry")]
    Geometry = ShaderStageFlags::GEOMETRY.as_raw(),
    #[display("fragment")]
    Fragment = ShaderStageFlags::FRAGMENT.as_raw(),
    #[display("compute")]
    Compute = ShaderStageFlags::COMPUTE.as_raw(),
}

impl ShaderStage {

    pub(super) fn execution_model(self) -> spirv::ExecutionModel {
        match self {
            Self::Vertex => spirv::ExecutionModel::Vertex,
            Self::TesellationControl => spirv::ExecutionModel::TessellationControl,
            Self::TesellationEvaluation => spirv::ExecutionModel::TessellationEvaluation,
            Self::Geometry => spirv::ExecutionModel::Geometry,
            Self::Fragment => spirv::ExecutionModel::Fragment,
            Self::Compute => spirv::ExecutionModel::GLCompute,
        }
    }
}

impl From<ShaderStage> for ShaderStageFlags {

    #[inline(always)]
    fn from(value: ShaderStage) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    
    #[inline(always)]
    fn from(value: ShaderStage) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl From<ShaderStageFlags> for vk::ShaderStageFlags {

    #[inline(always)]
    fn from(value: ShaderStageFlags) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl From<ShaderStage> for shaderc::ShaderKind {

    fn from(value: ShaderStage) -> Self {
        match value {
            ShaderStage::Vertex => Self::Vertex,
            ShaderStage::TesellationControl => Self::TessControl,
            ShaderStage::TesellationEvaluation => Self::TessEvaluation,
            ShaderStage::Geometry => Self::Geometry,
            ShaderStage::Fragment => Self::Fragment,
            ShaderStage::Compute => Self::Compute,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Display, Clone, Copy, AsRaw, PartialEq, Eq, Hash)]
pub enum DescriptorType {
    /// A type for situations where a supported desriptor type couldn't be determined
    Unknown = -1,
    /// A type associated with a [`Sampler`].
    ///
    /// Descriptors with this descriptor type *must* be written through an [`image write`][1] that
    /// contains a [`Sampler`].
    ///
    /// # Shader declaratios
    /// Glsl: `uniform sampler ...`
    ///
    /// [1]: DescriptorInfos::images
    #[display("sampler")]
    Sampler = vk::DescriptorType::SAMPLER.as_raw(),
    /// A type associated with a sampled image.
    ///
    /// Descriptors with this descriptor type *must* be written through an [`image_write`][1] that
    /// contains an [`ImageId`] to an image that *can* be [`sampled from`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform texture2D ...`
    ///
    /// [1]: DescriptorInfos::images
    /// [2]: ImageUsages::SAMPLED
    #[display("sampled image")]
    SampledImage = vk::DescriptorType::SAMPLED_IMAGE.as_raw(),
    /// A type that combines both a [`Sampler`] and a sampled image.
    ///
    /// Descriptors with this descriptor type *must* be written through an [`image write`][1] that
    /// contains a [`Sampler`] and an [`ImageId`] to an image that *can* be [`sampled from`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform sampler2D ...`
    ///
    /// [1]: DescriptorInfos::images
    /// [2]: ImageUsages::SAMPLED
    #[display("combined image sampler")]
    CombinedImageSampler = vk::DescriptorType::COMBINED_IMAGE_SAMPLER.as_raw(),
    /// A type associated with an image that can be used for load, store and atomic operations.
    ///
    /// Descriptors with this descriptor type *must* be written through an [`image write`][1] that
    /// contains an [`ImageId`] to an image that *can* be used as a [`storage image`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform image2D ...`
    ///
    /// [1]: DescriptorInfos::images
    /// [2]: ImageUsages::STORAGE
    #[display("storage image")]
    StorageImage = vk::DescriptorType::STORAGE_IMAGE.as_raw(),
    /// A type associated with a buffer that can be used for load operations.
    ///
    /// Descriptors with this descriptor type *must* be written through a [`buffer write`][1] that
    /// contains a [`BufferId`] to a buffer that *can* be used as a [`uniform buffer`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform struct UBO { ... } ubo`
    ///
    /// [1]: DescriptorInfos::buffers
    /// [2]: BufferUsages::UNIFORM_BUFFER
    #[display("uniform buffer")]
    UniformBuffer = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
    /// A type associated with a buffer that can be used for load, store and atomic operations.
    ///
    /// Descriptors with this descriptor type *must* be written through a [`buffer write`][1] that
    /// contains a [`BufferId`] to a buffer that *can* be used as a [`storage buffer`][2].
    ///
    /// # Shader declarations
    /// Glsl: `buffer struct SSBO { ... } ssbo`
    ///
    /// [1]: DescriptorInfos::buffers
    /// [2]: BufferUsages::STORAGE_BUFFER
    #[display("storage buffer")]
    StorageBuffer = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
    /// A type associated with a buffer and a buffer view that can be used for image sampling
    /// operations.
    ///
    /// Descriptors with this descriptor type *must* be written through a [`buffer write`][1] that
    /// contains a [`BufferId`] to a buffer that *can* be used as a [`uniform texel buffer`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform samplerBuffer ...`
    ///
    /// [1]: DescriptorInfos::buffers
    /// [2]: BufferUsages::UNIFORM_TEXEL_BUFFER
    #[display("uniform texel buffer")]
    UniformTexelBuffer = vk::DescriptorType::UNIFORM_TEXEL_BUFFER.as_raw(),
    /// A type associated with a buffer and a buffer view that can be used for image load,
    /// store and atomic operations.
    ///
    /// Descriptors with this descriptor type *must* be written through a [`buffer write`][1] that
    /// contains a [`BufferId`] to a buffer that *can* be used as a [`storage texel buffer`][2].
    ///
    /// # Shader declarations
    /// Glsl: `uniform imageBuffer ...`
    ///
    /// [1]: DescriptorInfos::buffers
    /// [2]: BufferUsages::STORAGE_TEXEL_BUFFER
    #[display("storage texel buffer")]
    StorageTexelBuffer = vk::DescriptorType::STORAGE_TEXEL_BUFFER.as_raw(),
    /// A type similar to [`DescriptorType::UniformBuffer`] where it's storage comes directly from
    /// the descriptor set rather than from a separate buffer.
    ///
    /// Descriptors with this descriptor type *must* be written through an
    /// [`inline uniform block write`][1].
    ///
    /// # Shader declarations
    /// Glsl: `uniform struct UBO { ... } ubo`
    ///
    /// [1]: DescriptorInfos::inline_uniform_block
    #[display("inline uniform block")]
    InlineUniformBlock = vk::DescriptorType::INLINE_UNIFORM_BLOCK.as_raw(),
    /// A type used for input attachments, which allows render passes to read earlier rendering
    /// results within the same pass.
    ///
    /// Descriptors with this descriptor type *must* be written through an [`image write`][1] that
    /// contains an [`ImageId`] to an image that *can* be used as an [`input attachment`][2].
    ///
    /// # Shader declarations
    /// Glsl: `layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput ...`
    ///
    /// # Shader load
    /// Glsl: `subpassLoad(subpassInput)`
    ///
    /// [1]: DescriptorInfos::images
    /// [2]: ImageUsages::INPUT_ATTACHMENT
    #[display("input attachment")]
    InputAttachment = vk::DescriptorType::INPUT_ATTACHMENT.as_raw(),
}

/// ``` rust
/// DescriptorType::UniformBuffer |
/// DescriptorType::StorageBuffer |
/// DescriptorType::UniformTexelBuffer |
/// DescriptorType::StorageTexelBuffer
/// ```
#[macro_export]
macro_rules! buffer_descriptor_types {
    () => {
        DescriptorType::UniformBuffer |
        DescriptorType::StorageBuffer |
        DescriptorType::UniformTexelBuffer |
        DescriptorType::StorageTexelBuffer
    };
}

/// ``` rust
/// DescriptorType::Sampler |
/// DescriptorType::SampledImage |
/// DescriptorType::CombinedImageSampler |
/// DescriptorType::StorageImage |
/// DescriptorType::InputAttachment
/// ```
#[macro_export]
macro_rules! image_descriptor_types {
    () => {
        DescriptorType::Sampler |
        DescriptorType::SampledImage |
        DescriptorType::CombinedImageSampler |
        DescriptorType::StorageImage |
        DescriptorType::InputAttachment
    };
}

impl DescriptorType {

    #[inline(always)]
    pub fn is_buffer(self) -> bool {
        matches!(self, buffer_descriptor_types!())
    }

    #[inline(always)]
    pub fn buffer_usage(self) -> Option<BufferUsages> {
        match self {
            Self::UniformBuffer => Some(BufferUsages::UNIFORM_BUFFER),
            Self::StorageBuffer => Some(BufferUsages::STORAGE_BUFFER),
            Self::UniformTexelBuffer => Some(BufferUsages::UNIFORM_TEXEL_BUFFER),
            Self::StorageTexelBuffer => Some(BufferUsages::STORAGE_TEXEL_BUFFER),
            Self::InlineUniformBlock => Some(BufferUsages::empty()),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn requires_buffer(self) -> bool {
        matches!(self,
            Self::UniformBuffer |
            Self::StorageBuffer |
            Self::UniformTexelBuffer |
            Self::StorageTexelBuffer
        )
    }

    #[inline(always)]
    pub fn is_inline_uniform_block(self) -> bool {
        matches!(self, Self::InlineUniformBlock)
    }

    #[inline(always)]
    pub fn is_image(self) -> bool {
        matches!(self, image_descriptor_types!())
    }

    #[inline(always)]
    pub fn image_usage(self) -> Option<ImageUsages> {
        match self {
            Self::Sampler => Some(ImageUsages::empty()),
            Self::SampledImage => Some(ImageUsages::SAMPLED),
            Self::StorageImage => Some(ImageUsages::STORAGE),
            Self::CombinedImageSampler => Some(ImageUsages::SAMPLED),
            Self::InputAttachment => Some(ImageUsages::INPUT_ATTACHMENT),
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
            Self::CombinedImageSampler |
            Self::StorageImage |
            Self::InputAttachment,
        )
    } 

    #[inline(always)]
    pub(crate) fn shader_image_layout(&self) -> Option<ShaderImageLayout> {
        match self {
            Self::SampledImage | Self::CombinedImageSampler => {
                Some(ShaderImageLayout::SampledReadOnly)
            },
            Self::StorageImage => {
                Some(ShaderImageLayout::General(vk::AccessFlags2::SHADER_READ | vk::AccessFlags2::SHADER_WRITE))
            },
            Self::InputAttachment => {
                Some(ShaderImageLayout::Attachment(AttachmentImageLayout::RenderingLocalRead { is_color: true, }))
            },
            _ => None,
        }
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
