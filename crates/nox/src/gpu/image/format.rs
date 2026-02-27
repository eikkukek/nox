use nox_proc::Display;

use nox_mem::{AsRaw, impl_as_raw_bit_op};

use nox_ash::vk;

pub use vk::Format as VkFormat;
pub use vk::ResolveModeFlags as VkResolveModeFlags;

use crate::gpu::ImageAspect;

#[derive(Clone, Copy, Debug, Display)]
pub enum ResolveAspect {
    #[display("Color")]
    Color,
    #[display("Depth")]
    Depth,
    #[display("Stencil")]
    Stencil,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FormatResolveModes {
    pub color: VkResolveModeFlags,
    pub depth: VkResolveModeFlags,
    pub stencil: VkResolveModeFlags,
}

impl FormatResolveModes {

    #[inline(always)]
    pub fn resolve_modes(self, aspect: ResolveAspect) -> VkResolveModeFlags {
        match aspect {
            ResolveAspect::Color => self.color,
            ResolveAspect::Depth => self.depth,
            ResolveAspect::Stencil => self.stencil,
        }
    }
}

pub trait Format: Copy + AsRaw<Repr = i32> {

    fn as_vk_format(self) -> VkFormat {
        vk::Format::from_raw(self.as_raw())
    }

    fn aspects(self) -> ImageAspect;

    fn resolve_modes(self) -> FormatResolveModes;
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RawFormat(pub VkFormat, pub ImageAspect, pub FormatResolveModes);

impl Format for RawFormat {

    fn as_vk_format(self) -> VkFormat {
        self.0
    }

    fn aspects(self) -> ImageAspect {
        self.1
    }

    fn resolve_modes(self) -> FormatResolveModes {
        self.2
    }
}

impl AsRaw for RawFormat {

    type Repr = i32;

    fn as_raw(self) -> Self::Repr {
        self.0.as_raw()
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum NormFormat {
    #[default]
    Undefined = vk::Format::UNDEFINED.as_raw(),
    UnormR8 = vk::Format::R8_UNORM.as_raw(),
    UnormRG8 = vk::Format::R8G8_UNORM.as_raw(),
    UnormRGB8 = vk::Format::R8G8B8_UNORM.as_raw(),
    UnormRGBA8 = vk::Format::R8G8B8A8_UNORM.as_raw(),
    SnormR8 = vk::Format::R8_SNORM.as_raw(),
    SnormRG8 = vk::Format::R8G8_SNORM.as_raw(),
    SnormRGB8 = vk::Format::R8G8B8_SNORM.as_raw(),
    SnormRGBA8 = vk::Format::R8G8B8A8_SNORM.as_raw(),
    SrgbR8 = vk::Format::R8_SRGB.as_raw(),
    SrgbRG8 = vk::Format::R8G8_SRGB.as_raw(),
    SrgbRGB8 = vk::Format::R8G8B8_SRGB.as_raw(),
    SrgbRGBA8 = vk::Format::R8G8B8A8_SRGB.as_raw(),
}

impl Format for NormFormat {

    #[inline(always)]
    fn aspects(self) -> ImageAspect {
        if self == Self::Undefined {
            ImageAspect::empty()
        }
        else {
            ImageAspect::COLOR
        }
    }

    #[inline(always)]
    fn resolve_modes(self) -> FormatResolveModes {
        FormatResolveModes {
            color: vk::ResolveModeFlags::AVERAGE,
            ..Default::default()
        }
    }
}

impl NormFormat {

    #[inline(always)]
    pub fn is_srgb(self) -> bool {
        matches!(self,
            Self::SrgbR8 |
            Self::SrgbRG8 |
            Self::SrgbRGB8 |
            Self::SrgbRGBA8,
        )
    }
}

#[repr(i32)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, AsRaw, Debug)]
pub enum DepthStencilFormat {
    #[default]
    Undefined = vk::Format::UNDEFINED.as_raw(),
    D32 = vk::Format::D32_SFLOAT.as_raw(),
    D16 = vk::Format::D16_UNORM.as_raw(),
    S8 = vk::Format::S8_UINT.as_raw(),
    D32S8 = vk::Format::D32_SFLOAT_S8_UINT.as_raw(),
    D24S8 = vk::Format::D24_UNORM_S8_UINT.as_raw(),
}

impl DepthStencilFormat {

    pub fn all_depth() -> &'static [DepthStencilFormat] {
        &[
            DepthStencilFormat::D32,
            DepthStencilFormat::D16,
            DepthStencilFormat::D32S8,
            DepthStencilFormat::D24S8,
        ]
    }

    pub fn all_stencil() -> &'static [DepthStencilFormat] {
        &[
            DepthStencilFormat::S8,
            DepthStencilFormat::D32S8,
            DepthStencilFormat::D24S8,
        ]
    }

    pub fn all_depth_stencil() -> &'static [DepthStencilFormat] {
        &[
            DepthStencilFormat::D32S8,
            DepthStencilFormat::D24S8,
        ]
    }
}

impl Format for DepthStencilFormat {

    #[inline(always)]
    fn aspects(self) -> ImageAspect {
        match self {
            Self::Undefined => {
                ImageAspect::empty()
            }
            Self::D32 | Self::D16 => {
                ImageAspect::DEPTH
            },
            Self::S8 => {
                ImageAspect::STENCIL
            },
            Self::D32S8 | Self::D24S8 => {
                ImageAspect::DEPTH |
                ImageAspect::STENCIL
            }
        }
    }

    #[inline(always)]
    fn resolve_modes(self) -> FormatResolveModes {
        FormatResolveModes {
            depth: match self {
                Self::D32 | Self::D16 | Self::D32S8 | Self::D24S8 =>
                    vk::ResolveModeFlags::AVERAGE |
                    vk::ResolveModeFlags::MIN | vk::ResolveModeFlags::MAX |
                    vk::ResolveModeFlags::SAMPLE_ZERO,
                _ => vk::ResolveModeFlags::NONE,
            },
            stencil: match self {
                Self::S8 | Self::D32S8 | Self::D24S8 =>
                    vk::ResolveModeFlags::MIN | vk::ResolveModeFlags::MAX |
                    vk::ResolveModeFlags::SAMPLE_ZERO,
                _ => vk::ResolveModeFlags::NONE
            },
            ..Default::default()
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum IntegerFormat {
    Undefined = vk::Format::UNDEFINED.as_raw(),
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

    #[inline(always)]
    fn aspects(self) -> ImageAspect {
        ImageAspect::COLOR
    }

    #[inline(always)]
    fn resolve_modes(self) -> FormatResolveModes {
        FormatResolveModes {
            color: match self {
                Self::Undefined => vk::ResolveModeFlags::NONE,
                _ => vk::ResolveModeFlags::SAMPLE_ZERO,
            },
            ..Default::default()
        }
        
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, AsRaw)]
pub enum FloatFormat {
    R32 = vk::Format::R32_SFLOAT.as_raw(),
    RG32 = vk::Format::R32G32_SFLOAT.as_raw(),
    RGB32 = vk::Format::R32G32B32_SFLOAT.as_raw(),
    RGBA32 = vk::Format::R32G32B32A32_SFLOAT.as_raw(),
}

impl Format for FloatFormat {

    #[inline(always)]
    fn aspects(self) -> ImageAspect {
        ImageAspect::COLOR
    }

    #[inline(always)]
    fn resolve_modes(self) -> FormatResolveModes {
        Default::default()
    }
}
