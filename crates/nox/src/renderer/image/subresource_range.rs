use std::sync::Arc;

use super::*;

pub(crate) struct ImageSubresourceRange {
    pub image: Arc<Image>,
    pub view: Option<NonZeroU64>,
    pub state: ImageState,
    pub subresource_info: ImageSubresourceRangeInfo,
    pub component_info: ComponentInfo,
    pub cube_map: bool,
}

impl ImageSubresourceRange {

    #[inline(always)]
    pub fn new(
        image: Arc<Image>,
        range_info: ImageRangeInfo,
        cube_map: bool,
    ) -> Result<Self, Error>
    {
        if let Some(err) = image.validate_range(range_info) {
            return Err(err.into())
        }
        Ok(Self {
            image: image.clone(),
            view: None,
            state: *image.state.read().unwrap(),
            subresource_info: range_info.subresource_info,
            component_info: range_info.component_info.unwrap_or(image.component_info()),
            cube_map,
        })
    }
    
    #[inline(always)]
    pub fn properties(&self) -> ImageProperties {
        let mut properties = self.image.properties();
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
        self.image.samples()
    }

    #[inline(always)]
    pub fn get_view(&mut self) -> Result<vk::ImageView, Error> {
        if self.view.is_none() {
            let device = self.image.device();
            let subresource_info = self.subresource_info;
            let component_info = self.component_info;
            let mut view_type = vk::ImageViewType::TYPE_2D;
            if self.image.properties.dimensions.depth > 1 {
                view_type = vk::ImageViewType::TYPE_3D
            }
            else if self.cube_map {
                debug_assert!(self.subresource_info.layer_count.get().is_multiple_of(6));
                if self.subresource_info.layer_count.get() > 6 {
                    view_type = vk::ImageViewType::CUBE_ARRAY;
                }
                else {
                    view_type = vk::ImageViewType::CUBE;
                }
            }
            else if self.subresource_info.layer_count.get() > 1 {
                view_type = vk::ImageViewType::TYPE_2D_ARRAY;
            }
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: self.image.handle(),
                view_type: view_type,
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
        if self.state == state {
            return
        }
        let device = self.image.device();
        let memory_barrier = self.state.to_memory_barrier(
            self.image.handle(),
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
        self.image.state.write().unwrap().layout = self.layout();
    }
}

impl Drop for ImageSubresourceRange {

    fn drop(&mut self) {
        if let Some(view) = self.view {
            unsafe {
                self.image.device().destroy_image_view(vk::Handle::from_raw(view.get()), None);
            }
        }
    }
}
