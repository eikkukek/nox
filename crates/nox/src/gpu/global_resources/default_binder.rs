use std::{ptr::NonNull, sync::Arc};

use ash::vk;

use crate::dev::has_bits;

use crate::gpu::{
    memory_binder::*,
    *,
};

type Result<T> = core::result::Result<T, MemoryBinderError>;

use MemoryBinderError::*;

#[derive(Clone)]
pub struct DefaultBinder {
    device: Arc<ash::Device>,
    memory_type_bits: u32,
    mappable: bool,
}

impl DefaultBinder {

    pub fn new(
        device: Arc<ash::Device>,
        required_properties: vk::MemoryPropertyFlags,
        forbidden_properties: vk::MemoryPropertyFlags,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Self
    {
        let memory_properties = physical_device_info.memory_properties();
        let mut memory_type_bits = 0;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize].iter().enumerate() {
            let property_flags = memory_type.property_flags;
            if has_bits!(property_flags, required_properties) && !property_flags.intersects(forbidden_properties) {
                memory_type_bits |= 1 << i;
            }
        }
        Self {
            device,
            memory_type_bits,
            mappable: has_bits!(required_properties, vk::MemoryPropertyFlags::HOST_VISIBLE),
        }
    }
}

pub struct Memory {
    device: Arc<ash::Device>,
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

    unsafe fn free_memory(&self) {
        unsafe {
            self.device.free_memory(self.memory, None);
        }
    }

    unsafe fn map_memory(&mut self) -> Result<core::ptr::NonNull<u8>> {
        if !self.mappable {
            return Err(UnmappableMemory)
        }
        if let Some(map) = self.map {
            return Ok(map)
        }
        let ptr = unsafe {
            self.device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::from_raw(0))?
        };
        Ok(*self.map.insert(NonNull::new(ptr as *mut u8).unwrap()))
    }
}

impl MemoryBinder for DefaultBinder {

    fn bind_image_memory(
        &mut self,
        image: vk::Image,
        _: Option<&mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>>>
    ) -> Result<Box<dyn DeviceMemory>>
    {
        let device = &self.device;
        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        let memory_type_bits = self.memory_type_bits & memory_requirements.memory_type_bits;
        if memory_type_bits == 0 {
            return Err(IncompatibleMemoryRequirements)
        }
        let memory_type_index = memory_type_bits.trailing_zeros();
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe {
            self.device.allocate_memory(&allocate_info, None)?
        };
        unsafe {
            device.bind_image_memory(image, memory, 0)?;
        }
        Ok(Box::new(Memory {
            device: self.device.clone(),
            memory,
            size: memory_requirements.size,
            map: None,
            mappable: self.mappable,
        }))
    }

    fn bind_buffer_memory(
        &mut self,
        buffer: vk::Buffer,
        _: Option<&mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>>>
    ) -> Result<Box<dyn DeviceMemory>> {
        let device = &self.device;
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type_bits = self.memory_type_bits & memory_requirements.memory_type_bits;
        if memory_type_bits == 0 {
            return Err(IncompatibleMemoryRequirements)
        }
        let memory_type_index = memory_type_bits.trailing_zeros();
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe {
            self.device.allocate_memory(&allocate_info, None)?
        };
        unsafe {
            device.bind_buffer_memory(buffer, memory, 0)?;
        }
        Ok(Box::new(Memory {
            device: self.device.clone(),
            memory,
            size: memory_requirements.size,
            map: None,
            mappable: self.mappable,
        }))
    }
}
