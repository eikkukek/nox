use std::sync::Arc;
use core::{
    ptr::NonNull,
    slice,
};

use nox_ash::vk;

use crate::dev::has_bits;

use crate::gpu::{
    *,
};

use super::*;

type Result<T> = core::result::Result<T, MemoryBinderError>;

use MemoryBinderError::*;

#[derive(Clone)]
pub struct DefaultBinder {
    vk: Arc<Vulkan>,
    memory_type_bits: u32,
    mappable: bool,
}

impl DefaultBinder {

    pub(crate) fn new(
        vk: Arc<Vulkan>,
        required_properties: vk::MemoryPropertyFlags,
        forbidden_properties: vk::MemoryPropertyFlags,
    ) -> Self
    {
        let memory_properties = vk.physical_device_info().memory_properties();
        let mut memory_type_bits = 0;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize]
            .iter()
            .enumerate()
        {
            let property_flags = memory_type.property_flags;
            if has_bits!(property_flags, required_properties) && !property_flags.intersects(forbidden_properties) {
                memory_type_bits |= 1 << i;
            }
        }
        Self {
            vk,
            memory_type_bits,
            mappable: has_bits!(required_properties, vk::MemoryPropertyFlags::HOST_VISIBLE),
        }
    }
}

pub struct Memory {
    vk: Arc<Vulkan>,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    map: Option<NonNull<u8>>,
    mappable: bool,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl DeviceMemory for Memory {

    fn device_memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    fn offset(&self) -> vk::DeviceSize {
        0
    }
    
    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    fn map_memory(&mut self) -> Result<&mut [u8]> {
        if !self.mappable {
            return Err(UnmappableMemory)
        }
        if let Some(map) = self.map {
            unsafe {
                return Ok(slice::from_raw_parts_mut(
                    map.as_ptr(),
                    self.size as usize,
                ))
            }
        }
        let ptr = unsafe {
            self.vk.device().map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::from_raw(0))?
        };
        let map = *self.map.insert(NonNull::new(ptr as *mut u8).unwrap());
        unsafe {
            Ok(slice::from_raw_parts_mut(
                map.as_ptr(),
                self.size as usize,
            ))
        }
    }
}

impl Drop for Memory {

    fn drop(&mut self) {
        unsafe {
            self.vk.device()
                .free_memory(self.memory, None);
        }
    }
}

impl MemoryBinder for DefaultBinder {

    #[inline(always)]
    fn max_alloc_size(&self) -> vk::DeviceSize {
        self.vk.max_memory_allocation_size()
    }

    #[inline(always)]
    fn is_mappable(&self) -> bool {
        self.mappable
    }

    #[inline(always)]
    unsafe fn release_resources(&mut self) {}

    unsafe fn alloc(
        &mut self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> memory_binder::Result<Box<dyn DeviceMemory>> {
        let memory_type_bits = self.memory_type_bits & memory_requirements.memory_requirements.memory_type_bits;
        if memory_type_bits == 0 {
            return Err(IncompatibleMemoryRequirements)
        }
        let memory_type_index = memory_type_bits.trailing_zeros();
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe {
            self.vk.device().allocate_memory(&allocate_info, None)?
        };
        Ok(Box::new(Memory {
            vk: self.vk.clone(),
            memory,
            size: memory_requirements.memory_requirements.size,
            map: None,
            mappable: self.mappable,
        }))
    } 
}
