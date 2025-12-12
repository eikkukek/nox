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

pub(crate) enum ImageSource {
    Owned(Arc<Image>),
    Swapchain(vk::Image, vk::ImageView, ImageState),
}

pub(crate) struct FrameContext<'a> {
    command_buffer: vk::CommandBuffer,
    device: Arc<ash::Device>,
    resource_pool: ResourcePoolContext<'a>,
    swapchain_image: vk::Image,
    swapchain_image_view: vk::ImageView,
    swapchain_format: vk::Format,
    swapchain_image_state: ImageState,
}

impl<'a> FrameContext<'a> {

    #[inline(always)]
    pub fn new(
        command_buffer: vk::CommandBuffer,
        context: GpuContext<'a>,
        resource_pool: &'a mut ResourcePool,
        swapchain_image: vk::Image,
        swapchain_image_view: vk::ImageView,
        swapchain_format: vk::Format,
        swapchain_image_state: ImageState,
    ) -> Self
    {
        Self {
            device: context.device(),
            resource_pool: ResourcePoolContext::new(
                context,
                resource_pool
            ),
            command_buffer,
            swapchain_image,
            swapchain_image_view,
            swapchain_format,
            swapchain_image_state,
        }
    }

    #[inline(always)]
    pub fn gpu(&self) -> &GpuContext<'a> {
        &self.resource_pool.context
    }

    #[inline(always)]
    pub fn gpu_mut(&mut self) -> &mut GpuContext<'a> {
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
    pub fn swapchain_image(&self, loc: Location) -> ResourceId {
        ResourceId {
            format: self.swapchain_format,
            samples: MSAA::X1,
            flags: ResourceFlags::SwapchainImage as u32,
            loc: Some(loc),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn get_image(&self, resource_id: ResourceId) -> Result<ImageSource> {
        if !resource_id.is_swapchain_image() {
            Ok(ImageSource::Owned(self.resource_pool.get_image(resource_id)?))
        } else {
            Ok(ImageSource::Swapchain(
                self.swapchain_image,
                self.swapchain_image_view,
                self.swapchain_image_state,
            ))
        }
    }

    #[inline(always)]
    pub fn get_swapchain_image_state(&self) -> ImageState {
        self.swapchain_image_state
    }

    #[inline(always)]
    pub fn set_swapchain_image_state(&mut self, state: ImageState) {
        self.swapchain_image_state = state;
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
