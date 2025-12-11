mod structs;
mod resource_pool;

pub use structs::*;
pub(crate) use resource_pool::ResourcePool;
use resource_pool::ResourcePoolContext;

use std::sync::Arc;

use ash::vk;

use crate::dev::error::Location;
use crate::gpu::*;

use super::LinearDeviceAlloc;

pub(crate) struct FrameContext<'a> {
    command_buffer: vk::CommandBuffer,
    device: Arc<ash::Device>,
    resource_pool: ResourcePoolContext<'a>,
}

impl<'a> FrameContext<'a> {

    #[inline(always)]
    pub fn new(
        command_buffer: vk::CommandBuffer,
        device: Arc<ash::Device>,
        context: GpuContext<'a>,
        resource_pool: &'a mut ResourcePool,
    ) -> Self
    {
        Self {
            device,
            resource_pool: ResourcePoolContext::new(
                context,
                resource_pool
            ),
            command_buffer,
        }
    }

    #[inline(always)]
    pub fn gpu(&mut self) -> &mut GpuContext<'a> {
        &mut self.resource_pool.context
    }

    #[inline(always)]
    pub fn device(&self) -> Arc<ash::Device> {
        self.device.clone()
    }

    #[inline(always)]
    pub fn add_image(
        &mut self,
        id: ImageId,
        loc: Location,
    ) -> Result<ResourceId> {
        self.resource_pool.add_image(id, loc)
    }

    #[inline(always)]
    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
        loc: Location,
    ) -> Result<ResourceId>
    {
        self.resource_pool.add_transient_image(f, loc)
    }

    #[inline(always)]
    pub fn set_render_image(
        &mut self,
        id: ResourceId,
        range_info: Option<ImageRangeInfo>,
        loc: Location,
    ) -> Result<()>
    {
        self.resource_pool.set_render_image(id, range_info, loc)?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_render_image(
        &mut self,
        graphics_queue: u32,
    ) -> Result<Option<(ImageId, Option<ImageRangeInfo>)>>
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
    pub fn get_image(&self, resource_id: ResourceId) -> Result<Arc<Image>> {
        self.resource_pool.get_image(resource_id)
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &self,
        id: ResourceId,
        state: ImageState,
        subresource_info: Option<ImageSubresourceRangeInfo>
    ) -> Result<()>
    {
        self.resource_pool.cmd_memory_barrier(id, state, self.command_buffer, subresource_info)
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceId) -> Result<(vk::ImageView, vk::ImageLayout)> {
        self.resource_pool.get_image_view(id)
    }

    #[inline(always)]
    pub fn create_image_view(
        &mut self,
        id: ResourceId,
        range_info: ImageRangeInfo,
    ) -> Result<(vk::ImageView, vk::ImageLayout)>
    {
        self.resource_pool.create_image_view(id, range_info)
    }
}
