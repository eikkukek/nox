use std::{
    sync::{Arc, RwLock},
};

use core::ptr;

use ash::vk;

use nox_mem::{vec_types::{FixedVec, GlobalVec, Vector}, GLOBAL_ALLOC};

use super::*;

use crate::{
    has_not_bits,
    renderer::{
        global_resources::*,
        image::*,
        buffer::*,
        frame_graph::ClearColorValue,
        linear_device_alloc::LinearDeviceAlloc,
        memory_binder::MemoryBinder,
        BufferError,
        Error
    }
};

pub struct TransferCommands {
    device: Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    global_resources: Arc<RwLock<GlobalResources>>,
    staging_buffers: GlobalVec<vk::Buffer>,
    linear_device_alloc: Arc<RwLock<LinearDeviceAlloc>>,
    fence: Option<vk::Fence>,
    transfer_queue_index: u32,
    id: CommandRequestID,
}

impl TransferCommands {

    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        command_pool: vk::CommandPool,
        global_resources: Arc<RwLock<GlobalResources>>,
        linear_device_alloc: Arc<RwLock<LinearDeviceAlloc>>,
        staging_buffer_capacity: u32,
        transfer_queue_index: u32,
        id: CommandRequestID,
    ) -> Result<Self, Error>
    {
        Ok(Self {
            device: device.clone(),
            command_buffer,
            command_pool,
            global_resources,
            staging_buffers: GlobalVec::with_capacity(staging_buffer_capacity as usize),
            linear_device_alloc,
            fence: None,
            transfer_queue_index,
            id,
        })
    }

    pub(crate) fn vk_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub(crate) fn id(&self) -> CommandRequestID {
        self.id
    }

    pub(crate) fn get_fence(&mut self) -> Result<(bool, vk::Fence), vk::Result> {
        let mut new = false;
        if self.fence.is_none() {
            let info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            self.fence = Some(unsafe {
                self.device.create_fence(&info, None)?
            });
            new = true;
        }
        Ok((new, self.fence.unwrap()))
    }

    #[inline(always)]
    pub fn clear_color_image(
        &mut self,
        image_id: ImageID,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
    ) -> Result<(), Error>
    {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(image_id)?;
        if has_not_bits!(image.properties.usage, vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(ImageError::UsageMismatch {
                missing_usage: vk::ImageUsageFlags::TRANSFER_DST,
            }.into())
        }
        let state = image.state();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        image.cmd_memory_barrier(
            ImageState::new(
                vk::AccessFlags::TRANSFER_WRITE,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                self.transfer_queue_index,
                vk::PipelineStageFlags::TRANSFER
            ),
            self.command_buffer,
            None,
        )?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            &GLOBAL_ALLOC
        )?;
        if let Some(infos) = subresources {
            for info in infos.iter().map(|v| *v) {
                if let Some(err) = image.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(err.into())
                }
                ranges.push(info.into()).unwrap();
            }
        }
        else {
            ranges.push(image.properties.whole_subresource().into()).unwrap();
        }
        unsafe {
            self.device.cmd_clear_color_image(
                self.command_buffer,
                image.handle(),
                image.layout(),
                &clear_value.into(),
                &ranges,
            );
        }
        if state.queue_family_index != dst_state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, self.command_buffer, None).unwrap();
        }
        Ok(())
    }

    #[inline(always)]
    pub fn clear_depth_stencil_image(
        &mut self,
        image_id: ImageID,
        depth: f32,
        stencil: u32,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
    ) -> Result<(), Error>
    {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(image_id)?;
        if has_not_bits!(image.properties.usage, vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(ImageError::UsageMismatch {
                missing_usage: vk::ImageUsageFlags::TRANSFER_DST,
            }.into())
        }
        let state = image.state();
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        image.cmd_memory_barrier(
            dst_state,
            self.command_buffer,
            None,
        )?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            &GLOBAL_ALLOC,
        )?;
        if let Some(infos) = subresources {
            for info in infos.iter().map(|v| *v) {
                if let Some(err) = image.validate_range(ImageRangeInfo::new(info, None)) {
                    return Err(err.into())
                }
                ranges.push(info.into()).unwrap();
            }
        }
        else {
            ranges.push(image.properties.whole_subresource().into()).unwrap();
        }
        unsafe {
            self.device.cmd_clear_depth_stencil_image(
                self.command_buffer,
                image.handle(),
                image.layout(),
                &vk::ClearDepthStencilValue { depth, stencil },
                &ranges,
            );
        }
        if state.queue_family_index != dst_state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, self.command_buffer, None).unwrap();
        }
        Ok(())
    }

    #[inline(always)]
    pub fn copy_data_to_buffer(
        &mut self,
        buffer_id: BufferID, 
        data: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<(), Error>
    {
        let mut g = self.global_resources.write().unwrap();
        let buffer = g.get_mut_buffer(buffer_id)?;
        let properties = buffer.properties();
        if has_not_bits!(properties.usage, vk::BufferUsageFlags::TRANSFER_DST) {
            return Err(BufferError::UsageMismatch {
                missing_usage: vk::BufferUsageFlags::TRANSFER_DST
            }.into())
        }
        if properties.size < offset + size {
            return Err(BufferError::OutOfRange {
                buffer_size: properties.size, requested_offset: offset, requested_size: size,
            }.into())
        }
        if (data.len() as u64) < size {
            return Err(Error::InvalidHostCopy {
                copy_size: size, host_buffer_size: data.len()
            })
        }
        let mut dst_state = BufferState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            self.transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER,
        );
        let state = buffer.state();
        buffer.cmd_memory_barrier(
            dst_state,
            self.command_buffer,
        );
        let device = &self.device;
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: data.len() as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let staging_buffer = unsafe {
            device.create_buffer(&buffer_info, None)?
        };
        let memory = self.linear_device_alloc
            .write()
            .expect("LinearDeviceAlloc lock poisoned")
            .bind_buffer_memory(staging_buffer)?;
        let ptr = unsafe { memory.get_mapped_memory() }.unwrap();

        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: offset,
            size,
        };

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr(), data.len());
            self.device.cmd_copy_buffer(
                self.command_buffer,
                staging_buffer,
                buffer.handle(),
                &[region],
            );
        };

        self.staging_buffers.push(staging_buffer);

        if dst_state.queue_family_index != state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            buffer.cmd_memory_barrier(dst_state, self.command_buffer);
        }

        Ok(())
    }

    #[inline(always)]
    pub fn copy_data_to_image(
        &mut self,
        image_id: ImageID,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset>,
        dimensions: Option<Dimensions>,
    ) -> Result<(), Error>
    {
        let g = self.global_resources.read().unwrap();
        let image = g.get_image(image_id)?;
        let properties = image.properties;
        if has_not_bits!(properties.usage, vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(ImageError::UsageMismatch {
                missing_usage: vk::ImageUsageFlags::TRANSFER_DST
            }.into())
        }
        let mut dst_state = ImageState::new(
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.transfer_queue_index,
            vk::PipelineStageFlags::TRANSFER
        );
        let state = image.state();
        image.cmd_memory_barrier(
            dst_state,
            self.command_buffer,
            None,
        ).unwrap();
        let mut subresource_layers = properties.all_layers(0);
        if let Some(layers) = layers {
            if let Some(err) = image.validate_layers(layers) {
                return Err(err.into())
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
                return Err(ImageError::InvalidCopy {
                    image_dimensions: properties.dimensions,
                    copy_offset: image_offset,
                    copy_dimensions: dimensions,
                }.into())
            }
            if image_offset.x as u32 + dimensions.width > properties.dimensions.width ||
                image_offset.y as u32 + dimensions.height > properties.dimensions.height ||
                image_offset.z as u32 + dimensions.depth > properties.dimensions.depth
            {
                return Err(ImageError::InvalidCopy {
                    image_dimensions: properties.dimensions,
                    copy_offset: image_offset,
                    copy_dimensions: dimensions,
                }.into())
            }
            image_extent = dimensions;
        }
        let device = &self.device;
        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: data.len() as vk::DeviceSize,
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let staging_buffer = unsafe {
            device.create_buffer(&buffer_info, None)?
        };
        let memory = self.linear_device_alloc
            .write()
            .expect("LinearDeviceAlloc lock poisoned")
            .bind_buffer_memory(staging_buffer)?;
        let ptr = unsafe { memory.get_mapped_memory() }.unwrap();

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
            self.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                staging_buffer,
                image.handle(),
                image.layout(),
                &[region],
            );
        };

        self.staging_buffers.push(staging_buffer);

        if dst_state.queue_family_index != state.queue_family_index {
            dst_state.queue_family_index = state.queue_family_index;
            image.cmd_memory_barrier(dst_state, self.command_buffer, None).unwrap();
        }

        Ok(())
    }
}

impl Drop for TransferCommands {

    fn drop(&mut self) {
        unsafe {
            for buffer in self.staging_buffers.iter() {
                self.device.destroy_buffer(*buffer, None);
            }
            self.staging_buffers.clear();
            if let Some(fence) = self.fence.take() {
                self.device.destroy_fence(fence, None);
            }
            self.device.free_command_buffers(self.command_pool, &[self.command_buffer]);
        }
    }
}
