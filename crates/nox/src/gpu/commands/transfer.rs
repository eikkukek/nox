use std::{
    sync::Arc,
};

use core::ops::{Deref, DerefMut};
use core::ptr;
use core::cell::UnsafeCell;

use ash::vk;

use nox_mem::{Allocator, vec_types::{FixedVec, GlobalVec, Vector}};

use super::*;

use crate::dev::{
    has_not_bits,
    format_location,
};

use crate::gpu::{memory_binder::MemoryBinder, *};

pub(crate) struct TransientCommandPool {
    device: Arc<ash::Device>,
    handle: vk::CommandPool,
    queue_family_index: u32,
}

impl TransientCommandPool {

    #[inline(always)]
    pub fn new(
        device: Arc<ash::Device>,
        queue_family_index: u32,
    ) -> Result<Self> {
        let handle = helpers::create_command_pool(
            &device,
            vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
        ).context("failed to create command pool")?;
        Ok(Self {
            device,
            handle,
            queue_family_index,
        })
    }

    #[inline(always)]
    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    #[inline(always)]
    pub fn handle(&self) -> vk::CommandPool {
        self.handle
    }

    #[inline(always)]
    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }
}

impl Drop for TransientCommandPool {

    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(
                self.handle,
                None
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SyncObjects {
    pub transfer_fence: vk::Fence,
    pub graphics_fence: vk::Fence,
    pub binary_semaphore: vk::Semaphore,
}

pub(crate) struct TransferCommandsStorage {
    transfer_command_pool: Arc<TransientCommandPool>,
    pub transfer_command_buffer: vk::CommandBuffer,
    graphics_command_pool: Arc<TransientCommandPool>,
    pub graphics_command_buffer: vk::CommandBuffer,
    staging_buffers: GlobalVec<vk::Buffer>,
    linear_device_alloc: LinearDeviceAllocLock,
    sync_objects: Option<SyncObjects>,
    signal_semaphores: GlobalVec<(TimelineSemaphoreId, u64)>,
    id: CommandRequestId,
}

impl TransferCommandsStorage {

    #[inline(always)]
    pub(crate) fn new(
        transfer_command_pool: Arc<TransientCommandPool>,
        transfer_command_buffer: vk::CommandBuffer,
        graphics_command_pool: Arc<TransientCommandPool>,
        graphics_command_buffer: vk::CommandBuffer,
        linear_device_alloc: LinearDeviceAllocLock,
        signal_semaphores: &[(TimelineSemaphoreId, u64)],
        id: CommandRequestId,
    ) -> Result<Self>
    {
        Ok(Self {
            transfer_command_pool,
            transfer_command_buffer,
            graphics_command_pool,
            graphics_command_buffer,
            staging_buffers: GlobalVec::new(),
            linear_device_alloc,
            sync_objects: None,
            signal_semaphores: signal_semaphores.into(),
            id,
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
            if let Some(objects) = self.sync_objects.take() {
                device.destroy_fence(objects.transfer_fence, None);
                device.destroy_fence(objects.graphics_fence, None);
                device.destroy_semaphore(objects.binary_semaphore, None);
            }
        }
    }
}

pub struct TransferCommands<'a, 'b> {
    storage: &'a mut TransferCommandsStorage,
    context: UnsafeCell<&'a mut GpuContext<'b>>,
}

impl<'a, 'b> Deref for TransferCommands<'a, 'b> {

    type Target = TransferCommandsStorage;

    fn deref(&self) -> &Self::Target {
        self.storage
    }
}

impl<'a, 'b> DerefMut for TransferCommands<'a, 'b> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.storage
    }
}

impl<'a, 'b> TransferCommands<'a, 'b> {

    #[inline(always)]
    pub(crate) fn new(
        storage: &'a mut TransferCommandsStorage,
        context: &'a mut GpuContext<'b>,
    ) -> Self
    {
        Self {
            storage,
            context: UnsafeCell::new(context),
        }
    }

    #[inline(always)]
    pub(crate) fn transfer_command_buffer(&self) -> vk::CommandBuffer {
        self.transfer_command_buffer
    }

    #[inline(always)]
    pub(crate) fn graphics_command_buffer(&self) -> vk::CommandBuffer {
        self.graphics_command_buffer
    }

    #[inline(always)]
    pub(crate) fn id(&self) -> CommandRequestId {
        self.id
    }

    #[inline(always)]
    pub(crate) fn get_sync_objects(
        &mut self
    ) -> Result<(bool, SyncObjects, &[(TimelineSemaphoreId, u64)], &GpuContext)>
    {
        let mut new = false;
        if self.sync_objects.is_none() {
            let fence_info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            let semaphore_info = vk::SemaphoreCreateInfo {
                s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
                ..Default::default()
            };
            let transfer_fence = unsafe {
                self.device()
                    .create_fence(&fence_info, None)
                    .context_with(|| format_location!("failed to create fence at {loc}"))?
            };
            let graphics_fence = unsafe {
                self.device()
                    .create_fence(&fence_info, None)
                    .context_with(|| format_location!("failed to create fence at {loc}"))?
            };
            let binary_semaphore = unsafe {
                self.device()
                    .create_semaphore(&semaphore_info, None)
                    .context_with(|| format_compact!("failed to create semaphore at {loc}"))?
            };
            self.sync_objects = Some(SyncObjects {
                transfer_fence,
                graphics_fence,
                binary_semaphore,
            });
            new = true;
        }
        Ok((new, self.sync_objects.unwrap(), &self.signal_semaphores, self.context.get_mut()))
    }

    pub(crate) fn device(&self) -> &ash::Device {
        self.transfer_command_pool.device()
    }

    pub fn gpu(&mut self) -> &mut GpuContext<'b>
    {
        self.context.get_mut()
    }

    #[inline(always)]
    pub fn reserve_staging_buffers(&mut self, capacity: usize) {
        self.staging_buffers.reserve(capacity);
    }

    #[inline(always)]
    pub fn clear_color_image(
        &mut self,
        image_id: ImageId,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator
    ) -> Result<()> {
        let image = self.context.get_mut().get_image(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new("image has incompatible usage", err))
        }
        let state = image.state();
        let transfer_queue_index = self.transfer_command_pool.queue_family_index();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            false,
        ).context("image memory barrier failed")?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            alloc
        ).context("vec error")?;
        if let Some(infos) = subresources {
            for info in infos.iter().map(|v| *v) {
                if let Some(err) = image.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(Error::new("given subresource range is incompatible with image", err))
                }
                ranges.push(info.into()).unwrap();
            }
        }
        else {
            ranges.push(image.properties.whole_subresource().into()).unwrap();
        }
        unsafe {
            self.device().cmd_clear_color_image(
                command_buffer,
                image.handle(),
                image.layout(),
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
        alloc: &impl Allocator,
    ) -> Result<()>
    {
        let image = self.context.get_mut().get_image(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new("image has incompatible usage", err))
        }
        let state = image.state();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.transfer_command_pool.queue_family_index(),
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            true,
        ).context("image memory barrier failed")?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            alloc,
        ).context("vec error")?;
        if let Some(infos) = subresources {
            for info in infos.iter().map(|v| *v) {
                if let Some(err) = image.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(Error::new("given subresource range is incompatible with image", err))
                }
                ranges.push(info.into()).unwrap();
            }
        }
        else {
            ranges.push(image.properties.whole_subresource().into()).unwrap();
        }
        unsafe {
            self.device().cmd_clear_depth_stencil_image(
                command_buffer,
                image.handle(),
                image.layout(),
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
        let context = unsafe { &mut *self.context.get() };
        let mut default_binder = context
            .default_memory_binder_mappable()
            .clone();
        let buffer = context.get_buffer_mut(buffer_id)?;
        if let Some(err) = buffer.validate_usage(vk::BufferUsageFlags::TRANSFER_DST) {
            return Err(Error::new("buffer has incompatible usage", err))
        }
        if let Some(err) = buffer.validate_range(offset, size) {
            return Err(Error::new("given buffer size and offset are out of range of the buffer", err))
        }
        if (data.len() as u64) < size {
            return Err(Error::just_context(format_compact!(
                "host copy out of range of host buffer, buffer size was {} while requested size was {size}", data.len()
            )))
        }
        let command_buffer = self.transfer_command_buffer;
        let mut dst_state = BufferState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            self.transfer_command_pool.queue_family_index(),
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
        let mut memory = self.linear_device_alloc
            .bind_buffer_memory(
                staging_buffer,
                Some(&mut |buffer| {
                    default_binder.bind_buffer_memory(buffer, None)
                })
            ).context("failed to bind staging buffer memory")?;
        let ptr = unsafe { memory.map_memory() }
            .context("failed to map staging buffer memory")?;

        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: offset,
            size,
        };

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr(), data.len());
            self.device().cmd_copy_buffer(
                command_buffer,
                staging_buffer,
                buffer.handle(),
                &[region],
            );
        };

        self.staging_buffers.push(staging_buffer);

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
        let image = self.context.get_mut().get_image(image_id)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new("image has incompatible usage", err))
        }
        let properties = image.properties;
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::QUEUE_FAMILY_IGNORED,
            vk::PipelineStageFlags::TRANSFER
        );
        let state = image.state();
        let command_buffer = self.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            true,
        ).unwrap();
        let mut subresource_layers = properties.all_layers(0);
        if let Some(layers) = layers {
            if let Some(err) = image.validate_layers(layers) {
                return Err(Error::new("given subresource range is incompatible with image", err))
            }
            subresource_layers = layers;
        }
        let mut image_offset = Default::default();
        let mut image_extent = properties.dimensions;
        if let Some(dimensions) = dimensions {
            if let Some(offset) = offset {
                image_offset = offset;
            }
            if image_offset.x < 0 || image_offset.x < 0 || image_offset.z < 0 {
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
        let mut default_binder = self.context.get_mut().default_memory_binder_mappable();
        let mut memory = self.linear_device_alloc
            .bind_buffer_memory(staging_buffer, Some(&mut |buffer| {
                default_binder.bind_buffer_memory(buffer, None)
            })).context("failed to bind staging buffer memory")?;
        let ptr = unsafe { memory.map_memory() }
            .context("failed to map staging buffer memory")?;
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: subresource_layers.into(),
            image_offset: image_offset.into(),
            image_extent: image_extent.into(),
        };

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr(), data.len());
            self.device().cmd_copy_buffer_to_image(
                command_buffer,
                staging_buffer,
                image.handle(),
                image.layout(),
                &[region],
            );
        };

        self.staging_buffers.push(staging_buffer);

        if dst_state.queue_family_index != state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, command_buffer, None, true).unwrap();
        }

        Ok(())
    } 

    #[inline(always)]
    pub fn gen_mip_maps_internal(
        &mut self,
        image: ImageId,
        filter: Filter,
    ) -> Result<()>
    {
        let filter = filter.into();
        let image = self.context.get_mut().get_image(image)?;
        if let Some(err) = image.validate_usage(vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(Error::new("image has incompatible usage", err))
        }
        let handle = image.handle();
        let properties = image.properties;
        let mip_levels = properties.mip_levels;
        let mut mip_dimensions = properties.dimensions;
        let graphics_queue_index = self.graphics_command_pool.queue_family_index();
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
        let command_buffer = self.graphics_command_buffer;
        let device = self.graphics_command_pool.device().clone();
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            false,
        ).context("image memory barrier failed")?;
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
            ).context("image subresource memory barrier failed")?;
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
        ).context("image subresource memory barrier failed")?;
        Ok(())
    }
}
