mod enums;
mod structs;
mod builder;
mod properties;
mod image;
mod sampler;
mod error;

use std::sync::Arc;

use core::{
    num::{NonZeroU64},
    hash::{Hash, Hasher},
};

use ash::vk;

use crate::{has_bits, has_not_bits};

use super::MSAA;

pub use enums::*;
pub use structs::*;
pub use error::*;
pub use builder::*;
pub use sampler::*;
pub use properties::ImageProperties;
pub(crate) use image::*;

pub(crate) fn make_aspect_mask(aspects: &[ImageAspect]) -> u32 {
    let mut mask = 0;
    for aspect in aspects {
        mask |= *aspect;
    }
    mask
}
