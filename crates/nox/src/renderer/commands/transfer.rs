use std::{
    sync::{Arc, RwLock},
};

use core::ptr;

use ash::vk;

use nox_mem::{Allocator, vec_types::{FixedVec, GlobalVec, Vector}};

use super::*;

use crate::{
    has_not_bits,
    renderer::{helpers, memory_binder::MemoryBinder},
    *,
};

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
    ) -> Result<Self, Error> {
        let handle = helpers::create_command_pool(
            &device,
            vk::CommandPoolCreateFlags::TRANSIENT,
            queue_family_index,
        )?;
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

pub struct TransferCommands {
    transfer_command_pool: Arc<TransientCommandPool>,
    transfer_command_buffer: vk::CommandBuffer,
    graphics_command_pool: Arc<TransientCommandPool>,
    graphics_command_buffer: vk::CommandBuffer,
    global_resources: Arc<RwLock<GlobalResources>>,
    staging_buffers: GlobalVec<vk::Buffer>,
    linear_device_alloc: LinearDeviceAllocLock,
    transfer_fence: Option<vk::Fence>,
    graphics_fence: Option<vk::Fence>,
    id: CommandRequestId,
}

pub struct TransferCommandsNested<'a> {
    transfer_commands: &'a mut TransferCommands,
}

impl TransferCommands {

    #[inline(always)]
    pub(crate) fn new(
        transfer_command_pool: Arc<TransientCommandPool>,
        transfer_command_buffer: vk::CommandBuffer,
        graphics_command_pool: Arc<TransientCommandPool>,
        graphics_command_buffer: vk::CommandBuffer,
        global_resources: Arc<RwLock<GlobalResources>>,
        linear_device_alloc: LinearDeviceAllocLock,
        id: CommandRequestId,
    ) -> Result<Self, Error>
    {
        Ok(Self {
            transfer_command_pool,
            transfer_command_buffer,
            graphics_command_pool,
            graphics_command_buffer,
            global_resources,
            staging_buffers: GlobalVec::new(),
            linear_device_alloc,
            transfer_fence: None,
            graphics_fence: None,
            id,
        })
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
    pub(crate) fn get_transfer_fence(&mut self) -> Result<(bool, vk::Fence), vk::Result> {
        let mut new = false;
        if self.transfer_fence.is_none() {
            let info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            self.transfer_fence = Some(unsafe {
                self.transfer_command_pool.device().create_fence(&info, None)?
            });
            new = true;
        }
        Ok((new, self.transfer_fence.unwrap()))
    }

    #[inline(always)]
    pub(crate) fn get_graphics_fence(&mut self) -> Result<(bool, vk::Fence), vk::Result> {
        let mut new = false;
        if self.graphics_fence.is_none() {
            let info = vk::FenceCreateInfo {
                s_type: vk::StructureType::FENCE_CREATE_INFO,
                ..Default::default()
            };
            self.graphics_fence = Some(unsafe {
                self.graphics_command_pool.device().create_fence(&info, None)?
            });
            new = true;
        }
        Ok((new, self.graphics_fence.unwrap()))
    }

    #[inline(always)]
    pub fn edit_resources(
        &mut self,
        mut f: impl FnMut(TransferCommandsNested, &mut GlobalResources) -> Result<(), Error>
    ) -> Result<(), Error> {
        let g = self.global_resources.clone();
        f(TransferCommandsNested { transfer_commands: self }, &mut g.write().unwrap())
    }

    #[inline(always)]
    pub fn reserve_staging_buffers(&mut self, capacity: usize) {
        self.staging_buffers.reserve(capacity);
    }

    #[inline(always)]
    pub fn clear_color_image(
        &self,
        image_id: ImageId,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator
    ) -> Result<(), Error> {
        self.clear_color_image_internal(&mut self.global_resources.write().unwrap(), image_id, clear_value, subresources, alloc)
    }

    #[inline(always)]
    pub fn clear_depth_stencil_image(
        &mut self,
        image_id: ImageId,
        depth: f32,
        stencil: u32,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator,
    ) -> Result<(), Error> {
        let g = self.global_resources.clone();
        self.clear_depth_stencil_image_internal(
            &mut g.write().unwrap(),
            image_id, depth, stencil, subresources, alloc
        )
    }

    #[inline(always)]
    pub fn copy_data_to_buffer(
        &mut self,
        buffer_id: BufferId, 
        data: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<(), Error> {
        let g = self.global_resources.clone();
        self.copy_data_to_buffer_internal(&mut g.write().unwrap(), buffer_id, data, offset, size)
    }

    #[inline(always)]
    pub fn copy_data_to_image(
        &mut self,
        image_id: ImageId,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset3D>,
        dimensions: Option<Dimensions>,
    ) -> Result<(), Error> {
        let g = self.global_resources.clone();
        self.copy_data_to_image_internal(&mut g.write().unwrap(), image_id, data, layers, offset, dimensions)
    }

    #[inline(always)]
    pub fn gen_mip_maps(
        &self,
        image: ImageId,
        filter: Filter,
    ) -> Result<(), Error> {
        self.gen_mip_maps_internal(&mut self.global_resources.write().unwrap(), image, filter)
    }

    #[inline(always)]
    fn clear_color_image_internal(
        &self,
        g: &mut GlobalResources,
        image_id: ImageId,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        let image = g.get_image(image_id)?;
        if has_not_bits!(image.properties.usage, vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(ImageError::UsageMismatch {
                missing_usage: vk::ImageUsageFlags::TRANSFER_DST,
            }.into())
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
            ImageState::new(
                vk::AccessFlags::TRANSFER_WRITE,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                transfer_queue_index,
                vk::PipelineStageFlags::TRANSFER
            ),
            command_buffer,
            None,
            false,
        )?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            alloc
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
            self.transfer_command_pool.device().cmd_clear_color_image(
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
    fn clear_depth_stencil_image_internal(
        &mut self,
        g: &mut GlobalResources,
        image_id: ImageId,
        depth: f32,
        stencil: u32,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
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
            self.transfer_command_pool.queue_family_index(),
            vk::PipelineStageFlags::TRANSFER,
        );
        let command_buffer = self.transfer_command_buffer;
        image.cmd_memory_barrier(
            dst_state,
            command_buffer,
            None,
            true,
        )?;
        let mut ranges = FixedVec::with_capacity(
            subresources.map(|v| v.len()).unwrap_or(1),
            alloc,
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
            self.transfer_command_pool.device().cmd_clear_depth_stencil_image(
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
    fn copy_data_to_buffer_internal(
        &mut self,
        g: &mut GlobalResources,
        buffer_id: BufferId, 
        data: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<(), Error>
    {
        let mut default_binder = g.default_memory_binder_mappable().clone();
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
        let device = &self.transfer_command_pool.device();
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
        let mut memory = self.linear_device_alloc
            .bind_buffer_memory(
                staging_buffer,
                Some(&mut |buffer| {
                    default_binder.bind_buffer_memory(buffer, None)
                })
            )?;
        let ptr = unsafe { memory.map_memory() };
        let Some(ptr) = ptr else {
            return Err(Error::NonMappableMemory)
        };

        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: offset,
            size,
        };

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.as_ptr(), data.len());
            device.cmd_copy_buffer(
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
    fn copy_data_to_image_internal(
        &mut self,
        g: &mut GlobalResources,
        image_id: ImageId,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset3D>,
        dimensions: Option<Dimensions>,
    ) -> Result<(), Error>
    {
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
        let device = &self.transfer_command_pool.device();
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
        let mut memory = self.linear_device_alloc
            .bind_buffer_memory(staging_buffer, Some(&mut |buffer| {
                g.default_memory_binder_mappable().bind_buffer_memory(buffer, None)
            }))?;
        let ptr = unsafe { memory.map_memory() };
        let Some(ptr) = ptr else {
            return Err(Error::NonMappableMemory)
        };

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
            device.cmd_copy_buffer_to_image(
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
    fn gen_mip_maps_internal(
        &self,
        g: &mut GlobalResources,
        image: ImageId,
        filter: Filter,
    ) -> Result<(), Error>
    {
        let filter = filter.into();
        let image = g.get_image(image)?;
        if !image.properties.usage.contains(vk::ImageUsageFlags::TRANSFER_SRC) {
            return Err(ImageError::UsageMismatch { missing_usage: vk::ImageUsageFlags::TRANSFER_SRC }.into())
        }
        if !image.properties.usage.contains(vk::ImageUsageFlags::TRANSFER_DST) {
            return Err(ImageError::UsageMismatch { missing_usage: vk::ImageUsageFlags::TRANSFER_DST }.into())
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
        )?;
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
            )?;
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
            )?;
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
        )?;
        Ok(())
    }
}

impl<'a> TransferCommandsNested<'a> {

    #[inline(always)]
    pub fn reserve_staging_buffers(&mut self, capacity: usize) {
        self.transfer_commands.staging_buffers.reserve(capacity);
    }

    #[inline(always)]
    pub fn clear_color_image(
        &self,
        g: &mut GlobalResources,
        image_id: ImageId,
        clear_value: ClearColorValue,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator
    ) -> Result<(), Error> {
        self.transfer_commands.clear_color_image_internal(g, image_id, clear_value, subresources, alloc)
    }

    #[inline(always)]
    pub fn clear_depth_stencil_image(
        &mut self,
        g: &mut GlobalResources,
        image_id: ImageId,
        depth: f32,
        stencil: u32,
        subresources: Option<&[ImageSubresourceRangeInfo]>,
        alloc: &impl Allocator,
    ) -> Result<(), Error> {
        self.transfer_commands.clear_depth_stencil_image_internal(
            g, image_id, depth, stencil, subresources, alloc
        )
    }

    #[inline(always)]
    pub fn copy_data_to_buffer(
        &mut self,
        g: &mut GlobalResources,
        buffer_id: BufferId, 
        data: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<(), Error> {
        self.transfer_commands.copy_data_to_buffer_internal(g, buffer_id, data, offset, size)
    }

    #[inline(always)]
    pub fn copy_data_to_image(
        &mut self,
        g: &mut GlobalResources,
        image_id: ImageId,
        data: &[u8],
        layers: Option<ImageSubresourceLayers>,
        offset: Option<Offset3D>,
        dimensions: Option<Dimensions>,
    ) -> Result<(), Error> {
        self.transfer_commands.copy_data_to_image_internal(g, image_id, data, layers, offset, dimensions)
    }

    #[inline(always)]
    pub fn gen_mip_maps(
        &self,
        g: &mut GlobalResources,
        image: ImageId,
        filter: Filter,
    ) -> Result<(), Error> {
        self.transfer_commands.gen_mip_maps_internal(g, image, filter)
    }
}

impl Drop for TransferCommands {

    fn drop(&mut self) {
        unsafe {
            let device = &self.transfer_command_pool.device();
            for buffer in self.staging_buffers.iter() {
                device.destroy_buffer(*buffer, None);
            }
            self.staging_buffers.clear();
            if let Some(fence) = self.transfer_fence.take() {
                device.destroy_fence(fence, None);
            }
            if let Some(fence) = self.graphics_fence.take() {
                device.destroy_fence(fence, None);
            }
        }
    }
}
