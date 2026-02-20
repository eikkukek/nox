use nox_ash::vk;

use nox_mem::{AsRaw, impl_as_raw_bit_op};

impl_as_raw_bit_op!(ImageAspect, ImageUsage);

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
    #[default]
    Repeat = vk::SamplerAddressMode::REPEAT.as_raw(),
    MirroredRepeat = vk::SamplerAddressMode::MIRRORED_REPEAT.as_raw(),
    ClampToEdge = vk::SamplerAddressMode::CLAMP_TO_EDGE.as_raw(),
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
