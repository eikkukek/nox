use ash::vk;

use nox_mem::slot_map::SlotMapError;

use crate::Offset3D;

use super::*;

#[derive(Clone, Copy, Debug)]
pub enum ImageError {
    SlotMapError(SlotMapError),
    VulkanError(vk::Result),
    AspectMismatch,
    SubresourceOutOfRange {
        image_mip_levels: u32, base_level: u32, level_count: u32,
        image_array_layers: u32, base_layer: u32, layer_count: u32,
    },
    ImmutableFormat {
        image_format: vk::Format,
        requested_format: vk::Format,
    },
    UsageMismatch {
        missing_usage: vk::ImageUsageFlags,
    },
    InvalidCopy {
        image_dimensions: Dimensions,
        copy_offset: Offset3D,
        copy_dimensions: Dimensions,
    },
    InvalidCubeMap {
        layer_count: u32,
    },
}

impl core::fmt::Display for ImageError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SlotMapError(err) => write!(f, "{err}"),
            Self::VulkanError(err) => write!(f, "{err}"),
            Self::AspectMismatch => write!(f, "aspect mismatch"),
            Self::SubresourceOutOfRange {
                image_mip_levels, base_level, level_count,
                image_array_layers, base_layer, layer_count,
            } => {
                write!(f,
                    "subresource (base level {base_level}, level count {level_count}, base layer {base_layer}, layer count {layer_count}) out of range with image mip levels {image_mip_levels} and array layers {image_array_layers}"
                )
            },
            Self::ImmutableFormat { image_format, requested_format } => write!(f, "image has immutable format {:?}, requested format is {:?}", image_format, requested_format),
            Self::UsageMismatch { missing_usage } => write!(f, "usage mismatch, missing usage {:?}", missing_usage),
            Self::InvalidCopy { image_dimensions, copy_offset, copy_dimensions } =>
                write!(f, "invalid copy (copy offset {copy_offset}, copy dimensions {copy_dimensions}) with image dimensions {image_dimensions}"),
            Self::InvalidCubeMap { layer_count } => write!(f, "invalid cube map with layer count {layer_count}")

        }
    }
}

impl core::error::Error for ImageError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SlotMapError(err) => Some(err),
            Self::VulkanError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<SlotMapError> for ImageError {

    fn from(value: SlotMapError) -> Self {
        Self::SlotMapError(value)
    }
}

impl From<vk::Result> for ImageError {

    fn from(value: vk::Result) -> Self {
        Self::VulkanError(value)
    }
}
