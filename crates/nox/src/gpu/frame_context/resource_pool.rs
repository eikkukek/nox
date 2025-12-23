use std::sync::Arc;

use ash::vk;

use rustc_hash::FxHashMap;

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex},
    vec_types::GlobalVec,
};

use crate::dev::{
    has_bits,
    error::{Result, Context, ErrorContext, Location},
};

use crate::gpu::{*, memory_binder::MemoryBinder};

use super::{
    LinearDeviceAlloc,
    ResourceFlags,
    ImageSourceId,
};

pub(crate) struct ResourcePool
{
    transient_images: GlobalSlotMap<ImageId>,
    subviews: GlobalVec<(ImageId, SlotIndex<vk::ImageView>)>,
    device_alloc: LinearDeviceAlloc,
    pub(super) swapchain_frame_data: FxHashMap<win::WindowId, FrameData>,
    frame: u64,
}

impl ResourcePool {

    #[inline(always)]
    pub fn new(
        device_alloc: LinearDeviceAlloc,
    ) -> Self {
        Self {
            transient_images: GlobalSlotMap::new(),
            subviews: GlobalVec::new(),
            device_alloc,
            swapchain_frame_data: FxHashMap::default(),
            frame: 0,
        }
    }

    #[inline(always)]
    pub unsafe fn force_clean_up(&mut self) {
        unsafe {
            self.device_alloc.clean_up();
        }
    }
}

pub(super) struct ResourcePoolContext<'a> {
    pub pool: &'a mut ResourcePool,
    pub context: GpuContext<'a>,
}

impl<'a> ResourcePoolContext<'a>
{

    #[inline(always)]
    pub fn new(
        mut context: GpuContext<'a>,
        resource_pool: &'a mut ResourcePool,
        swapchains: impl Iterator<Item = (win::WindowId, FrameData)>,
    ) -> Self
    {
        for (_, resource) in &resource_pool.transient_images {
            context.destroy_image(*resource);
        }
        resource_pool.transient_images.clear();
        for (image, index) in &resource_pool.subviews {
            if let Ok(image) = context.get_image(*image) {
                image.destroy_subview(*index).unwrap();
            }
        }
        resource_pool.subviews.clear();
        resource_pool.swapchain_frame_data.clear();
        for (id, frame_data) in swapchains {
            resource_pool.swapchain_frame_data
                .entry(id)
                .or_insert(frame_data);
        }
        unsafe {
            resource_pool.device_alloc.reset();
        }
        resource_pool.frame += 1;
        Self {
            pool: resource_pool,
            context,
        }
    }

    #[inline(always)]
    pub fn current_frame(&self) -> u64 {
        self.pool.frame
    }

    #[inline(always)]
    pub fn add_image(
        &mut self,
        id: ImageId,
        loc: Location,
    ) -> Result<ResourceId>
    {
        let image = self.context.get_image(id)?;
        let mut flags = 0;
        let properties = image.properties;
        if has_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
            flags |= ResourceFlags::Sampleable;
        }
        Ok(ResourceId {
            index: Default::default(),
            source: ImageSourceId::Owned(id),
            format: properties.format,
            samples: properties.samples,
            flags,
            loc: Some(loc),
        })
    }

    #[inline(always)]
    pub fn add_transient_image<F: FnMut(&mut ImageBuilder)>(
        &mut self,
        f: F,
        loc: Location,
    ) -> Result<ResourceId>
    {
        let mut default_binder = self.context.default_memory_binder();
        let index = self.pool.transient_images
            .insert(self.context.create_image(
                ResourceBinderImage::Owned(
                    &mut self.pool.device_alloc,
                    Some(&mut |image| default_binder.bind_image_memory(image, None))
                ),
                f,
            )?);
        let image_id = self.pool.transient_images[index];
        let image = self.context.get_image(image_id).unwrap();
        let mut flags = ResourceFlags::Transient.into();
        let properties = image.properties;
        if has_bits!(properties.usage, vk::ImageUsageFlags::SAMPLED) {
            flags |= ResourceFlags::Sampleable;
        }
        Ok(ResourceId {
            index,
            source: ImageSourceId::Owned(image_id),
            format: properties.format,
            samples: properties.samples,
            flags,
            loc: Some(loc),
        })
    } 

    #[inline(always)]
    pub fn get_image(&self, id: ImageId) -> Result<Arc<Image>> {
        self.context
            .get_image(id)
            .context("couldn't find image")
    }

    #[inline(always)]
    pub fn cmd_memory_barrier(
        &self,
        id: ResourceId,
        state: ImageState,
        command_buffer: vk::CommandBuffer,
        subresource_info: Option<ImageSubresourceRangeInfo>
    ) -> Result<()>
    {
        match id.source {
            ImageSourceId::Owned(image_id) => {
                let image = self.context
                    .get_image(image_id)
                    .context(ErrorContext::EventError(id.location_or_this()))?;
                image
                    .cmd_memory_barrier(state, command_buffer, subresource_info, false)
                    .context("image memory barrier failed")
                    .context(ErrorContext::EventError(id.location_or_this()))?;
                Ok(())
            },
            ImageSourceId::SwapchainImage(_) => {
                Err(Error::just_context("swapchain image used where owned image expected"))
                .context(ErrorContext::EventError(id.location_or_this()))
            },
        }
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceId) -> Result<(vk::ImageView, vk::ImageLayout)> {
        match id.source {
            ImageSourceId::Owned(image_id) => {
                let src = self.context
                    .get_image(image_id)
                    .context(ErrorContext::EventError(id.location_or_this()))?;
                Ok((
                    src.get_view()
                    .context("failed to get image view")
                    .context(ErrorContext::EventError(id.location_or_this()))?,
                    src.layout()
                ))
            },
            ImageSourceId::SwapchainImage(_) => {
                Err(Error::just_context("swapchain image used where owned image expected"))
                .context(ErrorContext::EventError(id.location_or_this()))
            },
        }
    }

    #[inline(always)]
    pub fn create_image_view(
        &mut self,
        id: ResourceId,
        range_info: ImageRangeInfo,
    ) -> Result<(vk::ImageView, vk::ImageLayout)>
    {
        match id.source {
            ImageSourceId::Owned(image_id) => {
                let image = self.context
                    .get_image(image_id)
                    .context(ErrorContext::EventError(id.location_or_this()))?;
                let (index, view) = image
                    .create_subview(range_info)
                    .context("failed to create image subview")
                    .context(ErrorContext::EventError(id.location_or_this()))?;
                self.pool.subviews.push((image_id, index));
                Ok((view, image.layout()))
            },
            ImageSourceId::SwapchainImage(_) => {
                Err(Error::just_context("swapchain image used where owned image expected"))
                .context(ErrorContext::EventError(id.location_or_this()))
            },
        }
    }
}
