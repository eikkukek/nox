use std::{
    sync::{Arc, RwLock},
};

use core::ptr;

use ash::vk;

use super::*;

use nox_mem::{Vector, vec_types::GlobalVec};

use crate::{
    has_not_bits,
    renderer::{
        image::{Dimensions, ImageError, ImageSubresourceLayers, Offset},
        linear_device_alloc::LinearDeviceAlloc,
        memory_binder::{DeviceMemory, MemoryBinder},
        Error,
        GlobalResources,
        ImageID,
    }
};

pub struct TransferCommandbuffer {
    device: Arc<ash::Device>,
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    global_resources: Arc<RwLock<GlobalResources>>,
    staging_buffers: GlobalVec<vk::Buffer>,
    linear_device_alloc: Arc<RwLock<LinearDeviceAlloc>>,
    fence: Option<vk::Fence>,
    id: CommandRequestID,
}

impl TransferCommandbuffer {

    pub(crate) fn new(
        device: Arc<ash::Device>,
        command_buffer: vk::CommandBuffer,
        command_pool: vk::CommandPool,
        global_resources: Arc<RwLock<GlobalResources>>,
        linear_device_alloc: Arc<RwLock<LinearDeviceAlloc>>,
        staging_buffer_capacity: u32,
        id: CommandRequestID,
    ) -> Result<Self, Error>
    {
        Ok(Self {
            device: device.clone(),
            command_buffer,
            command_pool,
            global_resources,
            staging_buffers: GlobalVec::with_capacity(staging_buffer_capacity as usize)?,
            linear_device_alloc,
            fence: None,
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
    pub fn copy_data_to_image(
        &mut self,
        image_id: ImageID,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset>,
        dimensions: Option<Dimensions>,
    ) -> Result<(), Error>
    {
        let g = self.global_resources.write().unwrap();
        let image = g.get_image(image_id);
        let properties = image.properties();
        if has_not_bits!(properties.usage, vk::ImageUsageFlags::TRANSFER_SRC) {
            return Err(ImageError::UsageMismatch {
                missing_usage: vk::ImageUsageFlags::TRANSFER_SRC
            }.into())
        }
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
            usage: vk::BufferUsageFlags::TRANSFER_DST,
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
        let ptr = unsafe {
            self.device.map_memory(
                memory.device_memory(),
                memory.offset(),
                memory.size(),
                Default::default(),
            )?
        } as *mut u8;

        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: subresource_layers.into(),
            image_offset: image_offset.into(),
            image_extent: image_extent.into(),
        };

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            self.device.cmd_copy_buffer_to_image(
                self.command_buffer,
                staging_buffer,
                image.handle(),
                image.layout(),
                &[region],
            );
        };

        self.staging_buffers
            .push(staging_buffer)
            .unwrap();
        
        Ok(())
    }
}

impl Drop for TransferCommandbuffer {

    fn drop(&mut self) {
        unsafe {
            for buffer in self.staging_buffers.iter() {
                self.device.destroy_buffer(*buffer, None);
            }
            self.staging_buffers.clear();
            if let Some(fence) = self.fence {
                self.device.destroy_fence(fence, None);
            }
            self.device.free_command_buffers(self.command_pool, &[self.command_buffer]);
        }
    }
}
