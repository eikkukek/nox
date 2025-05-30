use super::{
    physical_device::PhysicalDeviceInfo,
    helpers::Handle,
};

use crate::{
    string::{SmallError, String},
    utility::{has_bit, has_not_bit},
    vec_types::{VecOperations, FixedVec},
    allocator_traits::AllocateExt,
};

use ash::vk;

use core::marker::PhantomData;

pub struct GpuMemory<'r> {
    memory: Handle<'r, vk::DeviceMemory>,
    size: vk::DeviceSize,
    device: Handle<'r, ash::Device>,
    properties: vk::MemoryPropertyFlags,
}

impl<'r> GpuMemory<'r> {

    pub fn new(
        device: Handle<'r, ash::Device>,
        physical_device_info: &PhysicalDeviceInfo,
        size: vk::DeviceSize,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, SmallError> {
        let memory_properties = physical_device_info.memory_properties();
        let mut maybe_index = None;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize].iter().enumerate() {
            if has_bit!(memory_type.property_flags, properties) {
                maybe_index = Some(i as u32);
                break;
            }
        }
        let memory_type_index = maybe_index
            .ok_or_else(||
                String::from_str("could not find requested memory properties"
            ))?;
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };
        let memory = Handle::new(unsafe {
            device
            .allocate_memory(&allocate_info, None)
            .map_err(|e| {
                String::format(format_args!(
                    "failed to allocate memory {:?}", e
                ))
            })?
        });
        Ok(Self {
            memory,
            size,
            device,
            properties,
        })
    }
}

struct Block {
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
}

impl Block {

    fn new(offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
        Self {
            offset,
            size,
        }
    }
}

pub struct Allocation<'r, 'gpu> {
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
    _marker: PhantomData<&'gpu GpuMemory<'r>>,
}

impl<'r, 'gpu> Allocation<'r, 'gpu> {

    fn new(offset: vk::DeviceSize, size: vk::DeviceSize, marker: PhantomData<&'gpu GpuMemory<'r>>) -> Self {
        Self {
            offset,
            size,
            _marker: marker,
        }
    }
}

pub struct BufferAllocator<'r, 'gpu, 'cpu> {
    memory: Handle<'gpu, vk::DeviceMemory>,
    size: vk::DeviceSize,
    device: Handle<'r, ash::Device>,
    free_list: FixedVec<'cpu, Block>,
    properties: vk::MemoryPropertyFlags,
}

impl<'r, 'gpu, 'cpu> BufferAllocator<'r, 'gpu, 'cpu>
    where
        'r: 'gpu
{

    pub fn new<A: AllocateExt<'cpu>>(
        memory: &mut GpuMemory,
        free_list_size: usize,
        cpu_allocator: &mut A,
    ) -> Result<Self, SmallError> {
        let mut free_list = FixedVec
            ::new(free_list_size, cpu_allocator)
            .ok_or_else(||
                String::from_str("failed to allocate free list"
            ))?;
        free_list.push_back(Block::new(0, memory.size));
        Ok(Self {
            memory: Handle::new(*memory.memory),
            size: memory.size,
            device: Handle::new((*memory.device).clone()),
            free_list,
            properties: memory.properties,
        })
    }

    fn allocate(
        &mut self,
        size: vk::DeviceSize,
        align: vk::DeviceSize
    ) -> Option<Allocation<'r, 'gpu>> {
        for i in 0..self.free_list.len() {
            let block = &mut self.free_list[i];
            let aligned_offset = (block.offset + align - 1) & !(align - 1);
            let padding = aligned_offset - block.offset;
            let total = padding + size;
            if total <= block.size {
                let remaining = block.size - total;
                if remaining > 0 {
                    block.offset = aligned_offset + size;
                    block.size = remaining;
                }
                else {
                    self.free_list.remove(i);
                }
                return Some(Allocation::new(aligned_offset, size, PhantomData))
            }
        }
        None
    }

    pub fn free(&mut self, allocation: Allocation) {
        let end = allocation.offset + allocation.size;
        let mut i = 0;
        while i < self.free_list.len() {
            let block = &mut self.free_list[i];
            if end < block.offset {
                break;
            }
            let block_end = block.offset + block.size;
            if allocation.offset < block_end && end > block.offset {
                panic!("invalid free")
            }
            i += 1;
        }
        let mut new_offset = allocation.offset;
        let mut new_size = allocation.size;
        // try merge previous
        if i > 0 {
            let prev = &self.free_list[i - 1];
            let prev_end = prev.offset + prev.size;
            if prev_end == allocation.offset {
                new_offset = prev.offset;
                new_size += prev.size;
                self.free_list.remove(i - 1);
                i -= 1;
            }
        }
        // try merge next
        if i < self.free_list.len() {
            let next = &self.free_list[i];
            if next.offset == end {
                new_size += next.size;
                self.free_list.remove(i);
            }
        }
        self.free_list.insert(Block::new(new_offset, new_size), i);
    }

    pub fn bind_image_memory(&mut self, image: Handle<'r, vk::Image>) -> Result<Allocation, SmallError> {
        unsafe {
            let memory_requirements = self.device.get_image_memory_requirements(*image);
            if has_not_bit!(self.properties.as_raw(), memory_requirements.memory_type_bits) {
                return Err(String::from_str("incompatible memory properties"))
            }
            let allocation = self
                .allocate(memory_requirements.size, memory_requirements.alignment)
                .ok_or_else(|| String::from_str("out of CPU memory"))?;
            self.device
                .bind_image_memory(*image, *self.memory, allocation.offset)
                .map_err(|e| String::format(format_args!(
                    "failed to bind memory {:?}", e
                )))?;
            Ok(allocation)
        }
    }

    pub fn bind_buffer_memory(&mut self, buffer: Handle<'r, vk::Buffer>) -> Result<(), SmallError> {
        unsafe {
            let memory_requirements = self.device.get_buffer_memory_requirements(*buffer);
            if has_not_bit!(self.properties.as_raw(), memory_requirements.memory_type_bits) {
                return Err(String::from_str("incompatible memory properties"))
            }
            let allocation = self
                .allocate(memory_requirements.size, memory_requirements.alignment)
                .ok_or_else(|| String::from_str("out of CPU memory"))?;
            self.device
                .bind_buffer_memory(*buffer, *self.memory, allocation.offset)
                .map_err(|e| String::format(format_args!(
                    "failed to bind memory {:?}", e
                )))?;
        }
        Ok(())
    }
}
