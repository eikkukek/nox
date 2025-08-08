use std::sync::{Arc, RwLock};

use crate::renderer::{memory_binder::DeviceMemory};

use super::*;

pub(crate) struct Image {
    pub(super) handle: NonZeroU64,
    pub(super) memory: Option<Box<dyn DeviceMemory>>,
    pub(super) view: RwLock<Option<NonZeroU64>>,
    pub(super) device: Arc<ash::Device>,
    pub(super) state: RwLock<ImageState>,
    pub(super) properties: ImageProperties,
    pub(super) component_mapping: ComponentMapping,
}

impl Image {

    #[inline(always)]
    pub(crate) fn handle(&self) -> vk::Image {
        vk::Handle::from_raw(self.handle.get())
    }

    #[inline(always)]
    pub(crate) fn view_type(&self) -> vk::ImageViewType {
        self.properties.view_type()
    }

    #[inline(always)]
    pub(crate) fn device(&self) -> &ash::Device {
        &self.device
    }

    #[inline(always)]
    pub(crate) fn properties(&self) -> ImageProperties {
        self.properties
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
    pub fn samples(&self) -> MSAA {
        self.properties.samples
    }

    #[inline(always)]
    pub(crate) fn vk_format(&self) -> vk::Format {
        self.properties.format
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
    pub(crate) fn cmd_memory_barrier(
        &self,
        state: ImageState,
        command_buffer: vk::CommandBuffer,
    )
    {
        let mut write = self.state.write().unwrap();
        if *write == state {
            return
        }
        let device = &self.device;
        let memory_barrier = write.to_memory_barrier(
            self.handle(),
            state,
            self.properties.whole_subresource(),
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
        *write = state;
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
