use crate::vec_types::VecOperations;

use ash::vk;

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub fn allocate_command_buffers<V>(
    device: &ash::Device,
    info: &vk::CommandBufferAllocateInfo,
    out: &mut V,
) -> Result<(), vk::Result>
    where
        V: VecOperations<vk::CommandBuffer>,
{
    if info.command_buffer_count > out.len() as u32 {
        panic!("out len smaller than command buffer count!");
    }
    let allocate_command_buffers = device.fp_v1_0().allocate_command_buffers;
    let result = unsafe {
        allocate_command_buffers(device.handle(), info, out.as_mut_ptr() as _)
    };
    if result != vk::Result::SUCCESS {
        Err(result)
    }
    else {
        Ok(())
    }
}

pub fn create_command_pool(
    device: &ash::Device,
    flags: vk::CommandPoolCreateFlags,
    queue_family_index: u32,
) -> Result<vk::CommandPool, vk::Result> {
    let create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        flags,
        queue_family_index,
        ..Default::default()
    };
    unsafe {
        device.create_command_pool(&create_info, None)
    }
}

pub fn begin_command_buffer(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
) -> Result<(), vk::Result> {
    let begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        ..Default::default()
    };
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }
}

#[derive(Clone)]
pub struct Handle<'a, T> {
    handle: T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Handle<'a, T> {

    pub fn new(h: T) -> Self {
        Self {
            handle: h,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Deref for Handle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<'a, T> DerefMut for Handle<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
