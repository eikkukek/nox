use std::sync::{Arc, RwLock};

use super::*;

use crate::renderer::global_resources::{GlobalResources, ImageID};

pub(crate) struct ImageSubresourceRange {
    pub global_resources: Arc<RwLock<GlobalResources>>,
    pub image_id: ImageID,
    pub view: Option<NonZeroU64>,
    pub state: ImageState,
    pub subresource_info: ImageSubresourceRangeInfo,
    pub component_info: ComponentInfo,
}

impl ImageSubresourceRange {

    #[inline(always)]
    pub fn new(
        range_info: ImageRangeInfo,
        image_id: ImageID,
        global_resources: Arc<RwLock<GlobalResources>>,
    ) -> Result<Self, ImageError>
    {
        let g = global_resources.read().unwrap();
        let image = g.get_image(image_id);
        if let Some(err) = image.validate_range(range_info) {
            return Err(err)
        }
        Ok(Self {
            global_resources: global_resources.clone(),
            image_id,
            view: None,
            state: image.state,
            subresource_info: range_info.subresource_info,
            component_info: range_info.component_info.unwrap_or(image.component_info()),
        })
    }
    
    #[inline(always)]
    pub fn properties(&self) -> ImageProperties {
        let mut properties = self.global_resources
            .read()
            .unwrap()
            .get_image(self.image_id)
            .properties;
        let subresource_info = self.subresource_info;
        properties.mip_levels = subresource_info.level_count.get();
        properties.array_layers = subresource_info.layer_count.get();
        properties.format = self.vk_format();
        properties
    }

    #[inline(always)]
    pub fn vk_format(&self) -> vk::Format {
        self.component_info.format
    }

    #[inline(always)]
    pub fn layout(&self) -> vk::ImageLayout {
        self.state.layout
    }

    #[inline(always)]
    pub fn samples(&self) -> MSAA {
        self.global_resources
            .read()
            .unwrap()
            .get_image(self.image_id)
            .samples()
    }

    #[inline(always)]
    pub fn get_view(&mut self) -> Result<vk::ImageView, Error> {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(self.image_id);
        if self.view.is_none() {
            let device = image.device();
            let subresource_info = self.subresource_info;
            let component_info = self.component_info;
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: image.handle(),
                view_type: image.view_type(),
                format: component_info.format,
                components: self.component_info.component_mapping.into(),
                subresource_range: subresource_info.into(),
                ..Default::default()
            };

            self.view = NonZeroU64::new(unsafe {
                vk::Handle::as_raw(device.create_image_view(&create_info, None)?)
            });

        }

        Ok(vk::Handle::from_raw(self.view.unwrap().get()))
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &mut self,
        state: ImageState,
        command_buffer: vk::CommandBuffer
    )
    {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(self.image_id);
        if self.state == state {
            return
        }
        let device = image.device();
        let memory_barrier = self.state.to_memory_barrier(
            image.handle(),
            state,
            self.subresource_info.into(),
        );
        unsafe {
            device.cmd_pipeline_barrier(command_buffer,
                self.state.pipeline_stage,
                state.pipeline_stage,
                Default::default(),
                Default::default(),
                Default::default(),
                &[memory_barrier],
            );
        }
        self.state = state;
    }
}

impl Drop for ImageSubresourceRange {

    fn drop(&mut self) {
        if let Some(view) = self.view {
            let g = self.global_resources.read().unwrap();
            let image = g.get_image(self.image_id);
            unsafe {
                image.device().destroy_image_view(vk::Handle::from_raw(view.get()), None);
            }
        }
    }
}
