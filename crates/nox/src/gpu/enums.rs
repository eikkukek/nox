use core::{
    fmt::{self, Display},
    hash::{self, Hash}
};

use nox_proc::snake_case;

use nox_ash::{
    vk,
    ash_style_enum,
};

use nox_mem::{AsRaw, Display};

use crate::gpu::prelude::*;

pub type Flags32 = u32;
pub type Flags64 = u64;

ash_style_enum! {

    /// Specifies a bitmask of multisample anti-aliasing sample counts
    ///
    /// Default value is [`MSAA::X1`].
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkSampleCountFlagBits.html>
    #[flags(Flags32)]
    #[default = Self::X1]
    pub enum MsaaSamples {
        /// Specifies one sample per pixel.
        #[display("1 sample")]
        X1 = vk::SampleCountFlags::TYPE_1.as_raw(),
        /// Specifies 2 samples per pixel.
        #[display("2 samples")]
        X2 = vk::SampleCountFlags::TYPE_2.as_raw(),
        /// Specifies 4 samples per pixel.
        #[display("4 samples")]
        X4 = vk::SampleCountFlags::TYPE_4.as_raw(),
        /// Specifies 8 samples per pixel.
        #[display("8 samples")]
        X8 = vk::SampleCountFlags::TYPE_8.as_raw(),
        /// Specifies 16 samples per pixel.
        #[display("16 samples")]
        X16 = vk::SampleCountFlags::TYPE_16.as_raw(),
        /// Specifies 32 samples per pixel.
        #[display("32 samples")]
        X32 = vk::SampleCountFlags::TYPE_32.as_raw(),
        /// Specifies 64 samples per pixel.
        #[display("64 samples")]
        X64 = vk::SampleCountFlags::TYPE_64.as_raw(),
    }
    /// Describes what a buffer *can* be used for.
    ///
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferUsageFlagBits.html>
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum BufferUsages {
        /// Specifies that the buffer *can* be used as the source of transfer operations.
        #[display("transfer source")]
        TRANSFER_SRC = vk::BufferUsageFlags::TRANSFER_SRC.as_raw(),
        /// Specifies that the buffer *can* be used as the destination of transfer operations.
        #[display("transfer destination")]
        TRANSFER_DST = vk::BufferUsageFlags::TRANSFER_DST.as_raw(), 
        /// Specifies that the buffer *can* be used as a uniform texel buffer.
        #[display("uniform texel buffer")]
        UNIFORM_TEXEL_BUFFER = vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as a storage texel buffer.
        #[display("storage texel buffer")]
        STORAGE_TEXEL_BUFFER = vk::BufferUsageFlags::STORAGE_TEXEL_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as a uniform buffer.
        #[display("uniform buffer")]
        UNIFORM_BUFFER = vk::BufferUsageFlags::UNIFORM_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as a storage buffer.
        #[display("storage buffer")]
        STORAGE_BUFFER = vk::BufferUsageFlags::STORAGE_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as an index buffer.
        #[display("index buffer")]
        INDEX_BUFFER = vk::BufferUsageFlags::INDEX_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as a vertex buffer.
        #[display("vertex buffer")]
        VERTEX_BUFFER = vk::BufferUsageFlags::VERTEX_BUFFER.as_raw(),
        /// Specifies that the buffer *can* be used as in indirect commands.
        #[display("indirect buffer")]
        INDIRECT_BUFFER = vk::BufferUsageFlags::INDIRECT_BUFFER.as_raw(),
    }
    /// Specifies what an [`Image`] can be used for.
    ///
    /// Default value is [`ImageUsage::empty()`].
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkImageUsageFlagBits.html>
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum ImageUsages {
        /// Specifies that the image *can* be used as the source of transfer operations.
        #[display("transfer source")]
        TRANSFER_SRC = vk::ImageUsageFlags::TRANSFER_SRC.as_raw(),
        /// Specifies that the image *can* be used as the destination of transfer operations.
        #[display("transfer destination")]
        TRANSFER_DST = vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
        /// Specifies that the image *can* be used sampled from in a shader.
        #[display("Sampled")]
        SAMPLED = vk::ImageUsageFlags::SAMPLED.as_raw(),
        /// Specifies that the image *can* be used as a storage image in a shader.
        #[display("Storage")]
        STORAGE = vk::ImageUsageFlags::STORAGE.as_raw(),
        /// Specifies that the image *can* be used as a color attachment in rendering.
        #[display("color attachment")]
        COLOR_ATTACHMENT = vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw(),
        /// Specifies that the image *can* be used as a depth/stencil attachment in rendering.
        #[display("depth stencil attachment")]
        DEPTH_STENCIL_ATTACHMENT = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT.as_raw(),
        /// Specifies that the image *can* be used as an input attachment in rendering.
        #[display("input attachment")]
        INPUT_ATTACHMENT = vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw(),
    }

    /// Specifies which image aspect to use for e.g. [`ImageSubresourceRange`].
    ///
    /// Default value is [`ImageAspects::empty()`].
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkImageAspectFlagBits.html#>
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum ImageAspects {
        #[display("color")]
        COLOR = vk::ImageAspectFlags::COLOR.as_raw(),
        #[display("depth")]
        DEPTH = vk::ImageAspectFlags::DEPTH.as_raw(),
        #[display("stencil")]
        STENCIL = vk::ImageAspectFlags::STENCIL.as_raw(),
        #[display("plane 0")]
        PLANE_0 = vk::ImageAspectFlags::PLANE_0.as_raw(),
        #[display("plane 1")]
        PLANE_1 = vk::ImageAspectFlags::PLANE_1.as_raw(),
        #[display("plane 2")]
        PLANE_2 = vk::ImageAspectFlags::PLANE_2.as_raw(),
    }

    /// Specifies sets of stencil state for which to update operations.
    ///
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkStencilFaceFlagBits.html>
    #[flags(Flags32)]
    pub enum StencilFaces {
        /// Specifies that only the front set of stencil state is updated.
        #[display("front")]
        FRONT = vk::StencilFaceFlags::FRONT.as_raw(),
        /// Specifies that only the back set of stencil state is updated.
        #[display("back")]
        BACK = vk::StencilFaceFlags::BACK.as_raw(),
        /// Specifies that both the front and the back of stencil state is updated.
        #[display("front and back")]
        FRONT_AND_BACK = vk::StencilFaceFlags::FRONT_AND_BACK.as_raw(),
    }
   
    /// Bitmask controlling triangle culling.
    ///
    /// Default is [`CullModeFlags::FRONT`].
    /// # Vulkan docs
    /// <https://docs.vulkan.org/refpages/latest/refpages/source/VkCullModeFlagBits.html>
    #[flags(Flags32)]
    #[default = Self::FRONT]
    pub enum CullModes {
        /// Specifies that no triangles are discarded.
        #[display("none")]
        NONE = vk::CullModeFlags::NONE.as_raw(),
        /// Specifies that front-facing triangles are discarded.
        ///
        /// Front face is defined by [`FrontFace`].
        #[display("front")]
        FRONT = vk::CullModeFlags::FRONT.as_raw(),
        /// Specifies that back-facing triangles are discarded.
        ///
        /// Front face is defined by [`FrontFace`].
        #[display("back")]
        BACK = vk::CullModeFlags::BACK.as_raw(),
        /// Specifies that all triangles are discarded.
        #[display("front and back")]
        FRONT_AND_BACK = vk::CullModeFlags::FRONT_AND_BACK.as_raw(),
    }

    /// Bitmask specifying what features are supported by a format.
    #[flags(u64)]
    pub enum FormatFeatures {
        /// Specifies that an image view *can* be sampled from.
        #[display("sampled image")]
        SAMPLED_IMAGE = vk::FormatFeatureFlags2::SAMPLED_IMAGE.as_raw(),
        /// Specifies that an image view *can* be used as a storage image.
        #[display("storage image")]
        STORAGE_IMAGE = vk::FormatFeatureFlags2::STORAGE_IMAGE.as_raw(),
        /// Specifies that an image view *can* be used as a storage image that supports atomic
        /// operations.
        #[display("storage image atomic")]
        STORAGE_IMAGE_ATOMIC = vk::FormatFeatureFlags2::STORAGE_IMAGE_ATOMIC.as_raw(),
        /// Specifies that the format *can* be used as a vertex attribute format.
        #[display("vertex buffer")]
        VERTEX_BUFFER = vk::FormatFeatureFlags2::VERTEX_BUFFER.as_raw(),
        /// Specifies that an image view *can* be used as a color attachment.
        #[display("color attachment")]
        COLOR_ATTACHMENT = vk::FormatFeatureFlags2::COLOR_ATTACHMENT.as_raw(),
        /// Specifies that an image view *can* be used as a color attachment that supports blending.
        #[display("color attachment blend")]
        COLOR_ATTACHMENT_BLEND = vk::FormatFeatureFlags2::COLOR_ATTACHMENT_BLEND.as_raw(),
        /// Specifies that an image view *can* be used as a depth/stencil attachment and as
        /// an input attachment.
        #[display("depth stencil attachment")]
        DEPTH_STENCIL_ATTACHMENT = vk::FormatFeatureFlags2::DEPTH_STENCIL_ATTACHMENT.as_raw(),
        /// Specifies an image *can* be used as the source of a blitting.
        #[display("blit source")]
        BLIT_SRC = vk::FormatFeatureFlags2::BLIT_SRC.as_raw(),
        /// Specifies an image *can* be used as the destination of a blitting.
        #[display("blit destination")]
        BLIT_DST = vk::FormatFeatureFlags2::BLIT_DST.as_raw(),
        /// Specifies that an image *can* be sampled from with a linear [`Filter`].
        #[display("sampled image filter linear")]
        SAMPLED_IMAGE_FILTER_LINEAR = vk::FormatFeatureFlags2::SAMPLED_IMAGE_FILTER_LINEAR.as_raw(),
        /// Specifies that an image *can* be used as the source image of copy commands.
        #[display("transfer source")]
        TRANSFER_SRC = vk::FormatFeatureFlags2::TRANSFER_SRC.as_raw(),
        /// Specifies that an image *can* be used as the destionation image of copy commands and
        /// clear commands.
        #[display("transfer destination")]
        TRANSFER_DST = vk::FormatFeatureFlags2::TRANSFER_DST.as_raw(),
    }

    #[flags(Flags32)]
    #[default = Self::NONE]
    pub enum ResolveModes {
        #[display("none")]
        NONE = vk::ResolveModeFlags::NONE.as_raw(),
        #[display("sample zero")]
        SAMPLE_ZERO = vk::ResolveModeFlags::SAMPLE_ZERO.as_raw(),
        #[display("average")]
        AVERAGE = vk::ResolveModeFlags::AVERAGE.as_raw(),
        #[display("min")]
        MIN = vk::ResolveModeFlags::MIN.as_raw(),
        #[display("max")]
        MAX = vk::ResolveModeFlags::MAX.as_raw(),
    }

    #[flags(Flags32)]
    #[default = Self::RGBA]
    pub enum ColorComponents {
        #[display("red")]
        R = vk::ColorComponentFlags::R.as_raw(),
        #[display("green")]
        G = vk::ColorComponentFlags::G.as_raw(),
        #[display("blue")]
        B = vk::ColorComponentFlags::B.as_raw(),
        #[display("alpha")]
        A = vk::ColorComponentFlags::A.as_raw(),
        #[display("rgba")]
        RGBA =
            Self::R.as_raw() |
            Self::G.as_raw() |
            Self::B.as_raw() |
            Self::A.as_raw(),
    }

    #[flags(Flags32)]
    pub enum QueueFlags {
        /// Specifies that the queue supports graphics operations.
        ///
        /// # Supports commands
        /// - [`CopyCommands`]
        /// - [`GraphicsCommands`]
        /// - [`DrawCommands`]
        #[display("graphics")]
        GRAPHICS = vk::QueueFlags::GRAPHICS.as_raw(),
        /// Specifies that the queue supports compute operations.
        ///
        /// # Supports commands
        /// - [`ComputeCommands`]
        #[display("compute")]
        COMPUTE = vk::QueueFlags::COMPUTE.as_raw(),
    }
}

impl ImageAspects {

    pub fn plane(self) -> Option<u32> {
        if self == Self::PLANE_0 {
            Some(0)
        } else if self == Self::PLANE_1 {
            Some(1)
        } else if self == Self::PLANE_2 {
            Some(2)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum PhysicalDeviceType {
    #[display("other")]
    Other,
    #[display("integrated gpu")]
    IntegratedGpu,
    #[display("discrete gpu")]
    DiscreteGpu,
    #[display("virtual gpu")]
    VirtualGpu,
    #[display("cpu")]
    Cpu,
}

impl From<vk::PhysicalDeviceType> for PhysicalDeviceType {

    #[inline(always)]
    fn from(value: vk::PhysicalDeviceType) -> Self {
        match value {
            vk::PhysicalDeviceType::OTHER => Self::Other,
            vk::PhysicalDeviceType::INTEGRATED_GPU => Self::IntegratedGpu,
            vk::PhysicalDeviceType::DISCRETE_GPU => Self::DiscreteGpu,
            vk::PhysicalDeviceType::VIRTUAL_GPU => Self::VirtualGpu,
            vk::PhysicalDeviceType::CPU => Self::Cpu,
            _ => Self::Other,
        }
    }
}

/// Specifies how a component is swizzled
///
/// Default value is [`ComponentSwizzle::Identity`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkComponentSwizzle.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum ComponentSwizzle {
    #[default]
    #[display("identity")]
    Identity = vk::ComponentSwizzle::IDENTITY.as_raw(),
    #[display("zero")]
    Zero = vk::ComponentSwizzle::ZERO.as_raw(),
    #[display("one")]
    One = vk::ComponentSwizzle::ONE.as_raw(),
    #[display("r")]
    R = vk::ComponentSwizzle::R.as_raw(),
    #[display("g")]
    G = vk::ComponentSwizzle::G.as_raw(),
    #[display("b")]
    B = vk::ComponentSwizzle::B.as_raw(),
    #[display("a")]
    A = vk::ComponentSwizzle::A.as_raw(),
}

/// Specifies filters used for texture lookups.
///
/// Default value is [`Filter::Nearest`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkFilter.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum Filter {
    /// Specifies nearest filtering.
    #[default]
    #[display("nearest")]
    Nearest = vk::Filter::NEAREST.as_raw(),
    /// Specifies linear filtering.
    #[display("linear")]
    Linear = vk::Filter::LINEAR.as_raw(),
}

/// Specifies mipmap mode used for texture lookups.
///
/// Default value is [`MipmapMode::Nearest`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerMipmapMode.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum MipmapMode {
    /// Specifies nearest filtering.
    #[default]
    #[display("nearest")]
    Nearest = vk::SamplerMipmapMode::NEAREST.as_raw(),
    /// Specifies linear filtering.
    #[display("linear")]
    Linear = vk::SamplerMipmapMode::LINEAR.as_raw(),
}

/// Specifies behaviour of sampling with texture coordinates outside an image.
///
/// Default value is [`SamplerAddressMode::Repeat`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerAddressMode.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum SamplerAddressMode {
    /// Specifies repeat wrap mode.
    #[default]
    #[display("repeat")]
    Repeat = vk::SamplerAddressMode::REPEAT.as_raw(),
    /// Specifies mirrored repeat wrap mode.
    #[display("mirrored repeat")]
    MirroredRepeat = vk::SamplerAddressMode::MIRRORED_REPEAT.as_raw(),
    /// Specifies clamp to edge wrap mode.
    #[display("clamp to edge")]
    ClampToEdge = vk::SamplerAddressMode::CLAMP_TO_EDGE.as_raw(),
    /// Specifies clamp to border wrap mode.
    #[display("clamp to border")]
    ClampToBorder = vk::SamplerAddressMode::CLAMP_TO_BORDER.as_raw(),
}


/// Specifies the border color used for texture lookup.
///
/// Default value is [`BorderColor::FloatTransparentBlack`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkBorderColor.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum BorderColor {
    #[default]
    #[display("float transparent black")]
    FloatTransparentBlack = vk::BorderColor::FLOAT_TRANSPARENT_BLACK.as_raw(),
    #[display("int transparent black")]
    IntTransparentBlack = vk::BorderColor::INT_TRANSPARENT_BLACK.as_raw(),
    #[display("float opaque black")]
    FloatOpaqueBlack = vk::BorderColor::FLOAT_OPAQUE_BLACK.as_raw(),
    #[display("int opaque black")]
    IntOpaqueBlack = vk::BorderColor::INT_OPAQUE_BLACK.as_raw(),
    #[display("float opaque white")]
    FloatOpaqueWhite = vk::BorderColor::FLOAT_OPAQUE_WHITE.as_raw(),
    #[display("int opaque white")]
    IntOpaqueWhite = vk::BorderColor::INT_OPAQUE_WHITE.as_raw(),
}

/// Specifies comparison operator for depth, stencil and sampler operations.
///
/// Default value is [`CompareOp::Never`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkCompareOp.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum CompareOp {
    #[default]
    #[display("never")]
    Never = vk::CompareOp::NEVER.as_raw(),
    #[display("less")]
    Less = vk::CompareOp::LESS.as_raw(),
    #[display("equal")]
    Equal = vk::CompareOp::EQUAL.as_raw(),
    #[display("less or equal")]
    LessOrEqual = vk::CompareOp::LESS_OR_EQUAL.as_raw(),
    #[display("greater")]
    Greater = vk::CompareOp::GREATER.as_raw(),
    #[display("not equal")]
    NotEqual = vk::CompareOp::NOT_EQUAL.as_raw(),
    #[display("greater or equal")]
    GreaterOrEqual = vk::CompareOp::GREATER_OR_EQUAL.as_raw(),
    #[display("always")]
    Always = vk::CompareOp::ALWAYS.as_raw(),
}

/// Specifies the type of indices in an index buffer.
///
/// The default value is [`IndexType::U32`].
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkIndexType.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum IndexType {
    /// Specifies indices that are 8-bit unsigned integer values.
    ///
    /// Note that this requires enabling [`ext::index_type_uint8`] device extension.
    #[default]
    #[display("u8")]
    U8 = vk::IndexType::UINT8.as_raw(),
    /// Specifies indices that are 16-bit unsigned integer values.
    #[display("u16")]
    U16 = vk::IndexType::UINT16.as_raw(),
    /// Specifies indices that are 32-bit unsigned integer values.
    #[display("u32")]
    U32 = vk::IndexType::UINT32.as_raw(),
}

impl IndexType {

    pub fn index_size(self) -> vk::DeviceSize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
        }
    }
}

/// Specifies the robustness of buffer accesses in a pipeline.
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessBufferBehavior.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum PipelineRobustnessBufferBehavior {
    /// Specifies that out of bounds buffer accesses follow the behavior of robust buffer access
    /// features enabled for the device.
    #[default]
    #[display("device default")]
    DeviceDefault = vk::PipelineRobustnessBufferBehavior::DEVICE_DEFAULT.as_raw(),
    /// Specifies that buffer accesses *must* not be out of bounds.
    #[display("disabled")]
    Disabled = vk::PipelineRobustnessBufferBehavior::DISABLED.as_raw(),
    /// Specifies that bounds checks to shader buffers are performed.
    ///
    /// Out of bounds reads will either return zero values or values from the underlying
    /// [`DeviceMemory`] bound to the buffer, including bytes outside the buffer itself.
    ///
    /// Out of bounds writes will either be discarded, or write values to the underlying
    /// [`DeciceMemory`] bound to the buffer including outside the buffer's range.
    ///
    /// Atomic read-modify-write operations will behave the same as writes outside bounds,
    /// but will return *undefined* values.
    #[display("robust buffer access")]
    RobustBufferAccess = vk::PipelineRobustnessBufferBehavior::ROBUST_BUFFER_ACCESS.as_raw(),
    /// Specifies that stricter bounds checks to shader buffers are performed.
    ///
    /// Out of bounds reads will produce zero values (with some caveats described in the docs).
    ///
    /// Out of bounds writes will not modify any memory.
    ///
    /// Atomic read-modify-write operations will behave the same as writes outside bounds, but
    /// will return *undefined* values.
    #[display("robust buffer access 2")]
    RobustBufferAccess2 = vk::PipelineRobustnessBufferBehavior::ROBUST_BUFFER_ACCESS_2.as_raw(),
}


/// Specifies the robustness of image accesses in a pipeline.
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessImageBehavior.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum PipelineRobustnessImageBehavior {
    /// Specifies that out of bounds image accesses follow the behavior of robust image access features
    /// enabled for the device.
    #[default]
    #[display("device default")]
    DeviceDefault = vk::PipelineRobustnessImageBehavior::DEVICE_DEFAULT.as_raw(),
    /// Specifies that image accesses *must* not be out of bounds.
    #[display("disabled")]
    Disabled = vk::PipelineRobustnessImageBehavior::DISABLED.as_raw(),
    /// Specifies that out of bounds checks to shader images are performed.
    ///
    /// Out of bounds writes and atomic read-modify-write operations will not modify any
    /// memory.
    ///
    /// Reads, atomic read-modify-write operations, or fetches from images outside bounds will
    /// return zero values with (0,0,1) or (0,0,0) values inserted for missing G, B or A
    /// components based on the format.
    #[display("robust image access")]
    RobustImageAccess = vk::PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS.as_raw(),
    /// Specifies that out of bounds checks to shader images are performed.
    ///
    /// Out of bounds writes and atomic read-modify-write operations will not modify any
    /// memory.
    ///
    /// Reads, atomic read-modify-write operations, or fetches from images outside bounds will
    /// return zero values with (0,0,1) values inserted for missing G, B or A components based
    /// on the format.
    #[display("robust image access 2")]
    RobustImageAccess2 = vk::PipelineRobustnessImageBehavior::ROBUST_IMAGE_ACCESS_2.as_raw(),
}

/// Describes what parts of a pipeline can (and must) be dynamically changed.
///
/// Doesn't include viewport or scissor since they are always enabled.
///
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkDynamicState.html>
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug, Display)]
pub enum DynamicState {
    #[display("line width")]
    LineWidth = vk::DynamicState::LINE_WIDTH.as_raw(),
    #[display("depth bias")]
    DepthBias = vk::DynamicState::DEPTH_BIAS.as_raw(),
    #[display("blend constants")]
    BlendConstants = vk::DynamicState::BLEND_CONSTANTS.as_raw(),
    #[display("depth bounds")]
    DepthBounds = vk::DynamicState::DEPTH_BOUNDS.as_raw(),
    #[display("stencil compare Mask")]
    StencilCompareMask = vk::DynamicState::STENCIL_COMPARE_MASK.as_raw(),
    #[display("stencil write mask")]
    StencilWriteMask = vk::DynamicState::STENCIL_WRITE_MASK.as_raw(),
    #[display("stencil reference")]
    StencilReference = vk::DynamicState::STENCIL_REFERENCE.as_raw(),
    #[display("cull mode")]
    CullMode = vk::DynamicState::CULL_MODE.as_raw(),
    #[display("front face")]
    FrontFace = vk::DynamicState::FRONT_FACE.as_raw(),
    #[display("primitive topology")]
    PrimitiveTopology = vk::DynamicState::PRIMITIVE_TOPOLOGY.as_raw(),
    #[display("vertex input binding stride")]
    VertexInputBindingStride = vk::DynamicState::VERTEX_INPUT_BINDING_STRIDE.as_raw(),
    #[display("depth test enable")]
    DepthTestEnable =  vk::DynamicState::DEPTH_TEST_ENABLE.as_raw(),
    #[display("depth write enable")]
    DepthWriteEnable = vk::DynamicState::DEPTH_WRITE_ENABLE.as_raw(),
    #[display("depth compare operation")]
    DepthCompareOp = vk::DynamicState::DEPTH_COMPARE_OP.as_raw(),
    #[display("depth bounds test enable")]
    DepthBoundsTestEnable = vk::DynamicState::DEPTH_BOUNDS_TEST_ENABLE.as_raw(),
    #[display("stencil test enable")]
    StencilTestEnable = vk::DynamicState::STENCIL_TEST_ENABLE.as_raw(),
    #[display("stencil operation")]
    StencilOp = vk::DynamicState::STENCIL_OP.as_raw(),
    #[display("rasterizer discard enable")]
    RasterizerDiscardEnable = vk::DynamicState::RASTERIZER_DISCARD_ENABLE.as_raw(),
    #[display("depth bias enable")]
    DepthBiasEnable = vk::DynamicState::DEPTH_BIAS_ENABLE.as_raw(),
    #[display("primitive restart enable")]
    PrimitiveRestartEnable = vk::DynamicState::PRIMITIVE_RESTART_ENABLE.as_raw(),
}

/// Specifies polygon front-facing orientation.
///
/// The default value is [`FrontFace::CounterClockwise`].
/// 
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkFrontFace.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, AsRaw, Debug, Display)]
pub enum FrontFace {
    /// Specifies that triangles with positive area are considered front-facing.
    #[display("counter clockwise")]
    #[default]
    CounterClockwise = vk::FrontFace::COUNTER_CLOCKWISE.as_raw(),
    /// Specifies that triangles with negative area are considered front-facing.
    #[display("clockwise")]
    ClockWise = vk::FrontFace::CLOCKWISE.as_raw(),
}

/// Specifies primitive topology.
///
/// The default is [`PrimitiveTopology::TriangleList`].
///
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkPrimitiveTopology.html>
#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, AsRaw, Debug, Display)]
pub enum PrimitiveTopology {
    /// Specifies a series of separate point primities.
    #[display("point list")]
    PointList = vk::PrimitiveTopology::POINT_LIST.as_raw(),
    /// Specifies a series of separate line primities.
    #[display("line list")]
    LineList = vk::PrimitiveTopology::LINE_LIST.as_raw(),
    /// Specifies a series of connected line primities.
    #[display("line strip")]
    LineStrip = vk::PrimitiveTopology::LINE_STRIP.as_raw(),
    /// Specifies a series of separate triangle primities.
    #[display("triangle list")]
    #[default]
    TriangleList = vk::PrimitiveTopology::TRIANGLE_LIST.as_raw(),
    /// Specifies a series of connected triangle primities.
    #[display("triangle strip")]
    TriangleStrip = vk::PrimitiveTopology::TRIANGLE_STRIP.as_raw(),
    /// Specifies a series of connected triangle primitives with all triangles sharing a common
    /// vertex.
    #[display("triangle fan")]
    TriangleFan = vk::PrimitiveTopology::TRIANGLE_FAN.as_raw(),
    /// Specifies a series of separate line primitives with adjacency.
    #[display("line list with adjacency")]
    LineListWithAdjacency = vk::PrimitiveTopology::LINE_LIST_WITH_ADJACENCY.as_raw(),
    /// Specifies a series of connected line primitives with adjacency, with consecutive
    /// primitives sharing three vertices.
    #[display("line strip with adjacency")]
    LineStripWithAdjacency = vk::PrimitiveTopology::LINE_STRIP_WITH_ADJACENCY.as_raw(),
    /// Specifies a series of separate triangle primitives with adjacency.
    #[display("triangle list with adjacency")]
    TriangleListWithAdjacency = vk::PrimitiveTopology::TRIANGLE_LIST_WITH_ADJACENCY.as_raw(),
    /// Specifies connected triangle primitives with adjacency, with consecutive triangles sharing an edge.
    #[display("triangle strip with adjacency")]
    TriangleStripWithAdjacency = vk::PrimitiveTopology::TRIANGLE_STRIP_WITH_ADJACENCY.as_raw(),
    /// Specifies separate patch primitives.
    #[display("patch list")]
    PatchList = vk::PrimitiveTopology::PATCH_LIST.as_raw(),
}

impl PrimitiveTopology {

    #[inline(always)]
    pub fn can_restart(self) -> bool {
        matches!(self,
            Self::LineStrip | Self::TriangleStrip |
            Self::LineStripWithAdjacency  | Self::TriangleStripWithAdjacency
        )
    }
}

/// Stencil comparison function.
///
/// # Vulkan docs
/// <https://docs.vulkan.org/refpages/latest/refpages/source/VkStencilOp.html>
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, AsRaw, Debug, Display)]
pub enum StencilOp {
    /// Keeps the current value.
    #[display("keep")]
    Keep = vk::StencilOp::KEEP.as_raw(),
    /// Sets the value to 0.
    #[display("zero")]
    Zero = vk::StencilOp::ZERO.as_raw(),
    /// Sets the value to the reference.
    #[display("replace")]
    Replace = vk::StencilOp::REPLACE.as_raw(),
    /// Increments the current value, saturating at the maximum representable unsigned value.
    #[display("saturating increment")]
    SaturatingIncrement = vk::StencilOp::INCREMENT_AND_CLAMP.as_raw(),
    /// Increments the current value, saturating at 0.
    #[display("saturating decrement")]
    SaturatinDecrement = vk::StencilOp::DECREMENT_AND_CLAMP.as_raw(),
    /// Bitwise-inverts the current value.
    #[display("invert")]
    Invert = vk::StencilOp::INVERT.as_raw(),
    /// Increments the current value, wrapping around at the maximum representable unsigned
    /// value.
    #[display("wrapping increment")]
    WrappingIncrement = vk::StencilOp::INCREMENT_AND_WRAP.as_raw(),
    /// Decrements the current value, wrapping around at 0.
    #[display("wrapping decrement")]
    WrappingDecrement = vk::StencilOp::DECREMENT_AND_WRAP.as_raw(),
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum PolygonMode {
    #[default]
    Fill = vk::PolygonMode::FILL.as_raw(),
    Line = vk::PolygonMode::LINE.as_raw(),
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum BlendFactor {
    Zero = vk::BlendFactor::ZERO.as_raw(),
    One = vk::BlendFactor::ONE.as_raw(),
    SrcColor = vk::BlendFactor::SRC_COLOR.as_raw(),
    OneMinusSrcColor = vk::BlendFactor::ONE_MINUS_SRC_COLOR.as_raw(),
    DstColor = vk::BlendFactor::DST_COLOR.as_raw(),
    OneMinusDstColor = vk::BlendFactor::ONE_MINUS_DST_COLOR.as_raw(),
    SrcAlpha = vk::BlendFactor::SRC_ALPHA.as_raw(),
    OneMinusSrcAlpha = vk::BlendFactor::ONE_MINUS_SRC_ALPHA.as_raw(),
    DstAlpha = vk::BlendFactor::DST_ALPHA.as_raw(),
    OneMinusDstAlpha = vk::BlendFactor::ONE_MINUS_DST_ALPHA.as_raw(),
    ConstColor = vk::BlendFactor::CONSTANT_COLOR.as_raw(),
    OneMinusConstColor = vk::BlendFactor::ONE_MINUS_CONSTANT_COLOR.as_raw(),
    ConstAlpha = vk::BlendFactor::CONSTANT_ALPHA.as_raw(),
    OneMinusConstAlpha = vk::BlendFactor::ONE_MINUS_CONSTANT_ALPHA.as_raw(),
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum BlendOp {
    Add = vk::BlendOp::ADD.as_raw(),
    Sub = vk::BlendOp::SUBTRACT.as_raw(),
    SubRev = vk::BlendOp::REVERSE_SUBTRACT.as_raw(),
    Min = vk::BlendOp::MIN.as_raw(),
    Max = vk::BlendOp::MAX.as_raw(),
}

macro_rules! impl_convert_vk {
    ($([$name:ident, vk::$vk:ident]),+ $(,)?) => {
        $(
            impl From<$name> for vk::$vk {

                #[inline(always)]
                fn from(value: $name) -> Self {
                    Self::from_raw(value.as_raw())
                }
            }
        )+
    };
}

impl_convert_vk! {
    [MsaaSamples, vk::SampleCountFlags],
    [BufferUsages, vk::BufferUsageFlags],
    [ImageUsages, vk::ImageUsageFlags],
    [ImageAspects, vk::ImageAspectFlags],
    [ComponentSwizzle, vk::ComponentSwizzle],
    [Filter, vk::Filter],
    [MipmapMode, vk::SamplerMipmapMode],
    [SamplerAddressMode, vk::SamplerAddressMode],
    [BorderColor, vk::BorderColor],
    [BlendFactor, vk::BlendFactor],
    [BlendOp, vk::BlendOp],
    [CompareOp, vk::CompareOp],
    [StencilFaces, vk::StencilFaceFlags],
    [StencilOp, vk::StencilOp],
    [PolygonMode, vk::PolygonMode],
    [DynamicState, vk::DynamicState],
    [FrontFace, vk::FrontFace],
    [PrimitiveTopology, vk::PrimitiveTopology],
    [IndexType, vk::IndexType],
    [PipelineRobustnessBufferBehavior, vk::PipelineRobustnessBufferBehavior],
    [PipelineRobustnessImageBehavior, vk::PipelineRobustnessImageBehavior],
    [ResolveModes, vk::ResolveModeFlags],
    [ColorComponents, vk::ColorComponentFlags],
    [CullModes, vk::CullModeFlags],
}

impl From<vk::SampleCountFlags> for MsaaSamples {

    #[inline(always)]
    fn from(value: vk::SampleCountFlags) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[derive(Clone, Copy, Debug, Display)]
pub enum ResolveAspect {
    #[display("Color")]
    Color,
    #[display("Depth")]
    Depth,
    #[display("Stencil")]
    Stencil,
}

/// An enumeration of all Vulkan format compatiblity classes guaranteed to be available to Nox.
///
/// # Vulkan docs
/// <https://docs.vulkan.org/spec/latest/chapters/formats.html#formats-compatibility-classes>
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum FormatType {
    #[display("undefined")]
    Undefined,
    #[display("class 8-bit")]
    Class8Bit,
    #[display("class 8-bit alpha")]
    Class8BitAlpha,
    #[display("class 16-bit")]
    Class16Bit,
    #[display("class 24-bit")]
    Class24Bit,
    #[display("class 32-bit")]
    Class32Bit,
    #[display("class 48-bit")]
    Class48Bit,
    #[display("class 64-bit")]
    Class64Bit,
    #[display("class 96-bit")]
    Class96Bit,
    #[display("class 128-bit")]
    Class128Bit,
    #[display("class 192-bit")]
    Class192Bit,
    #[display("class 256-bit")]
    Class256Bit,
    #[display("class D16")]
    ClassD16,
    #[display("class D24")]
    ClassD24,
    #[display("class D32")]
    ClassD32,
    #[display("class S8")]
    ClassS8,
    #[display("class D16S8")]
    ClassD16s8,
    #[display("class D24S8")]
    ClassD24s8,
    #[display("class D32S8")]
    ClassD32s8,
    #[display("class BC1_RGB")]
    ClassBc1Rgb,
    #[display("class BC1_RGBA")]
    ClassBc1Rgba,
    #[display("class BC2")]
    ClassBc2,
    #[display("class BC3")]
    ClassBc3,
    #[display("class BC4")]
    ClassBc4,
    #[display("class BC5")]
    ClassBc5,
    #[display("class BC6H")]
    ClassBc6h,
    #[display("class BC7")]
    ClassBc7,
    #[display("class ETC2_RGB")]
    ClassEtc2Rgb,
    #[display("class ETC2_RGBA")]
    ClassEtc2Rgba,
    #[display("class ETC2_EAC_RGBA")]
    ClassEtc2EacRgba,
    #[display("class EAC_R")]
    ClassEacR,
    #[display("class EAC_RG")]
    ClassEacRG,
    #[display("class ASTC_4x4")]
    ClassAstc4x4,
    #[display("class ASTC_5x4")]
    ClassAstc5x4,
    #[display("class ASTC_5x5")]
    ClassAstc5x5,
    #[display("class ASTC_6x5")]
    ClassAstc6x5,
    #[display("class ASTC_6x6")]
    ClassAstc6x6,
    #[display("class ASTC_8x5")]
    ClassAstc8x5,
    #[display("class ASTC_8x6")]
    ClassAstc8x6,
    #[display("class ASTC_8x8")]
    ClassAstc8x8,
    #[display("class ASTC_10x5")]
    ClassAstc10x5,
    #[display("class ASTC_10x6")]
    ClassAstc10x6,
    #[display("class ASTC_10x8")]
    ClassAstc10x8,
    #[display("class ASTC_10x10")]
    ClassAstc10x10,
    #[display("class ASTC_12x10")]
    ClassAstc12x10,
    #[display("class ASTC_12x12")]
    ClassAstc12x12,
    #[display("class 32-bit G8B8G8R8")]
    Class32BitG8b8g8r8,
    #[display("class 32-bit B8G8R8G8")]
    Class32BitB8g8r8g8,
    #[display("class 8-bit 3-plane 420")]
    Class8Bit3Plane420,
    #[display("class 8-bit 2-plane 420")]
    Class8Bit2Plane420,
    #[display("class 8-bit 3-plane 422")]
    Class8Bit3Plane422,
    #[display("class 8-bit 2-plane 422")]
    Class8Bit2Plane422,
    #[display("class 8-bit 3-plane 444")]
    Class8Bit3Plane444,
    #[display("class 64-bit R10G10B10A10")]
    Class64BitR10g10b10a10,
    #[display("class 64-bit G10B10G10R10")]
    Class64BitG10b10g10r10,
    #[display("class 64-bit B10G10R10G10")]
    Class64BitB10g10r10g10,
    #[display("class 10-bit 3-plane 420")]
    Class10Bit3Plane420,
    #[display("class 10-bit 2-plane 420")]
    Class10Bit2Plane420,
    #[display("class 10-bit 3-plane 422")]
    Class10Bit3Plane422,
    #[display("class 10-bit 2-plane 422")]
    Class10Bit2Plane422,
    #[display("class 10-bit 3-plane 444")]
    Class10Bit3Plane444,
    #[display("class 64-bit R12G12B12A12")]
    Class64BitR12g12b12a12,
    #[display("class 64-bit G12B12G12R12")]
    Class64BitG12b12g12r12,
    #[display("class 64-bit B12G12R12G12")]
    Class64BitB12g12r12g12,
    #[display("class 12-bit 3-plane 420")]
    Class12Bit3Plane420,
    #[display("class 12-bit 2-plane 420")]
    Class12Bit2Plane420,
    #[display("class 12-bit 3-plane 422")]
    Class12Bit3Plane422,
    #[display("class 12-bit 2-plane 422")]
    Class12Bit2Plane422,
    #[display("class 12-bit 3-plane 444")]
    Class12Bit3Plane444,
    #[display("class 64-bit G16B16G16R16")]
    Class64BitG16b16g16r16,
    #[display("class 64-bit B16G16R16G16")]
    Class64BitB16g16r16g16,
    #[display("class 16-bit 3-plane 420")]
    Class16Bit3Plane420,
    #[display("class 16-bit 2-plane 420")]
    Class16Bit2Plane420,
    #[display("class 16-bit 3-plane 422")]
    Class16Bit3Plane422,
    #[display("class 16-bit 2-plane 422")]
    Class16Bit2Plane422,
    #[display("class 16-bit 3-plane 444")]
    Class16Bit3Plane444,
    #[display("class 8-bit 2-plane 444")]
    Class8Bit2Plane444,
    #[display("class 10-bit 2-plane 444")]
    Class10Bit2Plane444,
    #[display("class 12-bit 2-plane 444")]
    Class12Bit2Plane444,
    #[display("class 16-bit 2-plane 444")]
    Class16Bit2Plane444,
}

/// Metadata about a [`FormatType`].
///
/// You query this with [`Format::compatibility`].
#[derive(Clone, Copy, Debug, Display)]
#[display("{class}")]
pub struct FormatCompatibilityClass {
    class: FormatType,
    texel_block_size: DeviceSize,
    texel_block_extent: Dimensions,
    texels_per_block: u8,
}

impl FormatCompatibilityClass {  

    #[inline(always)]
    pub const fn texel_block_size(&self) -> DeviceSize {
        self.texel_block_size
    }

    #[inline(always)]
    pub const fn texel_block_extent(&self) -> Dimensions {
        self.texel_block_extent
    }

    #[inline(always)]
    pub const fn texels_per_block(&self) -> u8 {
        self.texels_per_block
    }


    #[inline(always)]
    pub const fn is_size_compatible(&self, other: &Self) -> bool {
        self.texel_block_size() == other.texel_block_size()
    }

    #[inline(always)]
    pub const fn is_compressed(&self) -> bool {
        use FormatType as F;
        matches!(self.class,
            F::ClassBc1Rgb |
            F::ClassBc1Rgba |
            F::ClassBc2 |
            F::ClassBc3 |
            F::ClassBc4 |
            F::ClassBc5 |
            F::ClassBc6h |
            F::ClassBc7 |
            F::ClassEtc2Rgb |
            F::ClassEtc2Rgba |
            F::ClassEtc2EacRgba |
            F::ClassEacR |
            F::ClassEacRG |
            F::ClassAstc4x4 |
            F::ClassAstc5x4 |
            F::ClassAstc5x5 |
            F::ClassAstc6x5 |
            F::ClassAstc6x6 |
            F::ClassAstc8x5 |
            F::ClassAstc8x6 |
            F::ClassAstc8x8 |
            F::ClassAstc10x5 |
            F::ClassAstc10x6 |
            F::ClassAstc10x8 |
            F::ClassAstc10x10 |
            F::ClassAstc12x10 |
            F::ClassAstc12x12
        )
    }

    snake_case! {

        const [!Undefined!]: Self = Self::new(FormatType::Undefined);
        const [!Class8Bit!]: Self = Self::new(FormatType::Class8Bit);
        const [!Class16Bit!]: Self = Self::new(FormatType::Class16Bit);
        const [!Class8BitAlpha!]: Self = Self::new(FormatType::Class8BitAlpha);
        const [!Class24Bit!]: Self = Self::new(FormatType::Class24Bit);
        const [!Class32Bit!]: Self = Self::new(FormatType::Class32Bit);
        const [!Class48Bit!]: Self = Self::new(FormatType::Class48Bit);
        const [!Class64Bit!]: Self = Self::new(FormatType::Class64Bit);
        const [!Class96Bit!]: Self = Self::new(FormatType::Class96Bit);
        const [!Class128Bit!]: Self = Self::new(FormatType::Class128Bit);
        const [!Class192Bit!]: Self = Self::new(FormatType::Class192Bit);
        const [!Class256Bit!]: Self = Self::new(FormatType::Class256Bit);
        const [!ClassD16!]: Self = Self::new(FormatType::ClassD16);
        const [!ClassD24!]: Self = Self::new(FormatType::ClassD24);
        const [!ClassD32!]: Self = Self::new(FormatType::ClassD32);
        const [!ClassS8!]: Self = Self::new(FormatType::ClassS8);
        const [!ClassD16s8!]: Self = Self::new(FormatType::ClassD16s8);
        const [!ClassD24s8!]: Self = Self::new(FormatType::ClassD24s8);
        const [!ClassD32s8!]: Self = Self::new(FormatType::ClassD32s8);
        const [!ClassBc1Rgb!]: Self = Self::new(FormatType::ClassBc1Rgb);
        const [!ClassBc1Rgba!]: Self = Self::new(FormatType::ClassBc1Rgba);
        const [!ClassBc2!]: Self = Self::new(FormatType::ClassBc2);
        const [!ClassBc3!]: Self = Self::new(FormatType::ClassBc3);
        const [!ClassBc4!]: Self = Self::new(FormatType::ClassBc4);
        const [!ClassBc5!]: Self = Self::new(FormatType::ClassBc5);
        const [!ClassBc6h!]: Self = Self::new(FormatType::ClassBc6h);
        const [!ClassBc7!]: Self = Self::new(FormatType::ClassBc7);
        const [!ClassEtc2Rgb!]: Self = Self::new(FormatType::ClassEtc2Rgb);
        const [!ClassEtc2Rgba!]: Self = Self::new(FormatType::ClassEtc2Rgba);
        const [!ClassEtc2EacRgba!]: Self = Self::new(FormatType::ClassEtc2EacRgba);
        const [!ClassEacR!]: Self = Self::new(FormatType::ClassEacR);
        const [!ClassEacRG!]: Self = Self::new(FormatType::ClassEacRG);
        const [!ClassAstc4x4!]: Self = Self::new(FormatType::ClassAstc4x4);
        const [!ClassAstc5x4!]: Self = Self::new(FormatType::ClassAstc5x4);
        const [!ClassAstc5x5!]: Self = Self::new(FormatType::ClassAstc5x5);
        const [!ClassAstc6x5!]: Self = Self::new(FormatType::ClassAstc6x5);
        const [!ClassAstc6x6!]: Self = Self::new(FormatType::ClassAstc6x6);
        const [!ClassAstc8x5!]: Self = Self::new(FormatType::ClassAstc8x5);
        const [!ClassAstc8x6!]: Self = Self::new(FormatType::ClassAstc8x6);
        const [!ClassAstc8x8!]: Self = Self::new(FormatType::ClassAstc8x8);
        const [!ClassAstc10x5!]: Self = Self::new(FormatType::ClassAstc10x5);
        const [!ClassAstc10x6!]: Self = Self::new(FormatType::ClassAstc10x6);
        const [!ClassAstc10x8!]: Self = Self::new(FormatType::ClassAstc10x8);
        const [!ClassAstc10x10!]: Self = Self::new(FormatType::ClassAstc10x10);
        const [!ClassAstc12x10!]: Self = Self::new(FormatType::ClassAstc12x10);
        const [!ClassAstc12x12!]: Self = Self::new(FormatType::ClassAstc12x12);
        const [!Class32BitG8b8g8r8!]: Self = Self::new(FormatType::Class32BitG8b8g8r8);
        const [!Class32BitB8g8r8g8!]: Self = Self::new(FormatType::Class32BitB8g8r8g8);
        const [!Class8Bit3Plane420!]: Self = Self::new(FormatType::Class8Bit3Plane420);
        const [!Class8Bit2Plane420!]: Self = Self::new(FormatType::Class8Bit2Plane420);
        const [!Class8Bit3Plane422!]: Self = Self::new(FormatType::Class8Bit3Plane422);
        const [!Class8Bit2Plane422!]: Self = Self::new(FormatType::Class8Bit2Plane422);
        const [!Class8Bit3Plane444!]: Self = Self::new(FormatType::Class8Bit3Plane444);
        const [!Class8Bit2Plane444!]: Self = Self::new(FormatType::Class8Bit2Plane444);
        const [!Class64BitR10g10b10a10!]: Self = Self::new(FormatType::Class64BitR10g10b10a10);
        const [!Class64BitG10b10g10r10!]: Self = Self::new(FormatType::Class64BitG10b10g10r10);
        const [!Class64BitB10g10r10g10!]: Self = Self::new(FormatType::Class64BitB10g10r10g10);
        const [!Class10Bit3Plane420!]: Self = Self::new(FormatType::Class10Bit3Plane420);
        const [!Class10Bit2Plane420!]: Self = Self::new(FormatType::Class10Bit2Plane420);
        const [!Class10Bit3Plane422!]: Self = Self::new(FormatType::Class10Bit3Plane422);
        const [!Class10Bit2Plane422!]: Self = Self::new(FormatType::Class10Bit2Plane422);
        const [!Class10Bit3Plane444!]: Self = Self::new(FormatType::Class10Bit3Plane444);
        const [!Class10Bit2Plane444!]: Self = Self::new(FormatType::Class10Bit2Plane444);
        const [!Class64BitR12g12b12a12!]: Self = Self::new(FormatType::Class64BitR12g12b12a12);
        const [!Class64BitG12b12g12r12!]: Self = Self::new(FormatType::Class64BitG12b12g12r12);
        const [!Class64BitB12g12r12g12!]: Self = Self::new(FormatType::Class64BitB12g12r12g12);
        const [!Class12Bit3Plane420!]: Self = Self::new(FormatType::Class12Bit3Plane420);
        const [!Class12Bit2Plane420!]: Self = Self::new(FormatType::Class12Bit2Plane420);
        const [!Class12Bit3Plane422!]: Self = Self::new(FormatType::Class12Bit3Plane422);
        const [!Class12Bit2Plane422!]: Self = Self::new(FormatType::Class12Bit2Plane422);
        const [!Class12Bit3Plane444!]: Self = Self::new(FormatType::Class12Bit3Plane444);
        const [!Class12Bit2Plane444!]: Self = Self::new(FormatType::Class12Bit2Plane444);
        const [!Class64BitG16b16g16r16!]: Self = Self::new(FormatType::Class64BitG16b16g16r16);
        const [!Class64BitB16g16r16g16!]: Self = Self::new(FormatType::Class64BitB16g16r16g16);
        const [!Class16Bit3Plane420!]: Self = Self::new(FormatType::Class16Bit3Plane420);
        const [!Class16Bit2Plane420!]: Self = Self::new(FormatType::Class16Bit2Plane420);
        const [!Class16Bit3Plane422!]: Self = Self::new(FormatType::Class16Bit3Plane422);
        const [!Class16Bit2Plane422!]: Self = Self::new(FormatType::Class16Bit2Plane422);
        const [!Class16Bit3Plane444!]: Self = Self::new(FormatType::Class16Bit3Plane444);
        const [!Class16Bit2Plane444!]: Self = Self::new(FormatType::Class16Bit2Plane444);

    }

    #[inline(always)]
    pub const fn new(
        class: FormatType,
    ) -> Self
    {
        use FormatType as F;
        match class {
            F::Class8Bit =>
                Self::_new(class, 1, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit =>
                Self::_new(class, 2, Dimensions::new(1, 1, 1), 1),
            F::Class8BitAlpha =>
                Self::_new(class, 1, Dimensions::new(1, 1, 1), 1),
            F::Class24Bit =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class32Bit =>
                Self::_new(class, 4, Dimensions::new(1, 1, 1), 1),
            F::Class48Bit =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class64Bit =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class96Bit =>
                Self::_new(class, 12, Dimensions::new(1, 1, 1), 1),
            F::Class128Bit =>
                Self::_new(class, 16, Dimensions::new(1, 1, 1), 1),
            F::Class192Bit =>
                Self::_new(class, 24, Dimensions::new(1, 1, 1), 1),
            F::Class256Bit =>
                Self::_new(class, 32, Dimensions::new(1, 1, 1), 1),
            F::ClassD16 =>
                Self::_new(class, 2, Dimensions::new(1, 1, 1), 1),
            F::ClassD24 =>
                Self::_new(class, 4, Dimensions::new(1, 1, 1), 1),
            F::ClassD32 =>
                Self::_new(class, 4, Dimensions::new(1, 1, 1), 1),
            F::ClassS8 =>
                Self::_new(class, 1, Dimensions::new(1, 1, 1), 1),
            F::ClassD16s8 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::ClassD24s8 =>
                Self::_new(class, 4, Dimensions::new(1, 1, 1), 1),
            F::ClassD32s8 =>
                Self::_new(class, 5, Dimensions::new(1, 1, 1), 1),
            F::ClassBc1Rgb =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassBc1Rgba =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassBc2 =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassBc3 =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassBc4 =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassBc5 =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassBc6h =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassBc7 =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassEtc2Rgb =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassEtc2Rgba =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassEtc2EacRgba =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassEacR =>
                Self::_new(class, 8, Dimensions::new(4, 4, 1), 16),
            F::ClassEacRG =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassAstc4x4 =>
                Self::_new(class, 16, Dimensions::new(4, 4, 1), 16),
            F::ClassAstc5x4 =>
                Self::_new(class, 16, Dimensions::new(5, 4, 1), 20),
            F::ClassAstc5x5 =>
                Self::_new(class, 16, Dimensions::new(5, 5, 1), 25),
            F::ClassAstc6x5 =>
                Self::_new(class, 16, Dimensions::new(6, 5, 1), 30),
            F::ClassAstc6x6 =>
                Self::_new(class, 16, Dimensions::new(6, 6, 1), 36),
            F::ClassAstc8x5 =>
                Self::_new(class, 16, Dimensions::new(8, 5, 1), 40),
            F::ClassAstc8x6 =>
                Self::_new(class, 16, Dimensions::new(8, 6, 1), 48),
            F::ClassAstc8x8 =>
                Self::_new(class, 16, Dimensions::new(8, 8, 1), 64),
            F::ClassAstc10x5 =>
                Self::_new(class, 16, Dimensions::new(10, 5, 1), 50),
            F::ClassAstc10x6 =>
                Self::_new(class, 16, Dimensions::new(10, 6, 1), 60),
            F::ClassAstc10x8 =>
                Self::_new(class, 16, Dimensions::new(10, 8, 1), 80),
            F::ClassAstc10x10 =>
                Self::_new(class, 16, Dimensions::new(10, 10, 1), 100),
            F::ClassAstc12x10 =>
                Self::_new(class, 16, Dimensions::new(12, 10, 1), 120),
            F::ClassAstc12x12 =>
                Self::_new(class, 16, Dimensions::new(12, 12, 1), 144),
            F::Class32BitG8b8g8r8 =>
                Self::_new(class, 4, Dimensions::new(2, 1, 1), 1),
            F::Class32BitB8g8r8g8 =>
                Self::_new(class, 4, Dimensions::new(2, 1, 1), 1),
            F::Class8Bit3Plane420 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class8Bit2Plane420 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class8Bit3Plane422 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class8Bit2Plane422 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class8Bit3Plane444 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class8Bit2Plane444 =>
                Self::_new(class, 3, Dimensions::new(1, 1, 1), 1),
            F::Class64BitR10g10b10a10 =>
                Self::_new(class, 8, Dimensions::new(1, 1, 1), 1),
            F::Class64BitG10b10g10r10 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class64BitB10g10r10g10 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class10Bit3Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class10Bit2Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class10Bit3Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class10Bit2Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class10Bit3Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class10Bit2Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class64BitR12g12b12a12 =>
                Self::_new(class, 8, Dimensions::new(1, 1, 1), 1),
            F::Class64BitG12b12g12r12 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class64BitB12g12r12g12 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class12Bit3Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class12Bit2Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class12Bit3Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class12Bit2Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class12Bit3Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class12Bit2Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class64BitG16b16g16r16 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class64BitB16g16r16g16 =>
                Self::_new(class, 8, Dimensions::new(2, 1, 1), 1),
            F::Class16Bit3Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit2Plane420 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit3Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit2Plane422 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit3Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Class16Bit2Plane444 =>
                Self::_new(class, 6, Dimensions::new(1, 1, 1), 1),
            F::Undefined =>
                Self::_new(class, 0, Dimensions::new(0, 0, 0), 0),
        }
    }

    const fn _new(
        class: FormatType,
        texel_block_size: DeviceSize,
        texel_block_extent: Dimensions,
        texels_per_block: u8,
    ) -> Self {
        Self {
            class,
            texel_block_size,
            texel_block_extent,
            texels_per_block
        }
    }   
}

impl PartialEq for FormatCompatibilityClass {

    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
    }
}

impl Eq for FormatCompatibilityClass {}

impl Hash for FormatCompatibilityClass {

    #[inline(always)]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
pub enum NumericFormat {
    #[display("unorm")]
    Unorm,
    #[display("snorm")]
    Snorm,
    #[display("uscaled")]
    Uscaled,
    #[display("sscaled")]
    Sscaled,
    #[display("uint")]
    Uint,
    #[display("sint")]
    Sint,
    #[display("ufloat")]
    Ufloat,
    #[display("sfloat")]
    Sfloat,
    #[display("srgb")]
    Srgb,
}

impl NumericFormat {

    #[inline(always)]
    pub fn is_floating_point(self) -> bool {
        matches!(self,
            Self::Unorm |
            Self::Snorm |
            Self::Uscaled |
            Self::Sscaled |
            Self::Ufloat |
            Self::Sfloat |
            Self::Srgb
        )
    }

    #[inline(always)]
    pub fn is_unsigned_integer(self) -> bool {
        matches!(self, Self::Uint)
    }

    #[inline(always)]
    pub fn is_signed_integer(self) -> bool {
        matches!(self, Self::Sint)
    }

    #[inline(always)]
    pub fn is_integer(self) -> bool {
        matches!(self, Self::Uint | Self::Sint)
    }
}

nox_proc::vk_to_rust_enum!(
    /// An enumeration of all Vulkan formats guaranteed to be supported by Nox.
    #[repr(i32)]
    #[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, AsRaw)]
    pub enum Format
        where
            color = NumericFormat,
            depth = NumericFormat,
            stencil = NumericFormat,
    {
        #[default]
        #[group(undefined)]
        VK_FORMAT_UNDEFINED = 0,
        #[group(color(NumericFormat::Unorm), class_8bit)]
        VK_FORMAT_R4G4_UNORM_PACK8 = 1,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R4G4B4A4_UNORM_PACK16 = 2,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_B4G4R4A4_UNORM_PACK16 = 3,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R5G6B5_UNORM_PACK16 = 4,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_B5G6R5_UNORM_PACK16 = 5,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R5G5B5A1_UNORM_PACK16 = 6,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_B5G5R5A1_UNORM_PACK16 = 7,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_A1R5G5B5_UNORM_PACK16 = 8,
        #[group(color(NumericFormat::Unorm), class_8bit)]
        VK_FORMAT_R8_UNORM = 9,
        #[group(color(NumericFormat::Snorm), class_8bit)]
        VK_FORMAT_R8_SNORM = 10,
        #[group(color(NumericFormat::Uscaled), class_8bit)]
        VK_FORMAT_R8_USCALED = 11,
        #[group(color(NumericFormat::Sscaled), class_8bit)]
        VK_FORMAT_R8_SSCALED = 12,
        #[group(color(NumericFormat::Uint), class_8bit)]
        VK_FORMAT_R8_UINT = 13,
        #[group(color(NumericFormat::Sint), class_8bit)]
        VK_FORMAT_R8_SINT = 14,
        #[group(color(NumericFormat::Srgb), class_8bit)]
        VK_FORMAT_R8_SRGB = 15,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R8G8_UNORM = 16,
        #[group(color(NumericFormat::Snorm), class_16bit)]
        VK_FORMAT_R8G8_SNORM = 17,
        #[group(color(NumericFormat::Uscaled), class_16bit)]
        VK_FORMAT_R8G8_USCALED = 18,
        #[group(color(NumericFormat::Sscaled), class_16bit)]
        VK_FORMAT_R8G8_SSCALED = 19,
        #[group(color(NumericFormat::Uint), class_16bit)]
        VK_FORMAT_R8G8_UINT = 20,
        #[group(color(NumericFormat::Sint), class_16bit)]
        VK_FORMAT_R8G8_SINT = 21,
        #[group(color(NumericFormat::Srgb), class_16bit)]
        VK_FORMAT_R8G8_SRGB = 22,
        #[group(color(NumericFormat::Unorm), class_24bit)]
        VK_FORMAT_R8G8B8_UNORM = 23,
        #[group(color(NumericFormat::Snorm), class_24bit)]
        VK_FORMAT_R8G8B8_SNORM = 24,
        #[group(color(NumericFormat::Uscaled), class_24bit)]
        VK_FORMAT_R8G8B8_USCALED = 25,
        #[group(color(NumericFormat::Sscaled), class_24bit)]
        VK_FORMAT_R8G8B8_SSCALED = 26,
        #[group(color(NumericFormat::Uint), class_24bit)]
        VK_FORMAT_R8G8B8_UINT = 27,
        #[group(color(NumericFormat::Sint), class_24bit)]
        VK_FORMAT_R8G8B8_SINT = 28,
        #[group(color(NumericFormat::Srgb), class_24bit)]
        VK_FORMAT_R8G8B8_SRGB = 29,
        #[group(color(NumericFormat::Unorm), class_24bit)]
        VK_FORMAT_B8G8R8_UNORM = 30,
        #[group(color(NumericFormat::Snorm), class_24bit)]
        VK_FORMAT_B8G8R8_SNORM = 31,
        #[group(color(NumericFormat::Uscaled), class_24bit)]
        VK_FORMAT_B8G8R8_USCALED = 32,
        #[group(color(NumericFormat::Sscaled), class_24bit)]
        VK_FORMAT_B8G8R8_SSCALED = 33,
        #[group(color(NumericFormat::Uint), class_24bit)]
        VK_FORMAT_B8G8R8_UINT = 34,
        #[group(color(NumericFormat::Sint), class_24bit)]
        VK_FORMAT_B8G8R8_SINT = 35,
        #[group(color(NumericFormat::Srgb), class_24bit)]
        VK_FORMAT_B8G8R8_SRGB = 36,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_R8G8B8A8_UNORM = 37,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_R8G8B8A8_SNORM = 38,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_R8G8B8A8_USCALED = 39,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_R8G8B8A8_SSCALED = 40,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_R8G8B8A8_UINT = 41,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_R8G8B8A8_SINT = 42,
        #[group(color(NumericFormat::Srgb), class_32bit)]
        VK_FORMAT_R8G8B8A8_SRGB = 43,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_B8G8R8A8_UNORM = 44,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_B8G8R8A8_SNORM = 45,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_B8G8R8A8_USCALED = 46,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_B8G8R8A8_SSCALED = 47,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_B8G8R8A8_UINT = 48,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_B8G8R8A8_SINT = 49,
        #[group(color(NumericFormat::Srgb), class_32bit)]
        VK_FORMAT_B8G8R8A8_SRGB = 50,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_A8B8G8R8_UNORM_PACK32 = 51,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_A8B8G8R8_SNORM_PACK32 = 52,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_A8B8G8R8_USCALED_PACK32 = 53,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_A8B8G8R8_SSCALED_PACK32 = 54,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_A8B8G8R8_UINT_PACK32 = 55,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_A8B8G8R8_SINT_PACK32 = 56,
        #[group(color(NumericFormat::Srgb), class_32bit)]
        VK_FORMAT_A8B8G8R8_SRGB_PACK32 = 57,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_A2R10G10B10_UNORM_PACK32 = 58,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_A2R10G10B10_SNORM_PACK32 = 59,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_A2R10G10B10_USCALED_PACK32 = 60,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_A2R10G10B10_SSCALED_PACK32 = 61,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_A2R10G10B10_UINT_PACK32 = 62,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_A2R10G10B10_SINT_PACK32 = 63,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_A2B10G10R10_UNORM_PACK32 = 64,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_A2B10G10R10_SNORM_PACK32 = 65,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_A2B10G10R10_USCALED_PACK32 = 66,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_A2B10G10R10_SSCALED_PACK32 = 67,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_A2B10G10R10_UINT_PACK32 = 68,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_A2B10G10R10_SINT_PACK32 = 69,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R16_UNORM = 70,
        #[group(color(NumericFormat::Snorm), class_16bit)]
        VK_FORMAT_R16_SNORM = 71,
        #[group(color(NumericFormat::Uscaled), class_16bit)]
        VK_FORMAT_R16_USCALED = 72,
        #[group(color(NumericFormat::Sscaled), class_16bit)]
        VK_FORMAT_R16_SSCALED = 73,
        #[group(color(NumericFormat::Uint), class_16bit)]
        VK_FORMAT_R16_UINT = 74,
        #[group(color(NumericFormat::Sint), class_16bit)]
        VK_FORMAT_R16_SINT = 75,
        #[group(color(NumericFormat::Sfloat), class_16bit)]
        VK_FORMAT_R16_SFLOAT = 76,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_R16G16_UNORM = 77,
        #[group(color(NumericFormat::Snorm), class_32bit)]
        VK_FORMAT_R16G16_SNORM = 78,
        #[group(color(NumericFormat::Uscaled), class_32bit)]
        VK_FORMAT_R16G16_USCALED = 79,
        #[group(color(NumericFormat::Sscaled), class_32bit)]
        VK_FORMAT_R16G16_SSCALED = 80,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_R16G16_UINT = 81,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_R16G16_SINT = 82,
        #[group(color(NumericFormat::Sfloat), class_32bit)]
        VK_FORMAT_R16G16_SFLOAT = 83,
        #[group(color(NumericFormat::Unorm), class_48bit)]
        VK_FORMAT_R16G16B16_UNORM = 84,
        #[group(color(NumericFormat::Snorm), class_48bit)]
        VK_FORMAT_R16G16B16_SNORM = 85,
        #[group(color(NumericFormat::Uscaled), class_48bit)]
        VK_FORMAT_R16G16B16_USCALED = 86,
        #[group(color(NumericFormat::Sscaled), class_48bit)]
        VK_FORMAT_R16G16B16_SSCALED = 87,
        #[group(color(NumericFormat::Uint), class_48bit)]
        VK_FORMAT_R16G16B16_UINT = 88,
        #[group(color(NumericFormat::Sint), class_48bit)]
        VK_FORMAT_R16G16B16_SINT = 89,
        #[group(color(NumericFormat::Sfloat), class_48bit)]
        VK_FORMAT_R16G16B16_SFLOAT = 90,
        #[group(color(NumericFormat::Unorm), class_64bit)]
        VK_FORMAT_R16G16B16A16_UNORM = 91,
        #[group(color(NumericFormat::Snorm), class_64bit)]
        VK_FORMAT_R16G16B16A16_SNORM = 92,
        #[group(color(NumericFormat::Uscaled), class_64bit)]
        VK_FORMAT_R16G16B16A16_USCALED = 93,
        #[group(color(NumericFormat::Sscaled), class_64bit)]
        VK_FORMAT_R16G16B16A16_SSCALED = 94,
        #[group(color(NumericFormat::Uint), class_64bit)]
        VK_FORMAT_R16G16B16A16_UINT = 95,
        #[group(color(NumericFormat::Sint), class_64bit)]
        VK_FORMAT_R16G16B16A16_SINT = 96,
        #[group(color(NumericFormat::Sfloat), class_64bit)]
        VK_FORMAT_R16G16B16A16_SFLOAT = 97,
        #[group(color(NumericFormat::Uint), class_32bit)]
        VK_FORMAT_R32_UINT = 98,
        #[group(color(NumericFormat::Sint), class_32bit)]
        VK_FORMAT_R32_SINT = 99,
        #[group(color(NumericFormat::Sfloat), class_32bit)]
        VK_FORMAT_R32_SFLOAT = 100,
        #[group(color(NumericFormat::Uint), class_64bit)]
        VK_FORMAT_R32G32_UINT = 101,
        #[group(color(NumericFormat::Sint), class_64bit)]
        VK_FORMAT_R32G32_SINT = 102,
        #[group(color(NumericFormat::Sfloat), class_64bit)]
        VK_FORMAT_R32G32_SFLOAT = 103,
        #[group(color(NumericFormat::Uint), class_96bit)]
        VK_FORMAT_R32G32B32_UINT = 104,
        #[group(color(NumericFormat::Sint), class_96bit)]
        VK_FORMAT_R32G32B32_SINT = 105,
        #[group(color(NumericFormat::Sfloat), class_96bit)]
        VK_FORMAT_R32G32B32_SFLOAT = 106,
        #[group(color(NumericFormat::Uint), class_128bit)]
        VK_FORMAT_R32G32B32A32_UINT = 107,
        #[group(color(NumericFormat::Sint), class_128bit)]
        VK_FORMAT_R32G32B32A32_SINT = 108,
        #[group(color(NumericFormat::Sfloat), class_128bit)]
        VK_FORMAT_R32G32B32A32_SFLOAT = 109,
        #[group(color(NumericFormat::Uint), class_64bit)]
        VK_FORMAT_R64_UINT = 110,
        #[group(color(NumericFormat::Sint), class_64bit)]
        VK_FORMAT_R64_SINT = 111,
        #[group(color(NumericFormat::Sfloat), class_64bit)]
        VK_FORMAT_R64_SFLOAT = 112,
        #[group(color(NumericFormat::Uint), class_128bit)]
        VK_FORMAT_R64G64_UINT = 113,
        #[group(color(NumericFormat::Sint), class_128bit)]
        VK_FORMAT_R64G64_SINT = 114,
        #[group(color(NumericFormat::Sfloat), class_128bit)]
        VK_FORMAT_R64G64_SFLOAT = 115,
        #[group(color(NumericFormat::Uint), class_192bit)]
        VK_FORMAT_R64G64B64_UINT = 116,
        #[group(color(NumericFormat::Sint), class_192bit)]
        VK_FORMAT_R64G64B64_SINT = 117,
        #[group(color(NumericFormat::Sfloat), class_192bit)]
        VK_FORMAT_R64G64B64_SFLOAT = 118,
        #[group(color(NumericFormat::Uint), class_256bit)]
        VK_FORMAT_R64G64B64A64_UINT = 119,
        #[group(color(NumericFormat::Sint), class_256bit)]
        VK_FORMAT_R64G64B64A64_SINT = 120,
        #[group(color(NumericFormat::Sfloat), class_256bit)]
        VK_FORMAT_R64G64B64A64_SFLOAT = 121,
        #[group(color(NumericFormat::Ufloat), class_32bit)]
        VK_FORMAT_B10G11R11_UFLOAT_PACK32 = 122,
        #[group(color(NumericFormat::Ufloat), class_32bit)]
        VK_FORMAT_E5B9G9R9_UFLOAT_PACK32 = 123,
        #[group(depth(NumericFormat::Unorm), class_d16)]
        VK_FORMAT_D16_UNORM = 124,
        #[group(depth(NumericFormat::Unorm), class_d24)]
        VK_FORMAT_X8_D24_UNORM_PACK32 = 125,
        #[group(depth(NumericFormat::Sfloat), class_d32)]
        VK_FORMAT_D32_SFLOAT = 126,
        #[group(stencil(NumericFormat::Uint), class_s8)]
        VK_FORMAT_S8_UINT = 127,
        #[group(depth(NumericFormat::Unorm), stencil(NumericFormat::Uint), class_d16s8)]
        VK_FORMAT_D16_UNORM_S8_UINT = 128,
        #[group(depth(NumericFormat::Unorm), stencil(NumericFormat::Uint), class_d24s8)]
        VK_FORMAT_D24_UNORM_S8_UINT = 129,
        #[group(depth(NumericFormat::Sfloat), stencil(NumericFormat::Uint), class_d32s8)]
        VK_FORMAT_D32_SFLOAT_S8_UINT = 130,
        #[group(color(NumericFormat::Unorm), class_bc1_rgb)]
        VK_FORMAT_BC1_RGB_UNORM_BLOCK = 131,
        #[group(color(NumericFormat::Srgb), class_bc1_rgb)]
        VK_FORMAT_BC1_RGB_SRGB_BLOCK = 132,
        #[group(color(NumericFormat::Unorm), class_bc1_rgba)]
        VK_FORMAT_BC1_RGBA_UNORM_BLOCK = 133,
        #[group(color(NumericFormat::Srgb), class_bc1_rgba)]
        VK_FORMAT_BC1_RGBA_SRGB_BLOCK = 134,
        #[group(color(NumericFormat::Unorm), class_bc2)]
        VK_FORMAT_BC2_UNORM_BLOCK = 135,
        #[group(color(NumericFormat::Srgb), class_bc2)]
        VK_FORMAT_BC2_SRGB_BLOCK = 136,
        #[group(color(NumericFormat::Unorm), class_bc3)]
        VK_FORMAT_BC3_UNORM_BLOCK = 137,
        #[group(color(NumericFormat::Srgb), class_bc3)]
        VK_FORMAT_BC3_SRGB_BLOCK = 138,
        #[group(color(NumericFormat::Unorm), class_bc4)]
        VK_FORMAT_BC4_UNORM_BLOCK = 139,
        #[group(color(NumericFormat::Snorm), class_bc4)]
        VK_FORMAT_BC4_SNORM_BLOCK = 140,
        #[group(color(NumericFormat::Unorm), class_bc5)]
        VK_FORMAT_BC5_UNORM_BLOCK = 141,
        #[group(color(NumericFormat::Snorm), class_bc5)]
        VK_FORMAT_BC5_SNORM_BLOCK = 142,
        #[group(color(NumericFormat::Ufloat), class_bc6h)]
        VK_FORMAT_BC6H_UFLOAT_BLOCK = 143,
        #[group(color(NumericFormat::Sfloat), class_bc6h)]
        VK_FORMAT_BC6H_SFLOAT_BLOCK = 144,
        #[group(color(NumericFormat::Unorm), class_bc7)]
        VK_FORMAT_BC7_UNORM_BLOCK = 145,
        #[group(color(NumericFormat::Srgb), class_bc7)]
        VK_FORMAT_BC7_SRGB_BLOCK = 146,
        #[group(color(NumericFormat::Unorm), class_etc2_rgb)]
        VK_FORMAT_ETC2_R8G8B8_UNORM_BLOCK = 147,
        #[group(color(NumericFormat::Srgb), class_etc2_rgb)]
        VK_FORMAT_ETC2_R8G8B8_SRGB_BLOCK = 148,
        #[group(color(NumericFormat::Unorm), class_etc2_rgba)]
        VK_FORMAT_ETC2_R8G8B8A1_UNORM_BLOCK = 149,
        #[group(color(NumericFormat::Srgb), class_etc2_rgba)]
        VK_FORMAT_ETC2_R8G8B8A1_SRGB_BLOCK = 150,
        #[group(color(NumericFormat::Unorm), class_etc2_eac_rgba)]
        VK_FORMAT_ETC2_R8G8B8A8_UNORM_BLOCK = 151,
        #[group(color(NumericFormat::Srgb), class_etc2_eac_rgba)]
        VK_FORMAT_ETC2_R8G8B8A8_SRGB_BLOCK = 152,
        #[group(color(NumericFormat::Unorm), class_eac_r)]
        VK_FORMAT_EAC_R11_UNORM_BLOCK = 153,
        #[group(color(NumericFormat::Snorm), class_eac_r)]
        VK_FORMAT_EAC_R11_SNORM_BLOCK = 154,
        #[group(color(NumericFormat::Unorm), class_eac_rg)]
        VK_FORMAT_EAC_R11G11_UNORM_BLOCK = 155,
        #[group(color(NumericFormat::Snorm), class_eac_rg)]
        VK_FORMAT_EAC_R11G11_SNORM_BLOCK = 156,
        #[group(color(NumericFormat::Unorm), class_astc_4x4)]
        VK_FORMAT_ASTC_4x4_UNORM_BLOCK = 157,
        #[group(color(NumericFormat::Srgb), class_astc_4x4)]
        VK_FORMAT_ASTC_4x4_SRGB_BLOCK = 158,
        #[group(color(NumericFormat::Unorm), class_astc_5x4)]
        VK_FORMAT_ASTC_5x4_UNORM_BLOCK = 159,
        #[group(color(NumericFormat::Srgb), class_astc_5x4)]
        VK_FORMAT_ASTC_5x4_SRGB_BLOCK = 160,
        #[group(color(NumericFormat::Unorm), class_astc_5x5)]
        VK_FORMAT_ASTC_5x5_UNORM_BLOCK = 161,
        #[group(color(NumericFormat::Srgb), class_astc_5x5)]
        VK_FORMAT_ASTC_5x5_SRGB_BLOCK = 162,
        #[group(color(NumericFormat::Unorm), class_astc_6x5)]
        VK_FORMAT_ASTC_6x5_UNORM_BLOCK = 163,
        #[group(color(NumericFormat::Srgb), class_astc_6x5)]
        VK_FORMAT_ASTC_6x5_SRGB_BLOCK = 164,
        #[group(color(NumericFormat::Unorm), class_astc_6x6)]
        VK_FORMAT_ASTC_6x6_UNORM_BLOCK = 165,
        #[group(color(NumericFormat::Srgb), class_astc_6x6)]
        VK_FORMAT_ASTC_6x6_SRGB_BLOCK = 166,
        #[group(color(NumericFormat::Unorm), class_astc_8x5)]
        VK_FORMAT_ASTC_8x5_UNORM_BLOCK = 167,
        #[group(color(NumericFormat::Srgb), class_astc_8x5)]
        VK_FORMAT_ASTC_8x5_SRGB_BLOCK = 168,
        #[group(color(NumericFormat::Unorm), class_astc_8x6)]
        VK_FORMAT_ASTC_8x6_UNORM_BLOCK = 169,
        #[group(color(NumericFormat::Srgb), class_astc_8x6)]
        VK_FORMAT_ASTC_8x6_SRGB_BLOCK = 170,
        #[group(color(NumericFormat::Unorm), class_astc_8x8)]
        VK_FORMAT_ASTC_8x8_UNORM_BLOCK = 171,
        #[group(color(NumericFormat::Srgb), class_astc_8x8)]
        VK_FORMAT_ASTC_8x8_SRGB_BLOCK = 172,
        #[group(color(NumericFormat::Unorm), class_astc_10x5)]
        VK_FORMAT_ASTC_10x5_UNORM_BLOCK = 173,
        #[group(color(NumericFormat::Srgb), class_astc_10x5)]
        VK_FORMAT_ASTC_10x5_SRGB_BLOCK = 174,
        #[group(color(NumericFormat::Unorm), class_astc_10x6)]
        VK_FORMAT_ASTC_10x6_UNORM_BLOCK = 175,
        #[group(color(NumericFormat::Srgb), class_astc_10x6)]
        VK_FORMAT_ASTC_10x6_SRGB_BLOCK = 176,
        #[group(color(NumericFormat::Unorm), class_astc_10x8)]
        VK_FORMAT_ASTC_10x8_UNORM_BLOCK = 177,
        #[group(color(NumericFormat::Srgb), class_astc_10x8)]
        VK_FORMAT_ASTC_10x8_SRGB_BLOCK = 178,
        #[group(color(NumericFormat::Unorm), class_astc_10x10)]
        VK_FORMAT_ASTC_10x10_UNORM_BLOCK = 179,
        #[group(color(NumericFormat::Srgb), class_astc_10x10)]
        VK_FORMAT_ASTC_10x10_SRGB_BLOCK = 180,
        #[group(color(NumericFormat::Unorm), class_astc_12x10)]
        VK_FORMAT_ASTC_12x10_UNORM_BLOCK = 181,
        #[group(color(NumericFormat::Srgb), class_astc_12x10)]
        VK_FORMAT_ASTC_12x10_SRGB_BLOCK = 182,
        #[group(color(NumericFormat::Unorm), class_astc_12x12)]
        VK_FORMAT_ASTC_12x12_UNORM_BLOCK = 183,
        #[group(color(NumericFormat::Srgb), class_astc_12x12)]
        VK_FORMAT_ASTC_12x12_SRGB_BLOCK = 184,
        #[group(color(NumericFormat::Unorm), class_32bit_g8b8g8r8)]
        VK_FORMAT_G8B8G8R8_422_UNORM = 1000156000,
        #[group(color(NumericFormat::Unorm), class_32bit_b8g8r8g8)]
        VK_FORMAT_B8G8R8G8_422_UNORM = 1000156001,
        #[group(plane3(NumericFormat::Unorm), class_8bit_3plane_420)]
        VK_FORMAT_G8_B8_R8_3PLANE_420_UNORM = 1000156002,
        #[group(plane2(NumericFormat::Unorm), class_8bit_2plane_420)]
        VK_FORMAT_G8_B8R8_2PLANE_420_UNORM = 1000156003,
        #[group(plane3(NumericFormat::Unorm), class_8bit_3plane_422)]
        VK_FORMAT_G8_B8_R8_3PLANE_422_UNORM = 1000156004,
        #[group(plane2(NumericFormat::Unorm), class_8bit_2plane_422)]
        VK_FORMAT_G8_B8R8_2PLANE_422_UNORM = 1000156005,
        #[group(plane3(NumericFormat::Unorm), class_8bit_3plane_444)]
        VK_FORMAT_G8_B8_R8_3PLANE_444_UNORM = 1000156006,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R10X6_UNORM_PACK16 = 1000156007,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_R10X6G10X6_UNORM_2PACK16 = 1000156008,
        #[group(color(NumericFormat::Unorm), class_64bit_r10g10b10a10)]
        VK_FORMAT_R10X6G10X6B10X6A10X6_UNORM_4PACK16 = 1000156009,
        #[group(color(NumericFormat::Unorm), class_64bit_g10b10g10r10)]
        VK_FORMAT_G10X6B10X6G10X6R10X6_422_UNORM_4PACK16 = 1000156010,
        #[group(color(NumericFormat::Unorm), class_64bit_b10g10r10g10)]
        VK_FORMAT_B10X6G10X6R10X6G10X6_422_UNORM_4PACK16 = 1000156011,
        #[group(plane3(NumericFormat::Unorm), class_10bit_3plane_420)]
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16 = 1000156012,
        #[group(plane2(NumericFormat::Unorm), class_10bit_2plane_420)]
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16 = 1000156013,
        #[group(plane3(NumericFormat::Unorm), class_10bit_3plane_422)]
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16 = 1000156014,
        #[group(plane2(NumericFormat::Unorm), class_10bit_2plane_422)]
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16 = 1000156015,
        #[group(plane3(NumericFormat::Unorm), class_10bit_3plane_444)]
        VK_FORMAT_G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16 = 1000156016,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_R12X4_UNORM_PACK16 = 1000156017,
        #[group(color(NumericFormat::Unorm), class_32bit)]
        VK_FORMAT_R12X4G12X4_UNORM_2PACK16 = 1000156018,
        #[group(color(NumericFormat::Unorm), class_64bit_r12g12b12a12)]
        VK_FORMAT_R12X4G12X4B12X4A12X4_UNORM_4PACK16 = 1000156019,
        #[group(color(NumericFormat::Unorm), class_64bit_g12b12g12r12)]
        VK_FORMAT_G12X4B12X4G12X4R12X4_422_UNORM_4PACK16 = 1000156020,
        #[group(color(NumericFormat::Unorm), class_64bit_b12g12r12g12)]
        VK_FORMAT_B12X4G12X4R12X4G12X4_422_UNORM_4PACK16 = 1000156021,
        #[group(plane3(NumericFormat::Unorm), class_12bit_3plane_420)]
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16 = 1000156022,
        #[group(plane2(NumericFormat::Unorm), class_12bit_2plane_420)]
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16 = 1000156023,
        #[group(plane3(NumericFormat::Unorm), class_12bit_3plane_422)]
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16 = 1000156024,
        #[group(plane2(NumericFormat::Unorm), class_12bit_2plane_422)]
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16 = 1000156025,
        #[group(plane3(NumericFormat::Unorm), class_12bit_3plane_444)]
        VK_FORMAT_G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16 = 1000156026,
        #[group(color(NumericFormat::Unorm), class_64bit_g16b16g16r16)]
        VK_FORMAT_G16B16G16R16_422_UNORM = 1000156027,
        #[group(color(NumericFormat::Unorm), class_64bit_b16g16r16g16)]
        VK_FORMAT_B16G16R16G16_422_UNORM = 1000156028,
        #[group(plane3(NumericFormat::Unorm), class_16bit_3plane_420)]
        VK_FORMAT_G16_B16_R16_3PLANE_420_UNORM = 1000156029,
        #[group(plane2(NumericFormat::Unorm), class_16bit_2plane_420)]
        VK_FORMAT_G16_B16R16_2PLANE_420_UNORM = 1000156030,
        #[group(plane3(NumericFormat::Unorm), class_16bit_3plane_422)]
        VK_FORMAT_G16_B16_R16_3PLANE_422_UNORM = 1000156031,
        #[group(plane2(NumericFormat::Unorm), class_16bit_2plane_422)]
        VK_FORMAT_G16_B16R16_2PLANE_422_UNORM = 1000156032,
        #[group(plane3(NumericFormat::Unorm), class_16bit_3plane_444)]
        VK_FORMAT_G16_B16_R16_3PLANE_444_UNORM = 1000156033,
        #[group(plane2(NumericFormat::Unorm), class_8bit_2plane_444)]
        VK_FORMAT_G8_B8R8_2PLANE_444_UNORM = 1000330000,
        #[group(plane2(NumericFormat::Unorm), class_10bit_2plane_444)]
        VK_FORMAT_G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16 = 1000330001,
        #[group(plane2(NumericFormat::Unorm), class_12bit_2plane_444)]
        VK_FORMAT_G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16 = 1000330002,
        #[group(plane2(NumericFormat::Unorm), class_16bit_2plane_444)]
        VK_FORMAT_G16_B16R16_2PLANE_444_UNORM = 1000330003,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_A4R4G4B4_UNORM_PACK16 = 1000340000,
        #[group(color(NumericFormat::Unorm), class_16_bit)]
        VK_FORMAT_A4B4G4R4_UNORM_PACK16 = 1000340001,
        #[group(color(NumericFormat::Sfloat), class_astc_4x4)]
        VK_FORMAT_ASTC_4x4_SFLOAT_BLOCK = 1000066000,
        #[group(color(NumericFormat::Sfloat), class_astc_5x4)]
        VK_FORMAT_ASTC_5x4_SFLOAT_BLOCK = 1000066001,
        #[group(color(NumericFormat::Sfloat), class_astc_5x5)]
        VK_FORMAT_ASTC_5x5_SFLOAT_BLOCK = 1000066002,
        #[group(color(NumericFormat::Sfloat), class_astc_6x5)]
        VK_FORMAT_ASTC_6x5_SFLOAT_BLOCK = 1000066003,
        #[group(color(NumericFormat::Sfloat), class_astc_6x6)]
        VK_FORMAT_ASTC_6x6_SFLOAT_BLOCK = 1000066004,
        #[group(color(NumericFormat::Sfloat), class_astc_8x5)]
        VK_FORMAT_ASTC_8x5_SFLOAT_BLOCK = 1000066005,
        #[group(color(NumericFormat::Sfloat), class_astc_8x6)]
        VK_FORMAT_ASTC_8x6_SFLOAT_BLOCK = 1000066006,
        #[group(color(NumericFormat::Sfloat), class_astc_8x8)]
        VK_FORMAT_ASTC_8x8_SFLOAT_BLOCK = 1000066007,
        #[group(color(NumericFormat::Sfloat), class_astc_10x5)]
        VK_FORMAT_ASTC_10x5_SFLOAT_BLOCK = 1000066008,
        #[group(color(NumericFormat::Sfloat), class_astc_10x6)]
        VK_FORMAT_ASTC_10x6_SFLOAT_BLOCK = 1000066009,
        #[group(color(NumericFormat::Sfloat), class_astc_10x8)]
        VK_FORMAT_ASTC_10x8_SFLOAT_BLOCK = 1000066010,
        #[group(color(NumericFormat::Sfloat), class_astc_10x10)]
        VK_FORMAT_ASTC_10x10_SFLOAT_BLOCK = 1000066011,
        #[group(color(NumericFormat::Sfloat), class_astc_12x10)]
        VK_FORMAT_ASTC_12x10_SFLOAT_BLOCK = 1000066012,
        #[group(color(NumericFormat::Sfloat), class_astc_12x12)]
        VK_FORMAT_ASTC_12x12_SFLOAT_BLOCK = 1000066013,
        #[group(color(NumericFormat::Unorm), class_16bit)]
        VK_FORMAT_A1B5G5R5_UNORM_PACK16 = 1000470000,
        #[group(color(NumericFormat::Unorm), class_8bit_alpha)]
        VK_FORMAT_A8_UNORM = 1000470001,
    }
);

impl Format {
    
    #[inline(always)]
    pub const fn as_raw(self) -> i32 {
        self as i32
    }

    /// Constructs [`Format`] from raw `value.
    ///
    /// # Safety
    /// It has to be ensured that transmuting the value to [`Format`] results in a valid [`Format`]
    /// value.
    #[inline(always)]
    pub const unsafe fn from_raw(value: i32) -> Self {
        unsafe {
            core::mem::transmute::<i32, Format>(value)
        }
    }

    #[inline(always)]
    pub fn aspects(self) -> ImageAspects {
        if self.is_in_group_color() {
            return ImageAspects::COLOR
        }
        let is_depth = self.is_in_group_depth();
        let is_stencil = self.is_in_group_stencil();
        if is_depth && is_stencil {
            return ImageAspects::DEPTH | ImageAspects::STENCIL
        }
        if is_depth {
            return ImageAspects::DEPTH
        }
        if is_stencil {
            return ImageAspects::STENCIL
        }
        if self.is_in_group_plane2() {
            return
                ImageAspects::PLANE_0 |
                ImageAspects::PLANE_1
        }
        if self.is_in_group_plane3() {
            return 
                ImageAspects::PLANE_0 |
                ImageAspects::PLANE_1 |
                ImageAspects::PLANE_2
        }
        ImageAspects::empty()
    }

    pub const fn compatibility(self) -> FormatCompatibilityClass {
        snake_case! {
            if self.is_in_group_class_8bit() {
                FormatCompatibilityClass::[!Class8Bit!]
            } else if self.is_in_group_class_16bit() {
                FormatCompatibilityClass::[!Class16Bit!]
            } else if self.is_in_group_class_8bit_alpha() {
                FormatCompatibilityClass::[!Class8BitAlpha!]
            } else if self.is_in_group_class_24bit() {
                FormatCompatibilityClass::[!Class24Bit!]
            } else if self.is_in_group_class_32bit() {
                FormatCompatibilityClass::[!Class32Bit!]
            } else if self.is_in_group_class_48bit() {
                FormatCompatibilityClass::[!Class48Bit!]
            } else if self.is_in_group_class_64bit() {
                FormatCompatibilityClass::[!Class64Bit!]
            } else if self.is_in_group_class_96bit() {
                FormatCompatibilityClass::[!Class96Bit!]
            } else if self.is_in_group_class_128bit() {
                FormatCompatibilityClass::[!Class128Bit!]
            } else if self.is_in_group_class_192bit() {
                FormatCompatibilityClass::[!Class192Bit!]
            } else if self.is_in_group_class_256bit() {
                FormatCompatibilityClass::[!Class256Bit!]
            } else if self.is_in_group_class_d16() {
                FormatCompatibilityClass::[!ClassD16!]
            } else if self.is_in_group_class_d24() {
                FormatCompatibilityClass::[!ClassD24!]
            } else if self.is_in_group_class_d32() {
                FormatCompatibilityClass::[!ClassD32!]
            } else if self.is_in_group_class_s8() {
                FormatCompatibilityClass::[!ClassS8!]
            } else if self.is_in_group_class_d16s8() {
                FormatCompatibilityClass::[!ClassD16s8!]
            } else if self.is_in_group_class_d24s8() {
                FormatCompatibilityClass::[!ClassD24s8!]
            } else if self.is_in_group_class_d32s8() {
                FormatCompatibilityClass::[!ClassD32s8!]
            } else if self.is_in_group_class_bc1_rgb() {
                FormatCompatibilityClass::[!ClassBc1Rgb!]
            } else if self.is_in_group_class_bc1_rgba() {
                FormatCompatibilityClass::[!ClassBc1Rgba!]
            } else if self.is_in_group_class_bc2() {
                FormatCompatibilityClass::[!ClassBc2!]
            } else if self.is_in_group_class_bc3() {
                FormatCompatibilityClass::[!ClassBc3!]
            } else if self.is_in_group_class_bc4() {
                FormatCompatibilityClass::[!ClassBc4!]
            } else if self.is_in_group_class_bc5() {
                FormatCompatibilityClass::[!ClassBc5!]
            } else if self.is_in_group_class_bc6h() {
                FormatCompatibilityClass::[!ClassBc6h!]
            } else if self.is_in_group_class_bc7() {
                FormatCompatibilityClass::[!ClassBc7!]
            } else if self.is_in_group_class_etc2_rgb() {
                FormatCompatibilityClass::[!ClassEtc2Rgb!]
            } else if self.is_in_group_class_etc2_rgba() {
                FormatCompatibilityClass::[!ClassEtc2Rgba!]
            } else if self.is_in_group_class_etc2_eac_rgba() {
                FormatCompatibilityClass::[!ClassEtc2EacRgba!]
            } else if self.is_in_group_class_eac_r() {
                FormatCompatibilityClass::[!ClassEacR!]
            } else if self.is_in_group_class_eac_r() {
                FormatCompatibilityClass::[!ClassEacRG!]
            } else if self.is_in_group_class_astc_4x4() {
                FormatCompatibilityClass::[!ClassAstc4x4!]
            } else if self.is_in_group_class_astc_5x4() {
                FormatCompatibilityClass::[!ClassAstc5x4!]
            } else if self.is_in_group_class_astc_5x5() {
                FormatCompatibilityClass::[!ClassAstc5x5!]
            } else if self.is_in_group_class_astc_6x5() {
                FormatCompatibilityClass::[!ClassAstc6x5!]
            } else if self.is_in_group_class_astc_6x6() {
                FormatCompatibilityClass::[!ClassAstc6x6!]
            } else if self.is_in_group_class_astc_8x5() {
                FormatCompatibilityClass::[!ClassAstc8x5!]
            } else if self.is_in_group_class_astc_8x6() {
                FormatCompatibilityClass::[!ClassAstc8x6!]
            } else if self.is_in_group_class_astc_8x8() {
                FormatCompatibilityClass::[!ClassAstc8x8!]
            } else if self.is_in_group_class_astc_10x5() {
                FormatCompatibilityClass::[!ClassAstc10x5!]
            } else if self.is_in_group_class_astc_10x6() {
                FormatCompatibilityClass::[!ClassAstc10x6!]
            } else if self.is_in_group_class_astc_10x8() {
                FormatCompatibilityClass::[!ClassAstc10x8!]
            } else if self.is_in_group_class_astc_10x10() {
                FormatCompatibilityClass::[!ClassAstc10x10!]
            } else if self.is_in_group_class_astc_12x10() {
                FormatCompatibilityClass::[!ClassAstc12x10!]
            } else if self.is_in_group_class_astc_12x12() {
                FormatCompatibilityClass::[!ClassAstc12x12!]
            } else if self.is_in_group_class_32bit_g8b8g8r8() {
                FormatCompatibilityClass::[!Class32BitG8b8g8r8!]
            } else if self.is_in_group_class_32bit_b8g8r8g8() {
                FormatCompatibilityClass::[!Class32BitB8g8r8g8!]
            } else if self.is_in_group_class_8bit_3plane_420() {
                FormatCompatibilityClass::[!Class8Bit3Plane420!]
            } else if self.is_in_group_class_8bit_2plane_420() {
                FormatCompatibilityClass::[!Class8Bit2Plane420!]
            } else if self.is_in_group_class_8bit_3plane_422() {
                FormatCompatibilityClass::[!Class8Bit3Plane422!]
            } else if self.is_in_group_class_8bit_2plane_422() {
                FormatCompatibilityClass::[!Class8Bit2Plane422!]
            } else if self.is_in_group_class_8bit_3plane_444() {
                FormatCompatibilityClass::[!Class8Bit3Plane444!]
            } else if self.is_in_group_class_8bit_2plane_444() {
                FormatCompatibilityClass::[!Class8Bit2Plane444!]
            } else if self.is_in_group_class_64bit_r10g10b10a10() {
                FormatCompatibilityClass::[!Class64BitR10g10b10a10!]
            } else if self.is_in_group_class_64bit_g10b10g10r10() {
                FormatCompatibilityClass::[!Class64BitG10b10g10r10!]
            } else if self.is_in_group_class_64bit_b10g10r10g10() {
                FormatCompatibilityClass::[!Class64BitB10g10r10g10!]
            } else if self.is_in_group_class_10bit_3plane_420() {
                FormatCompatibilityClass::[!Class10Bit3Plane420!]
            } else if self.is_in_group_class_10bit_2plane_420() {
                FormatCompatibilityClass::[!Class10Bit2Plane420!]
            } else if self.is_in_group_class_10bit_3plane_422() {
                FormatCompatibilityClass::[!Class10Bit3Plane422!]
            } else if self.is_in_group_class_10bit_2plane_422() {
                FormatCompatibilityClass::[!Class10Bit2Plane422!]
            } else if self.is_in_group_class_10bit_3plane_422() {
                FormatCompatibilityClass::[!Class10Bit3Plane444!]
            } else if self.is_in_group_class_10bit_2plane_422() {
                FormatCompatibilityClass::[!Class10Bit2Plane444!]
            } else if self.is_in_group_class_64bit_r12g12b12a12() {
                FormatCompatibilityClass::[!Class64BitR12g12b12a12!]
            } else if self.is_in_group_class_64bit_g12b12g12r12() {
                FormatCompatibilityClass::[!Class64BitG12b12g12r12!]
            } else if self.is_in_group_class_12bit_3plane_420() {
                FormatCompatibilityClass::[!Class12Bit3Plane420!]
            } else if self.is_in_group_class_12bit_2plane_420() {
                FormatCompatibilityClass::[!Class12Bit2Plane420!]
            } else if self.is_in_group_class_12bit_3plane_422() {
                FormatCompatibilityClass::[!Class12Bit3Plane422!]
            } else if self.is_in_group_class_12bit_2plane_422() {
                FormatCompatibilityClass::[!Class12Bit2Plane422!]
            } else if self.is_in_group_class_12bit_3plane_444() {
                FormatCompatibilityClass::[!Class12Bit3Plane444!]
            } else if self.is_in_group_class_12bit_2plane_444() {
                FormatCompatibilityClass::[!Class12Bit2Plane444!]
            } else if self.is_in_group_class_64bit_g16b16g16r16() {
                FormatCompatibilityClass::[!Class64BitG16b16g16r16!]
            } else if self.is_in_group_class_64bit_b16g16r16g16() {
                FormatCompatibilityClass::[!Class64BitB16g16r16g16!]
            } else if self.is_in_group_class_64bit_b12g12r12g12() {
                FormatCompatibilityClass::[!Class64BitB12g12r12g12!]
            } else if self.is_in_group_class_16bit_3plane_420() {
                FormatCompatibilityClass::[!Class16Bit3Plane420!]
            } else if self.is_in_group_class_16bit_2plane_420() {
                FormatCompatibilityClass::[!Class16Bit2Plane420!]
            } else if self.is_in_group_class_16bit_3plane_422() {
                FormatCompatibilityClass::[!Class16Bit3Plane422!]
            } else if self.is_in_group_class_16bit_2plane_422() {
                FormatCompatibilityClass::[!Class16Bit2Plane422!]
            } else if self.is_in_group_class_16bit_3plane_444() {
                FormatCompatibilityClass::[!Class16Bit3Plane444!]
            } else if self.is_in_group_class_16bit_2plane_444() {
                FormatCompatibilityClass::[!Class16Bit2Plane444!]
            } else {
                FormatCompatibilityClass::[!Undefined!]
            }
        }
    }
    
    #[inline(always)]
    pub fn is_compatible_with(self, other: Self) -> bool {
        self.compatibility() == other.compatibility()
    }

    #[inline(always)]
    pub const fn texel_block_size(self) -> DeviceSize {
        self.compatibility().texel_block_size()
    }

    #[inline(always)]
    pub const fn texel_block_extent(self) -> Dimensions {
        self.compatibility().texel_block_extent()
    }

    #[inline(always)]
    pub fn numeric_format_color(self) -> Option<NumericFormat> {
        self.group_color_value()
    }

    #[inline(always)]
    pub fn numeric_format_depth(self) -> Option<NumericFormat> {
        self.group_depth_value()
    }

    #[inline(always)]
    pub fn numeric_format_stencil(self) -> Option<NumericFormat> {
        self.group_stencil_value()
    }

    #[inline(always)]
    pub fn is_multi_planar(&self) -> bool {
        self.is_in_group_plane2() ||
        self.is_in_group_plane3()
    }

    #[inline(always)]
    pub fn is_depth_stencil(&self) -> bool {
        self.is_in_group_depth() ||
        self.is_in_group_stencil()
    }

    /// Returns which single-plane formats are compatible with the relative plane index in the
    /// returned array.
    ///
    /// For 2-plane formats, the last index is [`Format::Undefined`].
    ///
    /// For single-plane formats, index 0 is [`self`] and indices 1 and 2 are [Format::Undefined`].
    #[inline(always)]
    pub fn plane_formats(self) -> [Self; 3] {
        match self {
            Self::G8B8R83plane420Unorm =>
                [Format::R8Unorm; 3],
            Self::G8B8r82plane420Unorm =>
                [Format::R8Unorm, Format::R8g8Unorm, Format::Undefined],
            Self::G8B8R83plane422Unorm =>
                [Format::R8Unorm; 3],
            Self::G8B8r82plane422Unorm =>
                [Format::R8Unorm, Format::R8g8Unorm, Format::Undefined],
            Self::G8B8R83plane444Unorm =>
                [Format::R8Unorm; 3],
            Self::G8B8r82plane444Unorm =>
                [Format::R8Unorm, Format::R8g8Unorm, Format::Undefined],
            Self::G10x6B10x6R10x63plane420Unorm3pack16 =>
                [Format::R10x6UnormPack16; 3],
            Self::G10x6B10x6r10x62plane420Unorm3pack16 =>
                [Format::R10x6UnormPack16, Format::R10x6g10x6Unorm2pack16, Format::Undefined],
            Self::G10x6B10x6R10x63plane422Unorm3pack16 =>
                [Format::R10x6UnormPack16; 3],
            Self::G10x6B10x6r10x62plane422Unorm3pack16 =>
                [Format::R10x6UnormPack16, Format::R10x6g10x6Unorm2pack16, Format::Undefined],
            Self::G10x6B10x6R10x63plane444Unorm3pack16 =>
                [Format::R10x6UnormPack16; 3],
            Self::G10x6B10x6r10x62plane444Unorm3pack16 =>
                [Format::R10x6UnormPack16, Format::R10x6g10x6Unorm2pack16, Format::Undefined],
            Self::G12x4B12x4R12x43plane420Unorm3pack16 =>
                [Format::R12x4UnormPack16; 3],
            Self::G12x4B12x4r12x42plane420Unorm3pack16 =>
                [Format::R12x4UnormPack16, Format::R12x4g12x4Unorm2pack16, Format::Undefined],
            Self::G12x4B12x4R12x43plane422Unorm3pack16 =>
                [Format::R12x4UnormPack16; 3],
            Self::G12x4B12x4r12x42plane422Unorm3pack16 =>
                [Format::R12x4UnormPack16, Format::R12x4g12x4Unorm2pack16, Format::Undefined],
            Self::G12x4B12x4R12x43plane444Unorm3pack16 =>
                [Format::R12x4UnormPack16; 3],
            Self::G12x4B12x4r12x42plane444Unorm3pack16 =>
                [Format::R12x4UnormPack16, Format::R12x4g12x4Unorm2pack16, Format::Undefined],
            Self::G16B16R163plane420Unorm =>
                [Format::R16Unorm; 3],
            Self::G16B16r162plane420Unorm =>
                [Format::R16Unorm, Format::R16g16Unorm, Format::Undefined],
            Self::G16B16R163plane422Unorm =>
                [Format::R16Unorm; 3],
            Self::G16B16r162plane422Unorm =>
                [Format::R16Unorm, Format::R16g16Unorm, Format::Undefined],
            Self::G16B16R163plane444Unorm =>
                [Format::R16Unorm; 3],
            Self::G16B16r162plane444Unorm =>
                [Format::R16Unorm, Format::R16g16Unorm, Format::Undefined],
            _ => [self, Format::Undefined, Format::Undefined],
        }
    }

    #[inline(always)]
    pub fn resolve_modes(self) -> FormatResolveModes {
        let is_depth = self.is_in_group_depth();
        let is_stencil = self.is_in_group_stencil();
        if is_depth && is_stencil {
            FormatResolveModes {
                depth:
                    ResolveModes::AVERAGE |
                    ResolveModes::MIN |
                    ResolveModes::MAX |
                    ResolveModes::SAMPLE_ZERO
                ,
                stencil:
                    ResolveModes::MIN |
                    ResolveModes::MAX |
                    ResolveModes::SAMPLE_ZERO,
                ..Default::default()
            } 
        } else if is_depth {
            FormatResolveModes {
                depth:
                    ResolveModes::AVERAGE |
                    ResolveModes::MIN |
                    ResolveModes::MAX |
                    ResolveModes::SAMPLE_ZERO
                ,
                ..Default::default()
            }
        } else if is_stencil {
            FormatResolveModes {
                stencil:
                    ResolveModes::MIN |
                    ResolveModes::MAX |
                    ResolveModes::SAMPLE_ZERO,
                ..Default::default()
            }
        } else if let Some(numeric_format) = self.numeric_format_color() {
            if numeric_format.is_floating_point() {
                FormatResolveModes {
                    color: ResolveModes::AVERAGE,
                    ..Default::default()
                }
            } else {
                FormatResolveModes {
                    color: ResolveModes::SAMPLE_ZERO,
                    ..Default::default()
                }
            }
        } else {
            Default::default()
        }
    }
}

impl From<Format> for vk::Format {

    #[inline(always)]
    fn from(value: Format) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl Display for Format {

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}
