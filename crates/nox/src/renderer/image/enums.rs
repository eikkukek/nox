use ash::vk;

use nox_mem::{AsRaw, impl_as_raw_bit_op};

pub use vk::ImageAspectFlags as ImageAspectFlags;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ImageUsage {
    TransferSrc = vk::ImageUsageFlags::TRANSFER_SRC.as_raw(),
    TransferDst = vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
    Sampled = vk::ImageUsageFlags::SAMPLED.as_raw(),
    Storage = vk::ImageUsageFlags::STORAGE.as_raw(),
    ColorAttachment = vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw(),
    DepthStencilAttachment = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT.as_raw(),
}

impl From<ImageUsage> for vk::ImageUsageFlags {

    fn from(value: ImageUsage) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ImageAspect {
    Color = vk::ImageAspectFlags::COLOR.as_raw(),
    Depth = vk::ImageAspectFlags::DEPTH.as_raw(),
    Stencil = vk::ImageAspectFlags::STENCIL.as_raw(),
}

impl From<ImageAspect> for vk::ImageAspectFlags {

    fn from(value: ImageAspect) -> Self {
        Self::from_raw(value.as_raw())
    }
}

impl_as_raw_bit_op!(ImageAspect, ImageUsage);

pub trait Format: Copy + AsRaw<Repr = i32> {

    fn aspects(self) -> &'static [ImageAspect];

    fn as_vk_format(self) -> vk::Format {
        vk::Format::from_raw(self.as_raw())
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ColorFormat {
    UnormR8 = vk::Format::R8_UNORM.as_raw(),
    UnormRG8 = vk::Format::R8G8_UNORM.as_raw(),
    UnormRGB8 = vk::Format::R8G8B8_UNORM.as_raw(),
    UnormRGBA8 = vk::Format::R8G8B8A8_UNORM.as_raw(),
    SrgbR8 = vk::Format::R8_SRGB.as_raw(),
    SrgbRG8 = vk::Format::R8G8_SRGB.as_raw(),
    SrgbRGB8 = vk::Format::R8G8B8_SRGB.as_raw(),
    SrgbRGBA8 = vk::Format::R8G8B8A8_SRGB.as_raw(),
}

impl Format for ColorFormat {

    fn aspects(self) -> &'static [ImageAspect] {
        &[ImageAspect::Color]
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum DepthFormat {
    D32 = vk::Format::D32_SFLOAT.as_raw(),
    D16 = vk::Format::D16_UNORM.as_raw(),
    S8 = vk::Format::S8_UINT.as_raw(),
    D32S8 = vk::Format::D32_SFLOAT_S8_UINT.as_raw(),
    D24S8 = vk::Format::D24_UNORM_S8_UINT.as_raw(),
}

impl DepthFormat {

    pub fn all_depth() -> &'static [DepthFormat] {
        &[
            DepthFormat::D32,
            DepthFormat::D16,
            DepthFormat::D32S8,
            DepthFormat::D24S8,
        ]
    }

    pub fn all_stencil() -> &'static [DepthFormat] {
        &[
            DepthFormat::S8,
            DepthFormat::D32S8,
            DepthFormat::D24S8,
        ]
    }
}

impl Format for DepthFormat {

    fn aspects(self) -> &'static [ImageAspect] {
        match self {
            Self::D32 | Self::D16 => {
                &[ImageAspect::Depth]
            },
            Self::S8 => {
                &[ImageAspect::Stencil]
            },
            Self::D32S8 | Self::D24S8 => {
                &[
                    ImageAspect::Depth,
                    ImageAspect::Stencil,
                ]
            }
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum IntegerFormat {
    UIntR8 = vk::Format::R8_UINT.as_raw(),
    UIntRG8 = vk::Format::R8G8_UINT.as_raw(),
    UIntRGB8 = vk::Format::R8G8B8_UINT.as_raw(),
    UIntRGBA8 = vk::Format::R8G8B8A8_UINT.as_raw(),
    IntR8 = vk::Format::R8_SINT.as_raw(),
    IntRG8 = vk::Format::R8G8_SINT.as_raw(),
    IntRGB8 = vk::Format::R8G8B8_SINT.as_raw(),
    IntRGBA8 = vk::Format::R8G8B8A8_SINT.as_raw(),
}

impl Format for IntegerFormat {

    fn aspects(self) -> &'static [ImageAspect] {
        &[ImageAspect::Color]
    }
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum FormatFeature {
    SampledImage = vk::FormatFeatureFlags::SAMPLED_IMAGE.as_raw(),
    StorageImage = vk::FormatFeatureFlags::STORAGE_IMAGE.as_raw(),
    ColorAttachment = vk::FormatFeatureFlags::COLOR_ATTACHMENT.as_raw(),
    DepthStencilAttachment = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT.as_raw(),
    TransferSrc = vk::FormatFeatureFlags::TRANSFER_SRC.as_raw(),
    TransferDst = vk::FormatFeatureFlags::TRANSFER_DST.as_raw(),
}

impl_as_raw_bit_op!(FormatFeature);

impl From<FormatFeature> for u32 {

    fn from(value: FormatFeature) -> Self {
        value as Self
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum ComponentSwizzle {
    #[default]
    Identity = vk::ComponentSwizzle::IDENTITY.as_raw(),
    Zero = vk::ComponentSwizzle::ZERO.as_raw(),
    One = vk::ComponentSwizzle::ONE.as_raw(),
    R = vk::ComponentSwizzle::R.as_raw(),
    G = vk::ComponentSwizzle::G.as_raw(),
    B = vk::ComponentSwizzle::B.as_raw(),
    A = vk::ComponentSwizzle::A.as_raw(),
}

impl From<ComponentSwizzle> for vk::ComponentSwizzle {

    fn from(value: ComponentSwizzle) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum Filter {
    #[default]
    Nearest = vk::Filter::NEAREST.as_raw(),
    Linear = vk::Filter::LINEAR.as_raw(),
}

impl From<Filter> for vk::Filter {

    fn from(value: Filter) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum MipMode {
    #[default]
    Nearest = vk::SamplerMipmapMode::NEAREST.as_raw(),
    Linear = vk::SamplerMipmapMode::LINEAR.as_raw(),
}

impl From<MipMode> for vk::SamplerMipmapMode {

    fn from(value: MipMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum AddressMode {
    Repeat = vk::SamplerAddressMode::REPEAT.as_raw(),
    MirroredRepeat = vk::SamplerAddressMode::MIRRORED_REPEAT.as_raw(),
    ClampToEdge = vk::SamplerAddressMode::CLAMP_TO_EDGE.as_raw(),
    #[default]
    ClampToBorder = vk::SamplerAddressMode::CLAMP_TO_BORDER.as_raw(),
}

impl From<AddressMode> for vk::SamplerAddressMode {

    fn from(value: AddressMode) -> Self {
        Self::from_raw(value.as_raw())
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum BorderColor {
    #[default]
    FloatTransparentBlack = vk::BorderColor::FLOAT_TRANSPARENT_BLACK.as_raw(),
    IntTransparentBlack = vk::BorderColor::INT_TRANSPARENT_BLACK.as_raw(),
    FloatOpaqueBlack = vk::BorderColor::FLOAT_OPAQUE_BLACK.as_raw(),
    IntOpaqueBlack = vk::BorderColor::INT_OPAQUE_BLACK.as_raw(),
    FloatOpaqueWhite = vk::BorderColor::FLOAT_OPAQUE_WHITE.as_raw(),
    IntOpaqueWhite = vk::BorderColor::INT_OPAQUE_WHITE.as_raw(),
}

impl From<BorderColor> for vk::BorderColor {

    fn from(value: BorderColor) -> Self {
        Self::from_raw(value.as_raw())
    }
}
