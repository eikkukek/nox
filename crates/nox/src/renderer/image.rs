mod enums;
mod structs;

use core::{
    num::{NonZeroU64},
    hash::{Hash, Hasher},
};

use ash::vk;

use nox_mem::{slice, AsRaw};
use crate::{byte_hash::{self, ByteHash}, has_bits, has_not_bits};

use super::{Renderer, Error, MSAA, Handle};

pub use enums::*;
pub use structs::*;

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
}

fn make_aspect_mask(aspects: &[ImageAspect]) -> u32 {
    let mut mask = 0;
    for aspect in aspects {
        mask |= *aspect;
    }
    mask
}

#[derive(Clone)]
pub struct Image {
    handle: NonZeroU64,
    view: Option<NonZeroU64>,
    device: *const ash::Device,
    aspect_mask: u32,
    state: ImageState,
    dimensions: Dimensions,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    samples: MSAA,
    array_layers: u32,
    mip_levels: u32,
    create_flags: vk::ImageCreateFlags,
}

impl Image {

    #[inline(always)]
    pub(crate) fn handle(&self) -> vk::Image {
        vk::Handle::from_raw(self.handle.get())
    }

    #[inline(always)]
    pub(crate) fn device(&self) -> &ash::Device {
        unsafe {
            &*self.device
        }
    }

    #[inline(always)]
    pub(crate) fn view_type(&self) -> vk::ImageViewType {
        if self.dimensions.depth > 1 {
            vk::ImageViewType::TYPE_3D
        }
        else {
            vk::ImageViewType::TYPE_2D
        }
    }

    #[inline(always)]
    pub fn has_mutable_format(&self) -> bool {
        has_bits!(self.create_flags, vk::ImageCreateFlags::MUTABLE_FORMAT)
    }

    pub(crate) fn get_view(
        &mut self,
        component_mapping: ComponentMapping,
    ) -> Result<Handle<vk::ImageView>, Error>
    {
        if self.view.is_none() {
            let device = self.device();
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: self.handle(),
                view_type: self.view_type(),
                format: self.format,
                components: component_mapping.into(),
                subresource_range: ImageSubresourceRangeInfo::new(
                    self.aspect_mask,
                    0, self.mip_levels,
                    0, self.array_layers
                ).unwrap().into(),
                ..Default::default()
            };

            self.view = NonZeroU64::new(unsafe {
                vk::Handle::as_raw(device.create_image_view(&create_info, None)?)
            });
        }

        Ok(Handle::new(
            vk::Handle::from_raw(self.view.unwrap().get())
        ))
    }

    #[inline(always)]
    pub(crate) fn cmd_memory_barrier(
        &mut self,
        state: ImageState,
        command_buffer: vk::CommandBuffer
    )
    {
        if self.state == state {
            return
        }
        let memory_barrier = self.state.to_memory_barrier(
            self.handle(),
            self.state,
            ImageSubresourceRangeInfo::new(
                self.aspect_mask,
                0, self.mip_levels,
                0, self.array_layers
            ).unwrap().into(),
        );
        unsafe {
            self.device().cmd_pipeline_barrier(
                command_buffer,
                self.state.pipeline_stage,
                state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                slice![memory_barrier]);
        }
    }

    #[inline(always)]
    pub(crate) fn subresource_range(
        &self,
        format: vk::Format,
        info: ImageSubresourceRangeInfo,
        command_buffer: vk::CommandBuffer,
    ) -> Result<ImageSubresourceRange<'_>, ImageError>
    {
        if !self.has_mutable_format() && self.format != format {
            return Err(ImageError::ImmutableFormat {
                image_format: self.format,
                requested_format: format,
            })
        }
        if has_not_bits!(self.aspect_mask, info.aspect_mask) {
            return Err(ImageError::AspectMismatch)
        }
        if info.base_mip_level + info.level_count.get() > self.mip_levels ||
            info.base_array_layer + info.layer_count.get() > self.array_layers {
            return Err(ImageError::SubresourceOutOfRange {
                image_mip_levels: self.mip_levels, base_level: info.base_mip_level, level_count: info.level_count.get(),
                image_array_layers: self.array_layers, base_layer: info.base_array_layer, layer_count: info.layer_count.get(),
            })
        }
        let state = self.state;
        Ok(ImageSubresourceRange {
            image: self,
            view: None,
            command_buffer,
            state,
            info,
            format,
        })
    }
}

impl Drop for Image {

    fn drop(&mut self) {
        unsafe {
            let device = self.device();
            if let Some(view) = self.view {
                device.destroy_image_view(vk::Handle::from_raw(view.get()), None);
            }
            device.destroy_image(self.handle(), None);
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImageBuilder<'a> {
    renderer: &'a Renderer<'a>,
    dimensions: Dimensions,
    pub(crate) format: vk::Format,
    aspects: &'static [ImageAspect],
    usage: vk::ImageUsageFlags,
    samples: MSAA,
    array_layers: u32,
    mip_levels: u32,
    mutable_format: bool,
}

impl<'a> ImageBuilder<'a> {

    #[inline(always)]
    pub(crate) fn new(renderer: &'a Renderer<'a>) -> Self {
        Self {
            renderer,
            format: vk::Format::UNDEFINED,
            aspects: slice![],
            usage: Default::default(),
            samples: MSAA::X1,
            dimensions: Dimensions::new(0, 0, 0),
            array_layers: 1,
            mip_levels: 1,
            mutable_format: false,
        }
    }

    #[inline(always)]
    pub fn with_dimensions(&mut self, dimensions: Dimensions) -> &mut Self {
        assert!(!dimensions.zero(), "each image dimension must be greater than 0");
        self.dimensions = dimensions;
        self
    }

    #[inline(always)]
    pub fn with_format<F: Format>(&mut self, format: F, mutable: bool) -> &mut Self {
        self.format = format.as_vk_format();
        self.aspects = format.aspects();
        self.mutable_format = mutable;
        self
    }

    #[inline(always)]
    pub fn with_usage(&mut self, usage: ImageUsage) -> &mut Self {
        self.usage |= usage.into();
        self
    }

    #[inline(always)]
    pub fn with_samples(&mut self, samples: MSAA) -> &mut Self {
        self.samples = samples;
        self
    }

    #[inline(always)]
    pub fn with_array_layers(&mut self, layers: u32) -> &mut Self {
        assert!(layers > 0, "image layers must be greater than 0");
        self.array_layers = layers;
        self
    }

    #[inline(always)]
    pub fn with_mip_levels(&mut self, levels: u32) -> &mut Self {
        assert!(levels > 0, "image mip levels must be greater than 0");
        self.mip_levels = levels;
        self
    }

    pub(crate) fn build(self) -> Result<Image, Error> {
        let mut image_type = vk::ImageType::TYPE_2D;
        if self.dimensions.depth > 1 {
            assert!(self.array_layers == 1, "image layers must be 1 if depth is greater than 1");
            image_type = vk::ImageType::TYPE_3D;
        }
        let mut flags = Default::default();
        if self.mutable_format {
            flags |= vk::ImageCreateFlags::MUTABLE_FORMAT;
        }
        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            flags,
            image_type,
            format: self.format,
            extent: self.dimensions.into(),
            mip_levels: self.mip_levels,
            array_layers: self.array_layers,
            samples: self.samples.into(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: self.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        };
        let device = self.renderer.device();
        let handle = unsafe {
            device.create_image(&create_info, None)?
        };
        Ok(Image {
            handle: NonZeroU64::new(vk::Handle::as_raw(handle)).unwrap(),
            view: None,
            device,
            state: ImageState::new(
                vk::AccessFlags::NONE,
                vk::ImageLayout::UNDEFINED,
                vk::QUEUE_FAMILY_IGNORED,
                vk::PipelineStageFlags::NONE,
            ),
            format: self.format,
            aspect_mask: make_aspect_mask(self.aspects),
            usage: self.usage,
            samples: self.samples,
            dimensions: self.dimensions,
            array_layers: self.array_layers,
            mip_levels: self.mip_levels,
            create_flags: flags,
        })
    }
}

impl<'a> PartialEq for ImageBuilder<'a> {

    fn eq(&self, other: &Self) -> bool {
        self.dimensions == other.dimensions &&
        self.format == other.format &&
        make_aspect_mask(self.aspects) == make_aspect_mask(other.aspects) &&
        self.usage == other.usage &&
        self.samples == other.samples &&
        self.array_layers == other.array_layers &&
        self.mip_levels == other.mip_levels &&
        self.mutable_format == other.mutable_format
    }
}

impl<'a> Eq for ImageBuilder<'a> {}

impl<'a> Hash for ImageBuilder<'a> {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dimensions.hash(state);
        self.format.as_raw().hash(state);
        make_aspect_mask(self.aspects).hash(state);
        self.usage.as_raw().hash(state);
        self.samples.as_raw().hash(state);
        self.array_layers.hash(state);
        self.mip_levels.hash(state);
        self.mutable_format.hash(state);
    }
}

impl<'a> ByteHash for ImageBuilder<'a> {

    fn byte_hash(&self, hasher: &mut blake3::Hasher) {
        self.dimensions.byte_hash(hasher);
        self.format.as_raw().byte_hash(hasher);
        make_aspect_mask(self.aspects).byte_hash(hasher);
        self.usage.as_raw().byte_hash(hasher);
        self.samples.as_raw().byte_hash(hasher);
        self.array_layers.byte_hash(hasher);
        self.mip_levels.byte_hash(hasher);
        self.mutable_format.byte_hash(hasher);
    }
}

pub(crate) struct ImageSubresourceRange<'a> {
    image: &'a Image,
    view: Option<NonZeroU64>,
    command_buffer: vk::CommandBuffer,
    state: ImageState,
    info: ImageSubresourceRangeInfo,
    format: vk::Format,
}

impl<'a> ImageSubresourceRange<'a> {

    pub(crate) fn get_view(
        &mut self,
        component_mapping: ComponentMapping,
    ) -> Result<Handle<vk::ImageView>, Error>
    {
        if self.view.is_none() {
            let device = self.image.device();
            let info = self.info;
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: self.image.handle(),
                view_type: self.image.view_type(),
                format: self.format,
                components: component_mapping.into(),
                subresource_range: info.into(),
                ..Default::default()
            };

            self.view = NonZeroU64::new(unsafe {
                vk::Handle::as_raw(device.create_image_view(&create_info, None)?)
            });
        }

        Ok(Handle::new(
            vk::Handle::from_raw(self.view.unwrap().get())
        ))
    }

    pub(crate) fn cmd_memory_barrier(
        &mut self,
        state: ImageState,
    )
    {
        if self.state == state {
            return
        }
        let device = self.image.device();
        let memory_barrier = self.state.to_memory_barrier(
            self.image.handle(),
            state,
            self.info.into(),
        );
        unsafe {
            device.cmd_pipeline_barrier(self.command_buffer,
                self.state.pipeline_stage,
                state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                slice![memory_barrier],
            );
        }
        self.state = state;
    }
}

impl<'a> Drop for ImageSubresourceRange<'a> {

    fn drop(&mut self) {

        if self.state != self.image.state {
            let device = self.image.device();
            let image_state = self.image.state;
            let memory_barrier = self.state.to_memory_barrier(
                self.image.handle(),
                image_state,
                self.info.into(), 
            );
            unsafe {
                device.cmd_pipeline_barrier(self.command_buffer,
                    self.state.pipeline_stage,
                    image_state.pipeline_stage,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    slice![memory_barrier],
                );
            }
        }
    }
}
