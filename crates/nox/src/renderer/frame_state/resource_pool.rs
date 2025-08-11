use std::sync::{Arc, RwLock};

use ash::vk;

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex},
    vec_types::{GlobalVec, Vector}
};

use crate::{
    has_bits, renderer::{
        global_resources::*,
        image::{Image, ImageBuilder, ImageRangeInfo, ImageSubresourceRangeInfo},
        linear_device_alloc::LinearDeviceAlloc,
        Error, 
        ImageState,
    }
};

use super::{
    ResourceID,
    ResourceFlags,
};

pub(crate) struct ResourcePool
{
    device: Arc<ash::Device>,
    pub global_resources: Arc<RwLock<GlobalResources>>,
    transient_images: GlobalSlotMap<ImageID>,
    subviews: GlobalVec<(ImageID, SlotIndex<vk::ImageView>)>,
    render_image: Option<(ImageID, Option<ImageRangeInfo>)>,
    render_image_reset: Option<(ImageState, ImageSubresourceRangeInfo)>,
    device_alloc: LinearDeviceAlloc,
}

impl ResourcePool
{

    #[inline(always)]
    pub fn new(
        device: Arc<ash::Device>,
        global_resources: Arc<RwLock<GlobalResources>>,
        device_alloc: LinearDeviceAlloc,
    ) -> Self
    {
        Self {
            device,
            global_resources,
            transient_images: GlobalSlotMap::new(),
            subviews: GlobalVec::new(),
            render_image: None,
            render_image_reset: None,
            device_alloc,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        let mut g = self.global_resources.write().unwrap();
        assert!(self.render_image == None);
        for resource in &self.transient_images {
            g.destroy_image(*resource).unwrap();
        }
        self.transient_images.clear_elements();
        for (image, index) in &self.subviews {
            if let Ok(image) = g.get_image(*image) {
                image.destroy_subview(*index).unwrap();
            }
        }
        self.subviews.resize(0, Default::default()).unwrap();
        unsafe {
            self.device_alloc.reset();
        }
    }

    #[inline(always)]
    pub fn render_done(
        &mut self,
        command_buffer: vk::CommandBuffer,
    )
    {
        if let Some((state, subresource)) = self.render_image_reset {
            self.global_resources
                .write().unwrap()
                .get_image(self.render_image.unwrap().0).unwrap()
                .cmd_memory_barrier(
                    state,
                    command_buffer,
                    Some(subresource),
                )
                .unwrap();
            self.render_image_reset = None;
        }
        self.render_image = None;
    }

    #[inline(always)]
    pub fn device(&self) -> Arc<ash::Device> {
        self.device.clone()
    }

    #[inline(always)]
    pub fn is_valid_id(&self, id: ResourceID) -> bool {
        if has_bits!(id.flags, ResourceFlags::Transient) {
            self.transient_images.contains(id.index)
        }
        else {
            self.global_resources.read().unwrap().is_valid_image(id.image_id)
        }
    }

    #[inline(always)]
    pub fn add_image(&mut self, id: ImageID) -> Result<ResourceID, Error> {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(id)?;
        let mut flags = 0;
        let properties = image.properties;
        if has_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
            flags |= ResourceFlags::Sampleable;
        }
        Ok(ResourceID {
            index: Default::default(),
            image_id: id,
            format: properties.format,
            samples: properties.samples,
            flags,
        })
    }

    #[inline(always)]
    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
    ) -> Result<ResourceID, Error>
    {
        let mut g = self.global_resources.write().unwrap();
        let index = self.transient_images
            .insert(g.create_image(&mut self.device_alloc, f)?);
        let image_id = self.transient_images[index];
        let image = g.get_image(image_id).unwrap();
        let mut flags = ResourceFlags::Transient.into();
        let properties = image.properties;
        if has_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
            flags |= ResourceFlags::Sampleable;
        }
        Ok(ResourceID {
            index,
            image_id,
            format: properties.format,
            samples: properties.samples,
            flags,
        })
    }

    #[inline(always)]
    pub fn set_render_image(
        &mut self,
        resource_id: ResourceID,
        range_info: Option<ImageRangeInfo>,
    ) -> Result<(), Error>
    {
        let image = self.get_image(resource_id)?;
        if let Some(info) = range_info {
            if let Some(err) = image.validate_range(info) {
                return Err(err.into())
            }
        }
        self.render_image = Some((resource_id.image_id, range_info));
        Ok(())
    }

    #[inline(always)]
    pub fn get_render_image(
        &mut self,
        graphics_queue: u32,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Option<(ImageID, Option<ImageRangeInfo>)>, Error>
    {
        if self.render_image.is_none() {
            return Ok(None)
        }
        let (id, range_info) = unsafe { self.render_image.unwrap_unchecked() };
        let dst_state = ImageState::new(
            vk::AccessFlags::SHADER_READ,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            graphics_queue,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        );
        let image = self.global_resources
            .write().unwrap()
            .get_image(id)?;
        let state = image.state();
        if dst_state != state {
            if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                image.cmd_memory_barrier(
                    dst_state,
                    command_buffer,
                    None,
                ).unwrap();
            }
            else {
                image.cmd_memory_barrier(
                    dst_state,
                    command_buffer,
                    range_info.map(|v| v.subresource_info)
                ).unwrap();
                if let Some(info) = range_info {
                    self.render_image_reset = Some((state, info.subresource_info));
                }
            }
        }
        Ok(Some(
            (id, range_info)
        ))
    }

    #[inline(always)]
    pub fn get_image(&self, resource_id: ResourceID) -> Result<Arc<Image>, Error> {
        Ok(
            if has_bits!(resource_id.flags, ResourceFlags::Transient) {
                self.global_resources
                    .read()
                    .unwrap()
                    .get_image(resource_id.image_id)?
            }
            else {
                self.global_resources
                    .read()
                    .unwrap()
                    .get_image(resource_id.image_id)?
            }
        )
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &self,
        id: ResourceID,
        state: ImageState,
        command_buffer: vk::CommandBuffer,
        subresource_info: Option<ImageSubresourceRangeInfo>
    ) -> Result<(), Error>
    {
        let g = self.global_resources.write().unwrap();
        let image = g.get_image(id.image_id)?;
        image.cmd_memory_barrier(state, command_buffer, subresource_info)?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceID) -> Result<(vk::ImageView, vk::ImageLayout), Error> {
        let g = self.global_resources.write().unwrap();
        let src = g.get_image(id.image_id)?;
        Ok((src.get_view()?, src.layout()))
    }

    #[inline(always)]
    pub fn create_image_view(
        &mut self,
        id: ResourceID,
        range_info: ImageRangeInfo,
    ) -> Result<(vk::ImageView, vk::ImageLayout), Error>
    {
        let g = self.global_resources.write().unwrap();
        let image = g.get_image(id.image_id)?;
        let (index, view) = image.create_subview(range_info)?;
        self.subviews.push((id.image_id, index)).unwrap();
        Ok((view, image.layout()))
    }

    #[inline(always)]
    pub(super) unsafe fn force_clean_up(&mut self) {
        unsafe {
            self.device_alloc.clean_up();
        }
    }
}
