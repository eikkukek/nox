use std::sync::Arc;

use compact_str::format_compact;

use nox_ash::vk;

use nox_mem::{
    vec::{FixedVec32, Vec32, Vector},
    option::OptionExt,
};

use nox_alloc::arena::Arena;

use crate::dev::{
    error::*,
    format_location,
};

#[derive(Clone, Copy)]
pub(crate) struct BaseSyncObjects {
    pub transfer_fence: vk::Fence,
    pub graphics_fence: vk::Fence,
    pub binary_semaphore: vk::Semaphore,
}

pub(crate) struct SyncObjects<'a> {
    pub new: bool,
    pub base: BaseSyncObjects,
    pub signal_semaphores: &'a Vec32<(TimelineSemaphoreId, u64)>,
}

pub(crate) struct TransferCommandsStorage {
    pub transfer_command_pool: Arc<TransientCommandPool>,
    pub transfer_command_buffer: vk::CommandBuffer,
    pub graphics_command_pool: Arc<TransientCommandPool>,
    pub graphics_command_buffer: vk::CommandBuffer,
    pub staging_buffers: Vec32<vk::Buffer>,
    pub linear_binder: LinearBinderLock,
    pub base_sync_objects: Option<BaseSyncObjects>,
    pub signal_semaphores: Vec32<(TimelineSemaphoreId, u64)>,
    pub id: CommandRequestId,
}

impl TransferCommandsStorage {

    #[inline(always)]
    pub fn new(
        transfer_command_pool: Arc<TransientCommandPool>,
        transfer_command_buffer: vk::CommandBuffer,
        graphics_command_pool: Arc<TransientCommandPool>,
        graphics_command_buffer: vk::CommandBuffer,
        linear_binder: LinearBinderLock,
        signal_semaphores: &[(TimelineSemaphoreId, u64)],
        id: CommandRequestId,
    ) -> Result<Self>
    {
        Ok(Self {
            transfer_command_pool,
            transfer_command_buffer,
            graphics_command_pool,
            graphics_command_buffer,
            staging_buffers: Vec32::new(),
            linear_binder,
            base_sync_objects: None,
            signal_semaphores: signal_semaphores.into(),
            id,
        })
    }

    #[inline(always)]
    pub fn get_sync_objects(
        &mut self,
    ) -> Result<SyncObjects<'_>>
    {
        let mut new = false;
        let vk = self.transfer_command_pool.vk.clone();
        let &mut base = self.base_sync_objects.get_or_try_insert_with(|| {
            let fence_info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            let semaphore_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            };
            let transfer_fence = unsafe {
                vk.device()
                    .create_fence(&fence_info, None)
                    .context_with(|| format_location!("failed to create fence at {loc}"))?
            };
            let graphics_fence = unsafe {
                vk.device()
                    .create_fence(&fence_info, None)
                    .context_with(|| format_location!("failed to create fence at {loc}"))?
            };
            let binary_semaphore = unsafe {
                vk.device()
                    .create_semaphore(&semaphore_info, None)
                    .context_with(|| format_location!("failed to create semaphore at {loc}"))?
            };
            new = true;
            Ok(BaseSyncObjects {
                transfer_fence,
                graphics_fence,
                binary_semaphore,
            })
        })?;
        Ok(SyncObjects {
            new,
            base,
            signal_semaphores: &self.signal_semaphores
        })
    }
}

impl Drop for TransferCommandsStorage {

    fn drop(&mut self) {
        unsafe {
            let device = &self.transfer_command_pool.device();
            for buffer in self.staging_buffers.iter() {
                device.destroy_buffer(*buffer, None);
            }
            self.staging_buffers.clear();
            if let Some(objects) = self.base_sync_objects.take() {
                device.destroy_fence(objects.transfer_fence, None);
                device.destroy_fence(objects.graphics_fence, None);
                device.destroy_semaphore(objects.binary_semaphore, None);
            }
        }
    }
}

pub struct TransferCommands<'a> {
    storage: &'a mut TransferCommandsStorage,
    resources: Arc<Resources>,
}

impl<'a> TransferCommands<'a>
{

    #[inline(always)]
    pub(crate) fn new(
        storage: &'a mut TransferCommandsStorage,
        resources: Arc<Resources>,
    ) -> Self
    {
        Self {
            storage,
            resources,
        }
    }

    #[inline(always)]
    pub(crate) fn transfer_command_buffer(&self) -> vk::CommandBuffer {
        self.storage.transfer_command_buffer
    }

    #[inline(always)]
    pub(crate) fn graphics_command_buffer(&self) -> vk::CommandBuffer {
        self.storage.graphics_command_buffer
    }

    #[inline(always)]
    pub fn id(&self) -> CommandRequestId {
        self.storage.id
    } 

    pub(crate) fn device(&self) -> &ash::Device {
        self.storage.transfer_command_pool.device()
    }

    #[inline(always)]
    pub fn reserve_staging_buffers(&mut self, capacity: u32) {
        self.storage.staging_buffers.reserve(capacity);
    }

    #[inline(always)]
    pub fn clear_color_image(
        &mut self,
        image_id: ImageId,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
    ) -> Result<()> {
        let mut resources = self.resources.write();
        let tmp_alloc = resources.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let image = resources.get_image_mut(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new(err, "image has incompatible usage"))
        }
        let state = image.state();
        let transfer_queue_index = self.storage.transfer_command_pool.queue_family_index();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.storage.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            false,
        ).context_with(|| format_compact!(
            "image {:?} memory barrier failed",
            image_id,
        ))?;
        let ranges =
        if let Some(infos) = subresources {
            let mut ranges = FixedVec32::with_capacity(infos.len() as u32, &tmp_alloc)?;
            for &info in infos.iter() {
                if let Err(err) = image.properties.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(Error::new(err, "given subresource range is incompatible with image"))
                }
                ranges.push(info.into());
            }
            ranges
        }
        else {
            let mut ranges = FixedVec32::with_capacity(1, &tmp_alloc)?;
            ranges.push(image.properties.whole_subresource().into());
            ranges
        };
        unsafe {
            self.storage.transfer_command_pool.device().cmd_clear_color_image(
                command_buffer,
                image.handle(),
                image.state().layout,
                &clear_value.into(),
                &ranges,
            );
        }
        if state.queue_family_index != dst_state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, command_buffer, None, true).unwrap();
        }
        Ok(())
    } 

    #[inline(always)]
    pub fn clear_depth_stencil_image(
        &mut self,
        image_id: ImageId,
        depth: f32,
        stencil: u32,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
    ) -> Result<()>
    {
        let mut resources = self.resources.write();
        let tmp_alloc = resources.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let image = resources.get_image_mut(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new(err, "image has incompatible usage"))
        }
        let state = image.state();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.storage.transfer_command_pool.queue_family_index(),
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.storage.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            true,
        ).context_with(|| format_compact!(
            "image {:?} memory barrier failed",
            image_id,
        ))?;
        let ranges =
        if let Some(infos) = subresources {
            let mut ranges = FixedVec32::with_capacity(infos.len() as u32, &tmp_alloc)?;
            for &info in infos.iter() {
                if let Err(err) = image.properties.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(Error::new(err, "given subresource range is incompatible with image"))
                }
                ranges.push(info.into());
            }
            ranges
        }
        else {
            let mut ranges = FixedVec32::with_capacity(1, &tmp_alloc)?;
            ranges.push(image.properties.whole_subresource().into());
            ranges
        };
        unsafe {
            self.storage.transfer_command_pool.device().cmd_clear_depth_stencil_image(
                command_buffer,
                image.handle(),
                image.state().layout,
                &vk::ClearDepthStencilValue { depth, stencil },
                &ranges,
            );
        }
        if state.queue_family_index != dst_state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, command_buffer, None, true).unwrap();
        }
        Ok(())
    }

    #[inline(always)]
    pub fn copy_data_to_buffer(
        &mut self,
        buffer_id: BufferId, 
        data: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<()>
    {
        let mut resources = self.resources.write();
        let mut default_binder = resources.default_memory_binder_mappable();
        let buffer = resources.get_buffer_mut(buffer_id)?;
        if let Some(err) = buffer.validate_usage(vk::BufferUsageFlags::TRANSFER_DST) {
            return Err(Error::new(err, "buffer has incompatible usage"))
        }
        if let Some(err) = buffer.validate_range(offset, size) {
            return Err(Error::new(err, "given buffer size and offset are out of range of the buffer"))
        }
        if (data.len() as u64) < size {
            return Err(Error::just_context(format_compact!(
                "host copy out of range of host buffer, buffer size was {} while requested size was {size}",
                data.len()
            )))
        }
        let command_buffer = self.storage.transfer_command_buffer;
        let mut dst_state = BufferState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            self.storage.transfer_command_pool.queue_family_index(),
            vk::PipelineStageFlags::TRANSFER,
        );
        let state = buffer.state();
        buffer.cmd_memory_barrier(
            dst_state,
            command_buffer,
        );
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: data.len() as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let staging_buffer = unsafe {
            self.device()
                .create_buffer(&buffer_info, None)
                .context("failed to create staging buffer")?
        };
        let mut memory = self.storage.linear_binder
            .bind_buffer_memory(
                staging_buffer,
                Some(&mut default_binder),
            ).context("failed to bind staging buffer memory")?;
        let map = memory
            .map_memory()
            .context("failed to map staging buffer memory")?;

        map.copy_from_slice(data);

        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: offset,
            size,
        };

        unsafe {
            self.device().cmd_copy_buffer(
                command_buffer,
                staging_buffer,
                buffer.handle(),
                &[region],
            );
        };

        self.storage.staging_buffers.push(staging_buffer);

        if dst_state.queue_family_index != state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            buffer.cmd_memory_barrier(dst_state, command_buffer);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn copy_data_to_image(
        &mut self,
        image_id: ImageId,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset3D>,
        dimensions: Option<Dimensions>,
    ) -> Result<()>
    {
        let mut resources = self.resources.write();
        let mut default_binder = resources.default_memory_binder_mappable();
        let image = resources.get_image_mut(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new(err, "image has incompatible usage"))
        }
        let properties = image.properties;
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::PipelineStageFlags::TRANSFER
        );
        let state = image.state();
        let command_buffer = self.storage.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            true,
        ).unwrap();
        let mut subresource_layers = properties.all_layers(0);
        if let Some(layers) = layers {
            if let Some(err) = image.validate_layers(layers) {
                return Err(Error::new(err, "given subresource range is incompatible with image"))
            }
            subresource_layers = layers;
        }
        let mut image_offset = Default::default();
        let mut image_extent = properties.dimensions;
        if let Some(dimensions) = dimensions {
            if let Some(offset) = offset {
                image_offset = offset;
            }
            if image_offset.x < 0 || image_offset.y < 0 || image_offset.z < 0 {
                return Err(Error::just_context(ImageError::InvalidCopy {
                    image_dimensions: properties.dimensions,
                    copy_offset: image_offset,
                    copy_dimensions: dimensions,
                }))
            }
            if image_offset.x as u32 + dimensions.width > properties.dimensions.width ||
                image_offset.y as u32 + dimensions.height > properties.dimensions.height ||
                image_offset.z as u32 + dimensions.depth > properties.dimensions.depth
            {
                return Err(Error::just_context(ImageError::InvalidCopy {
                    image_dimensions: properties.dimensions,
                    copy_offset: image_offset,
                    copy_dimensions: dimensions,
                }))
            }
            image_extent = dimensions;
        }
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: data.len() as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let staging_buffer = unsafe {
            self.device().create_buffer(&buffer_info, None)
                .context("failed to create staging buffer")?
        };
        let mut memory = self.storage.linear_binder
            .bind_buffer_memory(staging_buffer, Some(&mut default_binder))
            .context("failed to bind staging buffer memory")?;
        let map = memory
            .map_memory()
            .context("failed to map staging buffer memory")?;
        map.copy_from_slice(data);
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: subresource_layers.into(),
            image_offset: image_offset.into(),
            image_extent: image_extent.into(),
        };

        unsafe {
            self.device().cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer,
                image.handle(),
                image.state().layout,
                &[region],
            );
        };

        self.storage.staging_buffers.push(staging_buffer);

        if dst_state.queue_family_index != state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, command_buffer, None, true).unwrap();
        }

        Ok(())
    } 

    #[inline(always)]
    pub fn gen_mip_maps(
        &mut self,
        image_id: ImageId,
        filter: Filter,
    ) -> Result<()>
    {
        let filter = filter.into();
        let mut resoures = self.resources.write();
        let image = resoures.get_image_mut(image_id)?;
        if let Some(err) = image.validate_usage(
            vk::ImageUsageFlags::TRANSFER_SRC |
            vk::ImageUsageFlags::TRANSFER_DST)
        {
            return Err(Error::new(err, "image has incompatible usage"))
        }
        let handle = image.handle();
        let properties = image.properties;
        let mip_levels = properties.mip_levels;
        let mut mip_dimensions = properties.dimensions;
        let graphics_queue_index = self.storage.graphics_command_pool.queue_family_index();
        let dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            graphics_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        let src_state = ImageState::new(
            vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            graphics_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.storage.graphics_command_buffer;
        let device = self.storage.graphics_command_pool.device().clone();
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            false,
        ).context_with(|| format_compact!(
            "image {:?} memory barrier failed",
            image_id,
        ))?;
        for i in 1..mip_levels {
            let mip_width = mip_dimensions.width as i32;
            let mip_height = mip_dimensions.height as i32;
            let subresource = ImageSubresourceRangeInfo::new(
                properties.aspect_mask,
                i - 1, 1, 
                0, properties.array_layers,
            ).unwrap();
            image.cmd_memory_barrier(
                src_state,
                command_buffer,
                Some(subresource),
                false
            ).context("image subresource memory barrier failed")?;
            let blit = vk::ImageBlit {
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D { x: mip_width , y: mip_height, z: 1 }
                ],
                src_subresource: ImageSubresourceLayers::new(
                    properties.aspect_mask,
                    i - 1,
                    0,
                    properties.array_layers,
                ).unwrap().into(),
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x:
                            if mip_width > 1 {
                                mip_width / 2
                            } else {
                                1
                            },
                        y:
                            if mip_height > 1 {
                                mip_height / 2
                            } else {
                                1
                            },
                        z: 1,
                    }
                ],
                dst_subresource: ImageSubresourceLayers::new(
                    properties.aspect_mask,
                    i,
                    0,
                    properties.array_layers
                ).unwrap().into(),
            };
            unsafe {
                device.cmd_blit_image(
                    command_buffer,
                    handle, vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    filter,
                );
            }
            image.cmd_memory_barrier(
                dst_state,
                command_buffer,
                Some(subresource),
                false
            ).context_with(|| format_compact!(
                "image {:?} subresource memory barrier failed",
                image_id,
            ))?;
            if mip_width > 1 {
                mip_dimensions.width /= 2;
            }
            if mip_height > 1 {
                mip_dimensions.height /= 2;
            }
        }
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            ImageSubresourceRangeInfo::new(
                properties.aspect_mask,
                mip_levels - 1, 1,
                0, properties.array_layers
            ),
            false,
        ).context_with(|| format_compact!(
            "image {:?} subresource memory barrier failed",
            image_id,
        ))?;
        Ok(())
    }
}
