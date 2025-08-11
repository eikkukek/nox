use std::sync::{Arc, RwLock};

use nox_mem::slot_map::{GlobalSlotMap, SlotIndex};

use crate::renderer::{memory_binder::DeviceMemory};

use super::*;

pub(crate) struct Image {
    pub handle: vk::Image,
    pub memory: Option<Box<dyn DeviceMemory>>,
    pub view: RwLock<Option<NonZeroU64>>,
    pub device: Arc<ash::Device>,
    pub subviews: RwLock<GlobalSlotMap<vk::ImageView>>,
    pub state: RwLock<ImageState>,
    pub properties: ImageProperties,
    pub component_mapping: ComponentMapping,
}

impl Image {

    #[inline(always)]
    pub(crate) fn handle(&self) -> vk::Image {
        self.handle
    }

    #[inline(always)]
    pub(crate) fn view_type(&self) -> vk::ImageViewType {
        self.properties.view_type()
    }

    #[inline(always)]
    pub(crate) fn state(&self) -> ImageState {
        *self.state.read().unwrap()
    }

    #[inline(always)]
    pub fn has_mutable_format(&self) -> bool {
        self.properties.has_mutable_format()
    }

    #[inline(always)]
    pub(crate) fn layout(&self) -> vk::ImageLayout {
        self.state.read().unwrap().layout
    }

    #[inline(always)]
    pub(crate) fn component_info(&self) -> ComponentInfo {
        ComponentInfo {
            component_mapping: self.component_mapping,
            format: self.properties.format,
        }
    }

    #[inline(always)]
    pub(crate) fn validate_range(
        &self,
        range_info: ImageRangeInfo,
    ) -> Option<ImageError>
    {
        if let Some(component_info) = range_info.component_info {
            if !self.has_mutable_format() && self.properties.format != component_info.format {
                return Some(ImageError::ImmutableFormat {
                    image_format: self.properties.format,
                    requested_format: component_info.format,
                })
            }
        }
        let subresource_info = range_info.subresource_info;
        if has_not_bits!(self.properties.aspect_mask, subresource_info.aspect_mask) {
            return Some(ImageError::AspectMismatch)
        }
        if subresource_info.base_mip_level + subresource_info.level_count.get() > self.properties.mip_levels ||
            subresource_info.base_array_layer + subresource_info.layer_count.get() > self.properties.array_layers
        {
            return Some(ImageError::SubresourceOutOfRange {
                image_mip_levels: self.properties.mip_levels,
                base_level: subresource_info.base_mip_level,
                level_count: subresource_info.level_count.get(),
                image_array_layers: self.properties.array_layers,
                base_layer: subresource_info.base_array_layer,
                layer_count: subresource_info.layer_count.get(),
            })
        }
        None
    }

    #[inline(always)]
    pub(crate) fn validate_layers(
        &self,
        layers: ImageSubresourceLayers
    ) -> Option<ImageError>
    {
        if has_not_bits!(self.properties.aspect_mask, layers.aspect_mask) {
            return Some(ImageError::AspectMismatch)
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
    pub(crate) fn get_view(&self) -> Result<vk::ImageView, Error> {
        let mut write = self.view.write().unwrap();
        if write.is_none() {
            let device = &self.device;
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: self.handle(),
                view_type: self.view_type(),
                format: self.properties.format,
                components: self.component_mapping.into(),
                subresource_range: self.properties.whole_subresource().into(),
                ..Default::default()
            };
            *write = NonZeroU64::new(unsafe {
                vk::Handle::as_raw(device.create_image_view(&create_info, None)?)
            });
        }

        Ok(vk::Handle::from_raw(write.unwrap().get()))
    }

    #[inline(always)]
    pub(crate) fn create_subview(
        &self,
        range_info: ImageRangeInfo,
    ) -> Result<(SlotIndex<vk::ImageView>, vk::ImageView), Error>
    {
        if let Some(err) = self.validate_range(range_info) {
            return Err(err.into())
        }
        let component_info = 
            if let Some(info) = range_info.component_info {
                info
            } else {
                self.component_info()
            };
        let device = &self.device;
        let create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image: self.handle(),
            view_type: self.view_type(),
            format: component_info.format,
            components: component_info.component_mapping.into(),
            subresource_range: range_info.subresource_info.into(),
            ..Default::default()
        };
        let view = unsafe {
            device.create_image_view(&create_info, None)?
        };
        let mut write = self.subviews.write().unwrap();
        let index = write.insert(view);
        Ok((index, view))
    }

    #[inline(always)]
    pub(crate) fn destroy_subview(
        &self,
        index: SlotIndex<vk::ImageView>
    ) -> Result<(), Error> {
        let mut write = self.subviews.write().unwrap();
        let view = write.remove(index)?;
        unsafe {
            self.device.destroy_image_view(view, None);
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn cmd_memory_barrier(
        &self,
        state: ImageState,
        command_buffer: vk::CommandBuffer,
        subresource_info: Option<ImageSubresourceRangeInfo>,
    ) -> Result<(), ImageError>
    {
        let mut write = self.state.write().unwrap();
        let device = &self.device;
        let subresource =
            if let Some(info) = subresource_info {
                if let Some(err) = self.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(err.into())
                }
                info.into()
            } else {
                self.properties.whole_subresource().into()
            };
        let memory_barrier = write.to_memory_barrier(
            self.handle(),
            state,
            subresource,
        );
        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                write.pipeline_stage,
                state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                &[memory_barrier]);
        }
        if subresource_info.is_none() {
            *write = state;
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) unsafe fn set_memory(&mut self, memory: Box<dyn DeviceMemory>) {
        debug_assert!(self.memory.is_none());
        self.memory = Some(memory);
    }
}

impl Drop for Image {

    fn drop(&mut self) {
        let device = &self.device;
        unsafe {
            for subview in self.subviews.read().unwrap().iter() {
                device.destroy_image_view(*subview, None);
            }
            if let Some(view) = *self.view.read().unwrap() {
                device.destroy_image_view(vk::Handle::from_raw(view.get()), None);
            }
            device.destroy_image(self.handle(), None);
            if let Some(memory) = self.memory.take() {
                memory.free_memory();
            }
        }
    }
}
