use core::cell::UnsafeCell;

use ash::vk;

use crate::{has_bits, has_not_bits};

use super::PhysicalDeviceInfo;

#[derive(Debug)]
pub enum Error {
    IncompatibleMemoryRequirements,
    OutOfMemory { size: vk::DeviceSize, align: vk::DeviceSize },
    VkError(vk::Result),
}

impl From<vk::Result> for Error {

    fn from(value: vk::Result) -> Self {
        VkError(value)
    }
}

use Error::{IncompatibleMemoryRequirements, OutOfMemory, VkError};

pub type Result<T> = core::result::Result<T, Error>;

pub struct LinearDeviceAlloc {
    device: ash::Device,
    device_memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    used: UnsafeCell<vk::DeviceSize>,
    granularity: vk::DeviceSize,
    properties: vk::MemoryPropertyFlags,
}

impl LinearDeviceAlloc {

    pub fn new(
        device: ash::Device,
        size: vk::DeviceSize,
        properties: vk::MemoryPropertyFlags,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> Result<Self>
    {
        let memory_properties = physical_device_info.memory_properties();
        let mut maybe_index = None;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize].iter().enumerate() {
            if has_bits!(memory_type.property_flags, properties) {
                maybe_index = Some(i as u32);
                break;
            }
        }
        let memory_type_index = maybe_index.ok_or_else(|| IncompatibleMemoryRequirements)?;
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };
        let device_memory = unsafe {
            device.allocate_memory(&allocate_info, None)?
        };
        Ok(Self {
            device,
            device_memory,
            size,
            used: UnsafeCell::new(0),
            granularity: physical_device_info.properties().limits.buffer_image_granularity,
            properties,
        })
    }

    pub fn device(&self) -> ash::Device {
        self.device.clone()
    }

    pub fn bind_image_memory(&self, image: vk::Image) -> Result<()> {
        let memory_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        if has_not_bits!(self.properties.as_raw(), memory_requirements.memory_type_bits) {
            return Err(IncompatibleMemoryRequirements)
        }
        let used = unsafe{ *self.used.get() };
        let align = memory_requirements.alignment.max(self.granularity);
        let offset = (used + align - 1) & !(align - 1);
        let end = offset + memory_requirements.size;
        if self.size < end {
            return Err(OutOfMemory { size: memory_requirements.size, align } )
        }
        unsafe {
            self.device.bind_image_memory(image, self.device_memory, offset)?;
            *self.used.get() = end;
        };
        Ok(())
    }
}

impl Drop for LinearDeviceAlloc {

    fn drop(&mut self) {
        unsafe {
            self.device.free_memory(
                self.device_memory,
                None
            );
        }
    }
}
