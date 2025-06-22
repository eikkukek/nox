use core::cell::RefCell;

use ash::vk;

use crate::{
    vec_types::{Dyn, Vector},
    string_types::{array_format, LargeError},
};

use super::{
    MemoryLayout,
    Handle,
    buffer_allocator::{BufferAlloc, Block},
    physical_device::PhysicalDeviceInfo,
};

pub struct DeviceAllocators<'mem, DynVec: Vector<Block, CapacityPol = Dyn>> {
    _device_local: RefCell<BufferAlloc<'mem, DynVec>>,
    _device_staging: RefCell<BufferAlloc<'mem, DynVec>>,
    _device_uniform: RefCell<BufferAlloc<'mem, DynVec>>,
}

impl<'p, DynVec> DeviceAllocators<'p, DynVec>
    where
        DynVec: Vector<Block, CapacityPol = Dyn>
{

    pub fn new(
        layout: &MemoryLayout,
        device: ash::Device,
        physical_device_info: &PhysicalDeviceInfo,
        device_local_free_list: DynVec,
        device_staging_free_list: DynVec,
        device_uniform_size: DynVec,
    ) -> Result<Self, LargeError>
    {
        let device_local = BufferAlloc::new(
            Handle::new(device.clone()),
            physical_device_info,
            layout.device_local_size(),
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_local_free_list,
            ).map_err(|e| array_format!("failed to create allocator ( {} )", e)
        )?;
        let device_staging = BufferAlloc::new(
            Handle::new(device.clone()),
            physical_device_info,
            layout.device_staging_size(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_VISIBLE,
            device_staging_free_list,
            ).map_err(|e| array_format!("failed to create allocator ( {} )", e)
        )?;
        let device_uniform = BufferAlloc::new(
            Handle::new(device.clone()),
            physical_device_info,
            layout.device_uniform_size(),
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_VISIBLE,
            device_uniform_size,
            ).map_err(|e| array_format!("failed to create allocator ( {} )", e)
        )?;
        Ok(Self {
            _device_local: RefCell::new(device_local),
            _device_staging: RefCell::new(device_staging),
            _device_uniform: RefCell::new(device_uniform),
        })
    }
}

