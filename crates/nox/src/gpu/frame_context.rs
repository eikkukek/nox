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
    Swapchain(win::WindowId, vk::Image, vk::ImageView, vk::Extent2D),
}

pub(crate) struct FrameContext<'a> {
    command_buffer: vk::CommandBuffer,
    vk: Arc<Vulkan>,
    resource_pool: ResourcePoolContext<'a>,
}

impl<'a> FrameContext<'a> {

    #[inline(always)]
    pub fn new(
        command_buffer: vk::CommandBuffer,
        gpu: GpuContext<'a>,
        resource_pool: &'a mut ResourcePool,
        swapchains: impl Iterator<Item = (win::WindowId, FrameData)>,
    ) -> Self
    {
        Self {
            vk: gpu.vk().clone(),
            resource_pool: ResourcePoolContext::new(
                gpu,
                resource_pool,
                swapchains,
            ),
            command_buffer,
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
    pub fn swapchain_count(&self) -> usize {
        self.resource_pool.pool.swapchain_frame_data.len()
    }

    #[inline(always)]
    pub fn vk(&self) -> &Arc<Vulkan> {
        &self.vk
    }

    #[inline(always)]
    pub fn current_frame(&self) -> u64 {
        self.resource_pool.current_frame()
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
    pub fn swapchain_image(
        &mut self,
        win_id: win::WindowId,
        loc: Location,
    ) -> Result<ResourceId>
    {
        let frame_data = self.resource_pool.pool
            .swapchain_frame_data.get(&win_id)
            .ok_or_else(|| Error::just_context(format_compact!("invalid window id {win_id:?}")))?;
        Ok(ResourceId {
            format: frame_data.format,
            samples: MSAA::X1,
            source: ImageSourceId::SwapchainImage(win_id),
            loc: Some(loc),
            ..Default::default()
        })
    }

    #[inline(always)]
    pub fn get_image(&self, resource_id: ResourceId) -> Result<ImageSource> {
        match resource_id.source {
            ImageSourceId::Owned(id) => {
                Ok(ImageSource::Owned(self.resource_pool
                    .get_image(id)
                    .context(ErrorContext::EventError(resource_id.location_or_this()))?
                ))
            },
            ImageSourceId::SwapchainImage(id) => {
                let frame_data = self.resource_pool.pool
                    .swapchain_frame_data.get(&id)
                    .ok_or_else(|| Error::just_context(format_compact!("invalid window id {id:?}")))?;
                Ok(ImageSource::Swapchain(
                    id, frame_data.image,
                    frame_data.image_view,
                    frame_data.extent.into(),
                ))
            },
        }
    }

    #[inline(always)]
    pub fn swapchain_image_state(&mut self, win_id: win::WindowId) -> Option<&mut ImageState> {
        self.resource_pool.pool.swapchain_frame_data
            .get_mut(&win_id)
            .map(|frame_data| &mut frame_data.image_state)
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
