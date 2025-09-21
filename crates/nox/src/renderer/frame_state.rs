mod structs;
mod resource_pool;

pub use structs::*;
pub(crate) use resource_pool::ResourcePool;

use std::sync::{Arc, RwLock};

use ash::vk;

use crate::renderer::{
    global_resources::{GlobalResources, ImageId},
    image::*,
    linear_device_alloc::LinearDeviceAlloc,
    Error,
};

pub(crate) struct FrameState {
    render_image: Option<(ResourceId, Option<ImageRangeInfo>)>,
    pub resource_pool: ResourcePool,
    command_buffer: vk::CommandBuffer,
}

impl FrameState {

    #[inline(always)]
    pub fn new(
        device: Arc<ash::Device>,
        global_resources: Arc<RwLock<GlobalResources>>,
        device_alloc: LinearDeviceAlloc,
    ) -> Result<Self, Error>
    {
        Ok(Self {
            render_image: None,
            resource_pool: ResourcePool::new(device, global_resources, device_alloc),
            command_buffer: vk::CommandBuffer::null(),
        })
    }

    #[inline(always)]
    pub fn init(&mut self, command_buffer: vk::CommandBuffer)
    {
        self.resource_pool.reset();
        self.command_buffer = command_buffer;
        self.render_image = None;
    }

    #[inline(always)]
    pub fn device(&self) -> Arc<ash::Device> {
        self.resource_pool.device()
    }

    #[inline(always)]
    pub fn add_image(&mut self, id: ImageId) -> Result<ResourceId, Error> {
        self.resource_pool.add_image(id)
    }

    #[inline(always)]
    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
    ) -> Result<ResourceId, Error>
    {
        self.resource_pool.add_transient_image(f)
    }

    #[inline(always)]
    pub fn set_render_image(
        &mut self,
        id: ResourceId,
        range_info: Option<ImageRangeInfo>,
    ) -> Result<(), Error>
    {
        self.resource_pool.set_render_image(id, range_info)?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_render_image(
        &mut self,
        graphics_queue: u32,
    ) -> Result<Option<(ImageId, Option<ImageRangeInfo>)>, Error>
    {
        self.resource_pool.get_render_image(graphics_queue, self.command_buffer)
    }

    #[inline(always)]
    pub fn render_done(&mut self) {
        self.resource_pool.render_done(self.command_buffer);
    }

    #[inline(always)]
    pub fn is_valid_resource_id(&self, id: ResourceId) -> bool {
        self.resource_pool.is_valid_id(id)
    }

    #[inline(always)]
    pub fn get_image(&self, resource_id: ResourceId) -> Result<Arc<Image>, Error> {
        self.resource_pool.get_image(resource_id)
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &self,
        id: ResourceId,
        state: ImageState,
        subresource_info: Option<ImageSubresourceRangeInfo>
    ) -> Result<(), Error>
    {
        self.resource_pool.cmd_memory_barrier(id, state, self.command_buffer, subresource_info)
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceId) -> Result<(vk::ImageView, vk::ImageLayout), Error> {
        self.resource_pool.get_image_view(id)
    }

    #[inline(always)]
    pub fn create_image_view(
        &mut self,
        id: ResourceId,
        range_info: ImageRangeInfo,
    ) -> Result<(vk::ImageView, vk::ImageLayout), Error>
    {
        self.resource_pool.create_image_view(id, range_info)
    }

    #[inline(always)]
    pub unsafe fn force_clean_up(&mut self) {
        unsafe {
            self.resource_pool.force_clean_up();
        }
    }
}
