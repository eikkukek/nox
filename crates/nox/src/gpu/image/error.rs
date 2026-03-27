use core::error;

use nox_mem::Display;

use super::*;

#[derive(Display, Debug)]
#[display("subresource (base level {base_level}, level count {level_count}, base layer {base_layer}, layer count {layer_count}) was out of range with image mip levels {image_mip_levels} and array layers {image_array_layers}")]
pub struct ImageSubresourceOutOfRangeError {
    pub image_mip_levels: u32, 
    pub base_level: u32,
    pub level_count: u32,
    pub image_array_layers: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

impl error::Error for ImageSubresourceOutOfRangeError {}

impl Flags for ImageUsages {

    const NAME: &str = "image usage";
}

impl Flags for ImageAspects {

    const NAME: &str = "image aspects";
}
