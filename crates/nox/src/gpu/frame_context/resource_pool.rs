use std::sync::{Arc, RwLockWriteGuard};

use core::panic::Location;

use ash::vk;

use nox_mem::{
    slot_map::{GlobalSlotMap, SlotIndex},
    vec_types::GlobalVec,
};

use crate::dev::{
    has_bits,
    export::{
        *,
    },
    error::{Result, Error, Context, ErrorContext},
};

use crate::gpu::{*, memory_binder::MemoryBinder};

use super::{
    LinearDeviceAlloc,
    ResourceFlags,
};

pub(crate) struct ResourcePool
{
    transient_images: GlobalSlotMap<ImageId>,
    subviews: GlobalVec<(ImageId, SlotIndex<vk::ImageView>)>,
    device_alloc: LinearDeviceAlloc,
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
    pub context: GpuContext,
    pub render_image: Option<(ImageId, Option<ImageRangeInfo>, &'static Location<'static>)>,
    pub render_image_reset: Option<(ImageState, ImageSubresourceRangeInfo)>,
}

impl<'a> ResourcePoolContext<'a>
{

    #[inline(always)]
    pub fn new(
        mut context: Context,
        resource_pool: &'a mut ResourcePool,
    ) -> Self
    {
        for (_, resource) in &resource_pool.transient_images {
            context.destroy_image(*resource);
        }
        resource_pool.transient_images.clear_elements();
        for (image, index) in &resource_pool.subviews {
            if let Ok(image) = context.get_image(*image) {
                image.destroy_subview(*index).unwrap();
            }
        }
        resource_pool.subviews.resize(0, Default::default());
        unsafe {
            resource_pool.device_alloc.reset();
        }
        Self {
            pool: resource_pool,
            context,
            render_image: None,
            render_image_reset: None,
        }
    }

    #[inline(always)]
    pub fn render_done(
        &mut self,
        command_buffer: vk::CommandBuffer,
    )
    {
        if let Some((state, subresource)) = self.render_image_reset {
            self.context
                .get_image(self.render_image.unwrap().0).unwrap()
                .cmd_memory_barrier(
                    state,
                    command_buffer,
                    Some(subresource),
                    false,
                )
                .unwrap();
            self.render_image_reset = None;
        }
        self.render_image = None;
    }

    #[inline(always)]
    pub fn is_valid_id(&self, id: ResourceId) -> bool {
        if has_bits!(id.flags, ResourceFlags::Transient) {
            self.pool.transient_images.contains(id.index)
        }
        else {
            self.context.is_valid_image(id.image_id)
        }
    }

    #[inline(always)]
    pub fn add_image(
        &mut self,
        id: ImageId,
        loc: &'static Location<'static>,
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
            image_id: id,
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
        loc: &'static Location<'static>,
    ) -> Result<ResourceId>
    {
        let mut default_binder = self.context.default_memory_binder();
        let index = self.pool.transient_images
            .insert(self.context.create_image(
                ResourceBinderImage::Owned(&mut self.pool.device_alloc, Some(&mut |image| default_binder.bind_image_memory(image, None))),
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
            image_id,
            format: properties.format,
            samples: properties.samples,
            flags,
            loc: Some(loc),
        })
    }

    #[inline(always)]
    pub fn set_render_image(
        &mut self,
        resource_id: ResourceId,
        range_info: Option<ImageRangeInfo>,
        loc: &'static Location<'static>,
    ) -> Result<()>
    {
        let image = self.get_image(resource_id)?;
        if let Some(info) = range_info {
            if let Some(err) = image.validate_range(info) {
                return Err(Error::new("invalid image range", err))
            }
        }
        self.render_image = Some((resource_id.image_id, range_info, loc));
        Ok(())
    }

    #[inline(always)]
    pub fn get_render_image(
        &mut self,
        graphics_queue: u32,
        command_buffer: vk::CommandBuffer,
    ) -> Result<Option<(ImageId, Option<ImageRangeInfo>)>>
    {
        if self.render_image.is_none() {
            return Ok(None)
        }
        let Some((id, range_info, loc)) = self.render_image else {
            return Ok(None);
        };
        let dst_state = ImageState::new(
            vk::AccessFlags::SHADER_READ,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            graphics_queue,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        );
        let image = self.context
            .get_image(id)
            .context(ErrorContext::EventError(loc))?;
        let state = image.state();
        if dst_state != state {
            if range_info.is_some() && state.layout == vk::ImageLayout::UNDEFINED {
                image.cmd_memory_barrier(
                    dst_state,
                    command_buffer,
                    None,
                    false,
                ).unwrap();
            }
            else {
                image.cmd_memory_barrier(
                    dst_state,
                    command_buffer,
                    range_info.map(|v| v.subresource_info),
                    false,
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
    pub fn get_image(&self, resource_id: ResourceId) -> Result<Arc<Image>> {
        self.context
            .get_image(resource_id.image_id)?
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
        let image = self.context
            .get_image(id.image_id)
            .context(ErrorContext::EventError(id.loc))?;
        image
            .cmd_memory_barrier(state, command_buffer, subresource_info, false)
            .context("image memory barrier failed")
            .context(ErrorContext::EventError(id.loc))?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_image_view(&self, id: ResourceId) -> Result<(vk::ImageView, vk::ImageLayout)> {
        let src = self.context.get_image(id.image_id)?;
        Ok((src.get_view()?, src.layout()))
    }

    #[inline(always)]
    pub fn create_image_view(
        &mut self,
        id: ResourceId,
        range_info: ImageRangeInfo,
    ) -> Result<(vk::ImageView, vk::ImageLayout)>
    {
        let image = self.context.get_image(id.image_id)?;
        let (index, view) = image.create_subview(range_info)?;
        self.pool.subviews.push((id.image_id, index));
        Ok((view, image.layout()))
    }
}
