use core::{
    num::NonZeroU32,
    hash::Hash,
    fmt::{self, Display, Formatter},
};

use nox_ash::vk;

use super::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Dimensions {

    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.width == 0 ||
        self.height == 0 ||
        self.depth == 0
    }

    pub fn texel_count(&self) -> vk::DeviceSize {
        self.width as vk::DeviceSize *
        self.height as vk::DeviceSize *
        self.depth as vk::DeviceSize
    }
}

impl From<Dimensions> for vk::Extent3D {

    fn from(value: Dimensions) -> Self {
        vk::Extent3D {
            width: value.width,
            height: value.height,
            depth: value.depth,
        }
    }
}

impl From<vk::Extent2D> for Dimensions {

    fn from(value: vk::Extent2D) -> Self {
        Dimensions::new(
            value.width,
            value.height,
            1
        )
    }
}

impl Display for Dimensions {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(width: {}, height: {}, depth: {})", self.width, self.height, self.depth)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

impl From<ComponentMapping> for vk::ComponentMapping {

    fn from(value: ComponentMapping) -> Self {
        Self {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: value.a.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceRange {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: NonZeroU32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceRange {

    pub fn new(
        aspect_mask: u32,
        base_mip_level: u32,
        level_count: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self>
    {
        Some(Self {
            aspect_mask,
            base_mip_level,
            level_count: NonZeroU32::new(level_count)?,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }

    pub fn overlaps(self, other: Self) -> bool {
        if self.aspect_mask & other.aspect_mask != 0 {
            self.base_mip_level < other.base_mip_level + other.level_count.get() &&
            other.base_mip_level < self.base_mip_level + self.level_count.get() &&
            self.base_array_layer < other.base_array_layer + other.layer_count.get() &&
            other.base_array_layer < self.base_array_layer + self.layer_count.get()
        } else {
            false
        }
    }
}

impl Default for ImageSubresourceRange {

    fn default() -> Self {
        Self {
            aspect_mask: 0,
            base_mip_level: 0,
            level_count: NonZeroU32::new(1).unwrap(),
            base_array_layer: 0,
            layer_count: NonZeroU32::new(1).unwrap(),
        }
    }
}

impl From<ImageSubresourceRange> for vk::ImageSubresourceRange {

    fn from(value: ImageSubresourceRange) -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::from_raw(value.aspect_mask),
            base_mip_level: value.base_mip_level,
            level_count: value.level_count.get(),
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSubresourceLayers {
    pub aspect_mask: u32,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: NonZeroU32,
}

impl ImageSubresourceLayers {

    pub fn new<A: Into<u32>>(
        aspect_mask: A,
        mip_level: u32,
        base_array_layer: u32,
        layer_count: u32,
    ) -> Option<Self> {
        Some(Self {
            aspect_mask: aspect_mask.into(),
            mip_level,
            base_array_layer,
            layer_count: NonZeroU32::new(layer_count)?,
        })
    }
}

impl From<ImageSubresourceLayers> for vk::ImageSubresourceLayers {

    fn from(value: ImageSubresourceLayers) -> Self {
        Self {
            aspect_mask: vk::ImageAspectFlags::from_raw(value.aspect_mask),
            mip_level: value.mip_level,
            base_array_layer: value.base_array_layer,
            layer_count: value.layer_count.get(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentInfo {
    pub component_mapping: ComponentMapping,
    pub(crate) format: vk::Format,
}

impl ComponentInfo {

    pub fn new<F: Format>(
        component_mapping: ComponentMapping,
        format: F,
    ) -> Self
    {
        Self {
            component_mapping,
            format: format.as_vk_format(),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ImageRange {
    pub subresource: ImageSubresourceRange,
    pub component_info: Option<ComponentInfo>,
    pub cube_map: bool,
}

impl ImageRange {

    #[inline(always)]
    pub fn new(
        subresource: ImageSubresourceRange,
        component_info: Option<ComponentInfo>,
    ) -> Self
    {
        Self {
            subresource,
            component_info,
            cube_map: false,
        }
    }

    pub fn with_cube_map(mut self, is: bool) -> Self {
        self.cube_map = is;
        self
    }
}
