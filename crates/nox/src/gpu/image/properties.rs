use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageProperties {
    pub(crate) dimensions: Dimensions,
    pub(crate) aspect_mask: u32,
    pub(crate) format: vk::Format,
    pub(crate) usage: vk::ImageUsageFlags,
    pub(crate) samples: MSAA,
    pub(crate) array_layers: u32,
    pub(crate) mip_levels: u32,
    pub(crate) create_flags: vk::ImageCreateFlags,
    pub(crate) always_cube_map: bool,
}

impl ImageProperties {

    #[inline(always)]
    pub(crate) fn view_type(&self) -> vk::ImageViewType {
        if self.dimensions.depth > 1 {
            vk::ImageViewType::TYPE_3D
        } else if self.always_cube_map {
            if self.array_layers > 6 {
                vk::ImageViewType::CUBE_ARRAY
            } else {
                vk::ImageViewType::CUBE
            }
        } else if self.array_layers > 1 {
            vk::ImageViewType::TYPE_2D_ARRAY
        } else {
            vk::ImageViewType::TYPE_2D
        }
    }

    #[inline(always)]
    pub(crate) fn has_mutable_format(&self) -> bool {
        has_bits!(self.create_flags, vk::ImageCreateFlags::MUTABLE_FORMAT)
    }

    #[inline(always)]
    pub(crate) fn whole_subresource(&self) -> ImageSubresourceRangeInfo {
        ImageSubresourceRangeInfo::new(
            self.aspect_mask,
            0, self.mip_levels,
            0, self.array_layers
        ).unwrap()
    }

    #[inline(always)]
    pub(crate) fn all_layers(&self, mip_level: u32) -> ImageSubresourceLayers {
        ImageSubresourceLayers::new(
            self.aspect_mask,
            mip_level,
            0,
            self.array_layers,
        ).unwrap()
    }
}
