mod format;
mod enums;
mod structs;
mod state;
mod attributes;
mod properties;
mod sampler;
mod error;

use crate::sync::Arc;

use core::ptr::NonNull;

use core::hash::{Hash, Hasher};

use nox_ash::vk;

use compact_str::{format_compact, CompactString};

use ahash::AHashMap;

use nox_error::Context;
use nox_mem::{
    collections::EntryExt,
    vec::{NonNullVec32, Vector},
    alloc::{LocalAlloc, StdAlloc, Layout},
    num::Integer,
};

use nox_alloc::arena::{self, CellArena};

use crate::gpu::prelude::{memory_binder::{DeviceMemory, MemoryBinder}, *};

use crate::dev::{has_bits, has_not_bits};

use crate::dev::error as dev_error;

use super::{MSAA, Vulkan};

pub use format::*;
pub use enums::*;
pub use structs::*;
pub use error::*;
pub use attributes::*;
pub use sampler::*;
pub(crate) use state::*;
pub(crate) use properties::ImageProperties;

pub struct Image;

impl Image {

    #[inline(always)]
    pub fn default_attributs() -> ImageAttributes {
        ImageAttributes::new()
    }
}

pub(crate) struct ImageMeta {
    vk: Arc<Vulkan>,
    handle: vk::Image,
    view: vk::ImageView,
    subviews: AHashMap<ImageRange, vk::ImageView>,
    properties: ImageProperties,
    states: NonNullVec32<'static, NonNullVec32<'static, ImageMipRange>>,
    memory: Box<dyn DeviceMemory>,
    last_used_frame: u64,
    arena_size: usize,
}

unsafe impl Send for ImageMeta {}
unsafe impl Sync for ImageMeta {}

impl ImageMeta {

    pub(crate) fn new(
        vk: Arc<Vulkan>,
        attributes: ImageAttributes,
        alloc: &mut (impl MemoryBinder + ?Sized),
    ) -> Result<Self, dev_error::Error>
    {
        let mut image_type = vk::ImageType::TYPE_2D;
        if attributes.dimensions.depth > 1 {
            if attributes.array_layers != 1 {
                return Err(dev_error::Error::just_context(CompactString::new(
                    "image layers must be 1 if depth is greater than 1"
                )))
            }
            image_type = vk::ImageType::TYPE_3D;
        }
        if attributes.format == vk::Format::UNDEFINED {
            return Err(dev_error::Error::just_context(CompactString::new(
                "image format must be defined"
            )))
        }
        let mut flags = Default::default();
        if attributes.mutable_format {
            flags |= vk::ImageCreateFlags::MUTABLE_FORMAT;
        }
        if attributes.cube_map {
            flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE;
            if attributes.dimensions.width != attributes.dimensions.height ||
                attributes.dimensions.depth != 1
            {
                return Err(dev_error::Error::just_context(format_compact!(
                    "width ({}) and height ({}) of a cube map compatible image must be equal and depth ({}) must be 1",
                    attributes.dimensions.width, attributes.dimensions.height, attributes.dimensions.depth,
                )))
            }
        }
        if attributes.dimensions.is_zero() {
            return Err(dev_error::Error::just_context(format_compact!(
                "image dimensions {} must not be zero",
                attributes.dimensions,
            )))
        }
        if attributes.array_layers == 0 {
            return Err(dev_error::Error::just_context(format_compact!(
                "image layers must be greater than 0",
            )))
        }
        if attributes.cube_map && attributes.array_layers < 6 {
            return Err(dev_error::Error::just_context(format_compact!(
                "layer count {} of a cube map/array image must be at least 6",
                attributes.array_layers,
            )))
        }
        if attributes.mip_levels == 0 {
            return Err(dev_error::Error::just_context(format_compact!(
                "mip levels must be greater than zero",
            )))
        }
        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            flags,
            image_type,
            format: attributes.format,
            extent: attributes.dimensions.into(),
            mip_levels: attributes.mip_levels,
            array_layers: attributes.array_layers,
            samples: attributes.samples.into(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: attributes.usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            ..Default::default()
        };
        let mut mem_requirements = Default::default();
        unsafe {
            let mut device_mem_requirements = vk::DeviceImageMemoryRequirements {
                s_type: vk::StructureType::DEVICE_IMAGE_MEMORY_REQUIREMENTS,
                p_create_info: &create_info,
                ..Default::default()
            };
            vk.maintenance4_device().get_device_image_memory_requirements(
                &device_mem_requirements, &mut mem_requirements,
            );
        };
        let memory = unsafe {
            alloc.alloc(&mem_requirements)
            .context("failed to allocate GPU memory for image")?
        };
        let handle = unsafe {
            vk.device().create_image(&create_info, None)
            .context("failed to create Vulkan image")?
        };
        unsafe {
            vk.device().bind_image_memory(
                handle, memory.device_memory(),
                memory.offset()
            ).context("failed to bind image memory")?
        };
        let properties = ImageProperties {
            dimensions: attributes.dimensions,
            format: attributes.format,
            aspect_mask: attributes.aspects.as_raw(),
            usage: attributes.usage,
            samples: attributes.samples,
            array_layers: attributes.array_layers,
            mip_levels: attributes.mip_levels,
            create_flags: flags,
            format_resolve_modes: attributes.resolve_modes,
        };
        let view_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image: handle,
            view_type: properties.view_type(), 
            format: properties.format,
            components: ComponentMapping::default().into(),
            subresource_range: properties.whole_subresource().into(),
            ..Default::default()
        };
        let view = unsafe {
            vk.device().create_image_view(&view_create_info, None)
            .context("failed to create main image view")?
        };
        let mip_levels = properties.mip_levels;
        let num_layers = properties.aspect_mask.count_ones() * properties.array_layers;
        let arena = CellArena::new(
            (num_layers as usize * size_of::<NonNullVec32<ImageMipRange>>()).next_multiple_of(8) +
            (num_layers * mip_levels) as usize * size_of::<ImageMipRange>()
        ).expect("global alloc failed");
        let mut states = NonNullVec32
            ::with_capacity(num_layers, &arena)
            .unwrap()
            .into_static();
        states.resize_with(num_layers, || {
            let mut vec = NonNullVec32::with_capacity(mip_levels, &arena).unwrap();
            vec.push(ImageMipRange {
                state: ImageSubresourceState {
                    stage_mask: vk::PipelineStageFlags2::NONE,
                    access_mask: vk::AccessFlags2::NONE,
                    layout: vk::ImageLayout::UNDEFINED,
                    queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    command_index: COMMAND_REQUEST_IGNORED,
                    command_timeline_value: 0,
                },
                base_mip_level: 0,
                level_count: mip_levels,
            });
            vec.into_static()
        });
        Ok(Self {
            vk,
            handle,
            view,
            subviews: AHashMap::default(),
            properties,
            states,
            memory,
            last_used_frame: 0,
            arena_size: arena.into_inner().size(),
        })
    }

    fn get_states_mut(
        &mut self,
        aspect: vk::ImageAspectFlags,
        layer: u32,
    ) -> Option<&mut NonNullVec32<'static, ImageMipRange>>
    {
        let aspect_mask = self.properties.aspect_mask;
        let aspect = aspect.as_raw();
        if aspect_mask & aspect != aspect {
            return None
        }
        let mut cindex = 0;
        let mut index = 0;
        macro_rules! index_op {
            ($($n:expr),* $(,)?) => {
                $(
                    cindex += ((1 << $n) & aspect_mask) >> $n;
                    index |= (((1 << $n) & aspect) >> $n).wrapping_neg() & cindex;
                )*
            };
        }
        index_op!(
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            /* 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, <- not needed */
        );
        index -= 1;
        let index = index * self.properties.array_layers + layer;
        Some(&mut self.states[index as usize])
    }

    #[inline(always)]
    pub(crate) fn handle(&self) -> vk::Image {
        self.handle
    }

    #[inline(always)]
    pub(crate) fn properties(&self) -> ImageProperties {
        self.properties.clone()
    }

    pub(crate) fn validate_usage(
        &self,
        usage: vk::ImageUsageFlags,
    ) -> Option<ImageError> {
        let has = self.properties.usage;
        (has_not_bits!(has, usage))
            .then_some(ImageError::UsageMismatch {
                missing_usage: usage ^ has & usage,
        })
    }

    #[inline(always)]
    pub(crate) fn validate_layers(
        &self,
        layers: ImageSubresourceLayers
    ) -> Option<ImageError>
    {
        if has_not_bits!(self.properties.aspect_mask, layers.aspect_mask) {
            return Some(ImageError::AspectMismatch(vk::ImageAspectFlags::from_raw(
                layers.aspect_mask ^ self.properties.aspect_mask & layers.aspect_mask
            )))
        }
        if layers.mip_level > self.properties.mip_levels ||
            layers.base_array_layer + layers.layer_count.get() > self.properties.array_layers
        {
            return Some(ImageError::SubresourceOutOfRange {
                image_mip_levels: self.properties.mip_levels,
                base_level: layers.mip_level,
                level_count: 1,
                image_array_layers: self.properties.array_layers,
                base_layer: layers.base_array_layer,
                layer_count: layers.layer_count.get(),
            })
        }
        None
    }

    #[inline(always)]
    pub(crate) fn get_view(&self) -> vk::ImageView {
        self.view
    }

    #[inline(always)]
    pub(crate) fn get_subview(
        &mut self,
        range: ImageRange,
    ) -> Result<vk::ImageView, ImageError>
    {
        Ok(*self.subviews.entry(range).or_try_insert_with::<_, ImageError>(|| {
            let component_info = 
                if let Some(info) = range.component_info {
                    info
                } else {
                    ComponentInfo {
                        component_mapping: ComponentMapping::default(),
                        format: self.properties.format,
                    }
                };
            let device = self.vk.device();
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: self.handle,
                view_type: self.properties.validate_range(range)?,
                format: component_info.format,
                components: component_info.component_mapping.into(),
                subresource_range: range.subresource.into(),
                ..Default::default()
            };
            let view = unsafe {
                device.create_image_view(&create_info, None)?
            };
            Ok(view)
        })?)
    }

    #[inline(always)]
    pub fn flush_subresources(&mut self) {
        for states in &mut self.states {
            for range in states {
                range.state.stage_mask = vk::PipelineStageFlags2::ALL_COMMANDS;
                range.state.access_mask = vk::AccessFlags2::MEMORY_WRITE;
            }
        }
    }

    #[inline(always)]
    pub(crate) fn memory_barrier<'a>(
        &mut self,
        state: ImageSubresourceState,
        subresource_info: Option<ImageSubresourceRange>,
        preserve_contents: bool,
        cache: &'a mut ImageMemoryBarrierCache,
    ) -> Result<&'a [ImageMemoryBarrier], ImageError>
    {
        cache.barriers.clear();
        let subresource =
            if let Some(info) = subresource_info {
                self.properties.validate_range(ImageRange::new(info, None))?;
                info
            } else {
                self.properties.whole_subresource()
            };
        let mut mip_range = ImageMipRange {
            state,
            base_mip_level: subresource.base_mip_level,
            level_count: subresource.level_count.get(),
        };
        let layer_start = subresource.base_array_layer;
        let layer_end = subresource.base_array_layer + subresource.layer_count.get();
        for aspect in subresource.aspect_mask
            .bit_iter()
            .map(|aspect| vk::ImageAspectFlags::from_raw(aspect))
        {
            for layer in layer_start..layer_end {
                let mut not_inserted = None;
                let ranges = self.get_states_mut(aspect, layer).unwrap();
                for i in (0..ranges.len()).rev() {
                    match unsafe { ranges.get_unchecked(i as usize).overwrite(&mip_range) } {
                        StateOverwrite::NoOverlap => continue,
                        StateOverwrite::Combine(new_range) => {
                            ranges.remove(i);
                            mip_range = new_range;
                            not_inserted = Some(i);
                        },
                        StateOverwrite::Consume(mut barrier) => {
                            ranges.remove(i);
                            barrier.subresource_range.aspect_mask = aspect;
                            barrier.subresource_range.base_array_layer = layer;
                            if !preserve_contents {
                                barrier.old_layout = vk::ImageLayout::UNDEFINED;
                            }
                            cache.insert(aspect, barrier);
                            not_inserted = Some(i);
                        },
                        StateOverwrite::Cut(left, right, mut barrier) => {
                            ranges.remove(i);
                            barrier.subresource_range.aspect_mask = aspect;
                            barrier.subresource_range.base_array_layer = layer;
                            if !preserve_contents {
                                barrier.old_layout = vk::ImageLayout::UNDEFINED;
                            }
                            cache.insert(aspect, barrier);
                            if left.level_count != 0 {
                                ranges.insert(i, left);
                            }
                            ranges.insert(i + 1, mip_range);
                            if right.level_count != 0 {
                                ranges.insert(i + 2, right);
                            }
                            not_inserted = None;
                            break
                        },
                        StateOverwrite::Shrink(new_range, mut barrier) => {
                            ranges[i as usize] = new_range;
                            barrier.subresource_range.aspect_mask = aspect;
                            barrier.subresource_range.base_array_layer = layer;
                            if !preserve_contents {
                                barrier.old_layout = vk::ImageLayout::UNDEFINED;
                            }
                            cache.insert(aspect, barrier);
                            if new_range.base_mip_level < mip_range.base_mip_level {
                                ranges.insert(i + 1, mip_range);
                                not_inserted = None;
                                break
                            }
                        },
                    }
                }
                if let Some(i) = not_inserted {
                    ranges.insert(i, mip_range);
                }
            }
        }
        for key in &cache.touched {
            let barriers = cache.cache.get_mut(key).unwrap();
            for i in (0..barriers.len() as usize - 1).rev() {
                let next_idx = i + 1;
                let mut next = barriers[next_idx];
                let this = &mut barriers[i];
                if this.src_stage_mask == next.src_stage_mask &&
                    this.src_access_mask == next.src_access_mask &&
                    this.old_layout == next.old_layout &&
                    this.src_command_index == next.src_command_index &&
                    next.subresource_range.base_array_layer ==
                    this.subresource_range.base_array_layer + 1
                {
                    next.subresource_range.base_array_layer -= 1;
                    next.subresource_range.layer_count += 1;
                    next.src_command_timeline_value = next.src_command_timeline_value.max(
                        this.src_command_timeline_value
                    );
                    *this = next;
                    barriers.remove(next_idx as u32);
                }
            }
            cache.barriers.append(barriers);
            barriers.clear();
        }
        cache.touched.clear();
        Ok(&cache.barriers)
    }

    #[inline(always)]
    pub(crate) fn get_last_used_frame(&self) -> u64 {
        self.last_used_frame
    }

    #[inline(always)]
    pub(crate) unsafe fn set_last_used_frame(&mut self, frame: u64) {
        self.last_used_frame = frame;
    }
}

impl Drop for ImageMeta {

    fn drop(&mut self) {
        let device = self.vk.device();
        unsafe {
            for &subview in self.subviews.values() {
                device.destroy_image_view(subview, None);
            }
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.handle(), None);
            StdAlloc.free_raw(
                NonNull::new_unchecked(self.states.as_mut_ptr()).cast(),
                Layout::from_size_align_unchecked(
                    self.arena_size,
                    arena::max_align()
                ),
            );
        }
    }
}
