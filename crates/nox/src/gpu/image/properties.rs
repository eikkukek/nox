use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ImageProperties {
    pub dimensions: Dimensions,
    pub aspect_mask: ImageAspect,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub samples: MSAA,
    pub array_layers: u32,
    pub mip_levels: u32,
    pub create_flags: vk::ImageCreateFlags,
    pub format_resolve_modes: FormatResolveModes,
}

impl ImageProperties {

    #[inline(always)]
    pub fn view_type(&self) -> vk::ImageViewType {
        if self.dimensions.depth > 1 {
            vk::ImageViewType::TYPE_3D
        } else if self.array_layers > 1 {
            vk::ImageViewType::TYPE_2D_ARRAY
        } else {
            vk::ImageViewType::TYPE_2D
        }
    }

    pub fn validate_range(&self, range: &ImageRange) -> Result<vk::ImageViewType, ImageError> {
        if let Some(component_info) = range.component_info &&
            !self.has_mutable_format() && self.format != component_info.format
        {
            return Err(ImageError::ImmutableFormat {
                image_format: self.format,
                requested_format: component_info.format,
            })
        }
        let subresource_info = range.subresource_range;
        if has_not_bits!(self.aspect_mask, subresource_info.aspect_mask) {
            return Err(ImageError::AspectMismatch(
                subresource_info.aspect_mask ^ self.aspect_mask & subresource_info.aspect_mask
            ))
        }
        if subresource_info.base_mip_level + subresource_info.level_count.get() > self.mip_levels ||
            subresource_info.base_array_layer + subresource_info.layer_count.get() > self.array_layers
        {
            return Err(ImageError::SubresourceOutOfRange {
                image_mip_levels: self.mip_levels,
                base_level: subresource_info.base_mip_level,
                level_count: subresource_info.level_count.get(),
                image_array_layers: self.array_layers,
                base_layer: subresource_info.base_array_layer,
                layer_count: subresource_info.layer_count.get(),
            })
        }
        if range.cube_map {
            if !self.create_flags.contains(vk::ImageCreateFlags::CUBE_COMPATIBLE) {
                Err(ImageError::ValidationError(format_compact!(
                    "image is not cube compatible"
                )))
            } else if !range.subresource_range.layer_count.get().is_multiple_of(6) {
                Err(ImageError::ValidationError(format_compact!(
                    "cube subview layer count {} must be multiple of 6",
                    range.subresource_range.layer_count,
                )))
            } else if range.subresource_range.layer_count.get() > 6 {
                Ok(vk::ImageViewType::CUBE_ARRAY)
            } else {
                Ok(vk::ImageViewType::CUBE)
            }
        } else {
            Ok(if self.dimensions.depth > 1 {
                vk::ImageViewType::TYPE_3D
            } else if range.subresource_range.layer_count.get() > 1 {
                vk::ImageViewType::TYPE_2D_ARRAY
            } else {
                vk::ImageViewType::TYPE_2D
            })
        }
    }

    #[inline(always)]
    pub fn has_mutable_format(&self) -> bool {
        has_bits!(self.create_flags, vk::ImageCreateFlags::MUTABLE_FORMAT)
    }

    #[inline(always)]
    pub fn whole_subresource(&self) -> ImageSubresourceRange {
        ImageSubresourceRange::new(
            self.aspect_mask,
            0, self.mip_levels,
            0, self.array_layers
        ).unwrap()
    }

    #[inline(always)]
    pub fn all_layers(&self, mip_level: u32) -> ImageSubresourceLayers {
        ImageSubresourceLayers::new(
            self.aspect_mask,
            mip_level,
            0,
            self.array_layers,
        ).unwrap()
    }
}
