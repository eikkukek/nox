use ash::vk;

use nox_mem::{AsRaw, impl_as_raw_bit_op};

pub use vk::ImageAspectFlags as ImageAspectFlags;

#[repr(u32)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
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
#[derive(Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
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
#[derive(Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
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
#[derive(Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
pub enum DepthFormat {
    D32 = vk::Format::D32_SFLOAT.as_raw(),
    D16 = vk::Format::D16_UNORM.as_raw(),
    S8 = vk::Format::S8_UINT.as_raw(),
    D32S8 = vk::Format::D32_SFLOAT_S8_UINT.as_raw(),
    D24S8 = vk::Format::D24_UNORM_S8_UINT.as_raw(),
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
#[derive(Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
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

#[repr(i32)]
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, AsRaw)]
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
