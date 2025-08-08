use std::sync::{Arc, RwLock};

use ash::vk;

use nox_mem::{vec_types::{GlobalVec, Vector}};

use crate::renderer::{
    global_resources::{
        GlobalResources, 
        ImageSourceID,
    },
    image::{ImageBuilder, ImageProperties, ImageRangeInfo},
    linear_device_alloc::LinearDeviceAlloc,
    Error, 
    ImageState,
};

use super::{
    ResourceID,
    SubresourceResetGuard,
};

pub(crate) struct ResourcePool
{
    device: Arc<ash::Device>,
    global_resources: Arc<RwLock<GlobalResources>>,
    transient_image_sources: GlobalVec<ImageSourceID>,
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
            transient_image_sources: GlobalVec::with_capacity(4).unwrap(),
            device_alloc,
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        let mut g = self.global_resources.write().unwrap();
        for id in self.transient_image_sources.iter() {
            g.destroy_image_source(*id).unwrap();
        }
        self.transient_image_sources.clear();
        unsafe {
            self.device_alloc.reset();
        }
    }

    #[inline(always)]
    pub fn device(&self) -> Arc<ash::Device> {
        self.device.clone()
    }

    #[inline(always)]
    pub fn is_valid_id(&self, id: ResourceID) -> bool {
        self.global_resources.read().unwrap().is_valid_image_id(id.id)
    }

    #[inline(always)]
    pub fn add_image(&mut self, id: ImageSourceID) -> Result<ResourceID, Error> {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image_source(id)?;
        Ok(ResourceID {
            id,
            format: image.vk_format(),
            samples: image.samples(),
            is_transient: false,
        })
    }

    #[inline(always)]
    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
    ) -> Result<ResourceID, Error>
    {
        let mut g = self.global_resources.write().unwrap();
        let builder = ImageBuilder::new(self.device.clone());
        let id = *self.transient_image_sources
            .push(g.create_image(f, &mut self.device_alloc)?.into())
            .unwrap();
        Ok(ResourceID {
            id,
            format: builder.format,
            samples: builder.samples,
            is_transient: true,
        })
    }

    #[inline(always)]
    pub fn add_transient_image_subresource(
        &mut self,
        resource_id: ResourceID,
        range_info: ImageRangeInfo,
    ) -> Result<ResourceID, Error>
    {
        let img_id = match resource_id.id {
            ImageSourceID::ImageID(id) => {
                id
            },
            _ => panic!("resource ID must contain an ImageID when creating subresource")
        };
        let mut g = self.global_resources.write().unwrap();
        let sub_id = g.create_image_subresource(
            img_id,
            range_info
        )?.into();
        if !resource_id.is_transient {
            self.transient_image_sources.push(sub_id).unwrap();
        }
        Ok(ResourceID {
            id: sub_id,
            format: resource_id.vk_format(),
            samples: resource_id.samples(),
            is_transient: true,
        })
    }

    #[inline(always)]
    pub fn get_image_properties(&self, resource_id: ResourceID) -> Result<ImageProperties, Error> {
        Ok(self.global_resources
            .read()
            .unwrap()
            .get_image_source(resource_id.id)?
            .properties()
        )
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &self,
        id: ResourceID,
        state: ImageState,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Option<SubresourceResetGuard>, Error>
    {
        let mut g = self.global_resources.write().unwrap();
        let mut source = g.get_mut_image_source(id.id)?;
        source.cmd_memory_barrier(state, command_buffer);
        Ok(if let ImageSourceID::SubresourceID(id) = id.id {
            Some(SubresourceResetGuard {
                resources: self.global_resources.clone(),
                command_buffer,
                dst_state: source.state(),
                id,
            })
        }
        else {
            None
        })
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceID) -> Result<(vk::ImageView, vk::ImageLayout), Error> {
        let mut g = self.global_resources.write().unwrap();
        let mut src = g.get_mut_image_source(id.id)?;
        Ok((src.get_view()?, src.layout()))
    }

    #[inline(always)]
    pub(super) unsafe fn force_clean_up(&mut self) {
        unsafe {
            self.device_alloc.clean_up();
        }
    }
}
