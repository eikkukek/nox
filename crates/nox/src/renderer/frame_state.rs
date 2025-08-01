mod structs;
mod resource_pool;

pub use structs::*;
pub(crate) use resource_pool::ResourcePool;

use std::sync::{Arc, RwLock};

use ash::vk;

use crate::renderer::{
    global_resources::{GlobalResources, ImageSourceID},
    image::{ImageBuilder, ImageProperties, ImageRangeInfo, ImageState},
    linear_device_alloc::LinearDeviceAlloc,
    Error,
};

pub(crate) struct FrameState {
    render_image: Option<ResourceID>,
    resource_pool: ResourcePool,
    command_buffer: vk::CommandBuffer,
}

impl FrameState {

    pub fn new(
        device: Arc<ash::Device>,
        global_resources: Arc<RwLock<GlobalResources>>,
        device_alloc: LinearDeviceAlloc,
    ) -> Self
    {
        Self {
            render_image: None,
            resource_pool: ResourcePool::new(device, global_resources, device_alloc),
            command_buffer: vk::CommandBuffer::null(),
        }
    }

    pub fn init(&mut self, command_buffer: vk::CommandBuffer)
    {
        self.resource_pool.reset();
        self.command_buffer = command_buffer;
        self.render_image = None;
    }

    pub fn device(&self) -> Arc<ash::Device> {
        self.resource_pool.device()
    }

    pub fn add_image(&mut self, id: ImageSourceID) -> ResourceID {
        self.resource_pool.add_image(id)
    }

    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
    ) -> Result<ResourceID, Error>
    {
        self.resource_pool.add_transient_image(f)
    }

    pub fn add_transient_image_subresource(
        &mut self,
        resource_id: ResourceID,
        range_info: ImageRangeInfo,
    ) -> Result<ResourceID, Error>
    {
        self.resource_pool.add_transient_image_subresource(resource_id, range_info)
    }

    pub fn get_image_properties(&self, resource_id: ResourceID) -> ImageProperties {
        self.resource_pool.get_image_properties(resource_id)
    }

    pub fn set_render_image(&mut self, id: ResourceID) {
        self.render_image = Some(id);
    }

    pub fn render_image(&self) -> Option<ResourceID> {
        self.render_image
    }

    pub fn is_valid_resource_id(&self, id: ResourceID) -> bool {
        self.resource_pool.is_valid_id(id)
    }

    pub fn cmd_memory_barrier(
        &self,
        id: ResourceID,
        state: ImageState
    ) -> Option<SubresourceResetGuard>
    {
        self.resource_pool.cmd_memory_barrier(id, state, self.command_buffer)
    }

    pub fn get_image_view(&self, id: ResourceID) -> Result<(vk::ImageView, vk::ImageLayout), Error> {
        self.resource_pool.get_image_view(id)
    }
}
