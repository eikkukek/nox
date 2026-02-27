use nox_ash::{
    vk,
    ash_style_enum,
};

use nox_mem::{AsRaw, Display};

pub type Flags32 = u32;
pub type Flags64 = u64;

ash_style_enum! {

    /// Specifies a bitmask of multisample anti-aliasing sample counts
    ///
    /// Default value is [`MSAA::X1`].
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkSampleCountFlagBits.html>**
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
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkBufferUsageFlagBits.html>**
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum BufferUsageFlags {
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
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageUsageFlagBits.html>**
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum ImageUsageFlags {
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
    }

    /// Specifies which image aspect to use for e.g. [`ImageSubresourceRange`].
    ///
    /// Default value is [`ImageAspect::empty()`].
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkImageAspectFlagBits.html#>**
    #[flags(Flags32)]
    #[default = Self::empty()]
    pub enum ImageAspectFlags {
        #[display("color")]
        COLOR = vk::ImageAspectFlags::COLOR.as_raw(),
        #[display("depth")]
        DEPTH = vk::ImageAspectFlags::DEPTH.as_raw(),
        #[display("stencil")]
        STENCIL = vk::ImageAspectFlags::STENCIL.as_raw(),
    }

    /// Specifies sets of stencil state for which to update operations.
    ///
    /// # Vulkan docs
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkStencilFaceFlagBits.html>**
    #[flags(Flags32)]
    pub enum StencilFaceFlags {
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
    /// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkCullModeFlagBits.html>**
    #[flags(Flags32)]
    #[default = Self::FRONT]
    pub enum CullModeFlags {
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
}

/// Specifies how a component is swizzled
///
/// Default value is [`ComponentSwizzle::Identity`].
/// # Vulkan docs
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkComponentSwizzle.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkFilter.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerMipmapMode.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkSamplerAddressMode.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkBorderColor.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkCompareOp.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkIndexType.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessBufferBehavior.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineRobustnessImageBehavior.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkDynamicState.html>**
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
    #[display("depth bias enable")]
    DepthBiasEnable =  vk::DynamicState::DEPTH_BIAS_ENABLE.as_raw(),
    #[display("primitive restart enable")]
    PrimitiveRestartEnable = vk::DynamicState::PRIMITIVE_RESTART_ENABLE.as_raw(),
}

/// Specifies polygon front-facing orientation.
///
/// The default value is [`FrontFace::CounterClockwise`].
/// 
/// # Vulkan docs
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkFrontFace.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkPrimitiveTopology.html>**
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
/// **<https://docs.vulkan.org/refpages/latest/refpages/source/VkStencilOp.html>**
#[repr(i32)]
#[derive(Copy, PartialEq, Eq, AsRaw, Debug, Display)]
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

impl BlendFactor {

    pub(crate) fn from_vk(value: vk::BlendFactor) -> Option<Self> {
        match value.as_raw() {
            x if x == Self::Zero as i32 => Some(Self::Zero),
            x if x == Self::One as i32 => Some(Self::One),
            x if x == Self::SrcColor as i32 => Some(Self::SrcColor),
            x if x == Self::OneMinusSrcColor as i32 => Some(Self::OneMinusSrcColor),
            x if x == Self::DstColor as i32 => Some(Self::DstColor),
            x if x == Self::OneMinusDstColor as i32 => Some(Self::OneMinusDstColor),
            x if x == Self::SrcAlpha as i32 => Some(Self::SrcAlpha),
            x if x == Self::OneMinusSrcAlpha as i32 => Some(Self::OneMinusSrcAlpha),
            x if x == Self::DstAlpha as i32 => Some(Self::DstAlpha),
            x if x == Self::OneMinusDstAlpha as i32 => Some(Self::OneMinusDstAlpha),
            x if x == Self::ConstColor as i32 => Some(Self::ConstColor),
            x if x == Self::OneMinusConstColor as i32 => Some(Self::OneMinusConstColor),
            x if x == Self::ConstAlpha as i32 => Some(Self::ConstAlpha),
            x if x == Self::OneMinusConstAlpha as i32 => Some(Self::OneMinusConstAlpha),
            _ => None,
        }
    }
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

impl BlendOp {

    pub(crate) fn from_vk(value: vk::BlendOp) -> Option<Self> {
        match value.as_raw() {
            x if x == Self::Add as i32 => Some(Self::Add),
            x if x == Self::Sub as i32 => Some(Self::Sub),
            x if x == Self::SubRev as i32 => Some(Self::SubRev),
            x if x == Self::Min as i32 => Some(Self::Min),
            x if x == Self::Max as i32 => Some(Self::Max),
            _ => None,
        }
    }
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
    [BufferUsageFlags, vk::BufferCreateFlags],
    [ImageUsageFlags, vk::ImageUsageFlags],
    [ImageAspectFlags, vk::ImageAspectFlags],
    [ComponentSwizzle, vk::ComponentSwizzle],
    [Filter, vk::Filter],
    [MipmapMode, vk::SamplerMipmapMode],
    [SamplerAddressMode, vk::SamplerAddressMode],
    [BorderColor, vk::BorderColor],
    [BlendFactor, vk::BlendFactor],
    [BlendOp, vk::BlendOp],
    [CompareOp, vk::CompareOp],
    [StencilFaceFlags, vk::StencilFaceFlags],
    [StencilOp, vk::StencilOp],
    [PolygonMode, vk::PolygonMode],
    [DynamicState, vk::DynamicState],
    [IndexType, vk::IndexType],
    [PipelineRobustnessBufferBehavior, vk::PipelineRobustnessBufferBehavior],
    [PipelineRobustnessImageBehavior, vk::PipelineRobustnessImageBehavior],
}
