use ash::vk;

use nox_mem::slot_map::SlotMapError;

use crate::gpu::Offset3D;

use crate::dev::error::Error;

use super::*;

#[derive(Error, Debug)]
pub enum ImageError {
    #[display("vulkan error")]
    VulkanError(#[from] #[source] vk::Result),
    #[display("aspect mismatch")]
    AspectMismatch,
    #[display("subresource (base level {base_level}, level count {level_count}, base layer {base_layer}, layer count {layer_count}) was out of range with image mip levels {image_mip_levels} and array layers {image_array_layers}")]
    SubresourceOutOfRange {
        image_mip_levels: u32, base_level: u32, level_count: u32,
        image_array_layers: u32, base_layer: u32, layer_count: u32,
    },
    #[display("image has immutable format {image_format:?}, requested format was {requested_format:?}")]
    ImmutableFormat {
        image_format: vk::Format,
        requested_format: vk::Format,
    },
    #[display("image usage mismatch, missing usage was {missing_usage:?}")]
    UsageMismatch {
        missing_usage: vk::ImageUsageFlags,
    },
    #[display("invalid image copy (copy offset {copy_offset}, copy dimensions {copy_dimensions}) with image dimensions {image_dimensions}")]
    InvalidCopy {
        image_dimensions: Dimensions,
        copy_offset: Offset3D,
        copy_dimensions: Dimensions,
    },
    #[display("invalid cube map with layer count {layer_count}")]
    InvalidCubeMap {
        layer_count: u32,
    },
    #[display("slot map error")]
    SlotMapError(#[from] #[source] SlotMapError),
}
