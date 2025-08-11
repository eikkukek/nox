use super::*;

#[derive(Clone, Copy, Debug)]
pub enum ImageError {
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
        copy_offset: Offset,
        copy_dimensions: Dimensions,
    },
    InvalidCubeMap {
        layer_count: u32,
    },
}
