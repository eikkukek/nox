use std::sync::Arc;

use core::ptr::NonNull;

use ash::vk;

use nox_mem::{vec_types::{GlobalVec, Vector}};

use crate::{
        has_bits,
        memory_binder::DeviceMemory,
        RendererContext
};

use super::{
    PhysicalDeviceInfo,
    memory_binder::MemoryBinder,
    Error::{self, OutOfDeviceMemory, IncompatibleMemoryRequirements}
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Copy)]
struct Block {
    device_memory: Option<vk::DeviceMemory>,
    mapped_pointer: Option<NonNull<u8>>,
    used: vk::DeviceSize,
}

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

impl Block {

    #[inline(always)]
    fn new() -> Self {
        Self {
            device_memory: None,
            mapped_pointer: None,
            used: 0,
        }
    }

    #[inline(always)]
    fn bind_image_memory(
        &mut self,
        device: &ash::Device,
        image: vk::Image,
        memory_requirements: vk::MemoryRequirements,
        block_size: vk::DeviceSize,
        memory_type_index: u32,
        granularity: vk::DeviceSize,
    ) -> Result<Memory>
    {
        if self.device_memory.is_none() {
            let allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                allocation_size: block_size,
                memory_type_index,
                ..Default::default()
            };
            self.device_memory = Some(unsafe {
                device.allocate_memory(&allocate_info, None)?
            });
        }
        let device_memory = self.device_memory.unwrap();
        let used = self.used;
        let align = memory_requirements.alignment.max(granularity);
        let offset = (used + align - 1) & !(align - 1);
        let end = offset + memory_requirements.size;
        if block_size < end {
            return Err(OutOfDeviceMemory { size: memory_requirements.size, align, avail: block_size - used } )
        }
        unsafe {
            device.bind_image_memory(image, device_memory, offset)?;
        };
        self.used = end;
        Ok(Memory::new(device_memory, None, offset, memory_requirements.size))
    }

    #[inline(always)]
    fn bind_buffer_memory(
        &mut self,
        device: &ash::Device,
        buffer: vk::Buffer,
        memory_requirements: vk::MemoryRequirements,
        block_size: vk::DeviceSize,
        memory_type_index: u32,
        granularity: vk::DeviceSize,
        map: bool,
    ) -> Result<Memory>
    {
        if self.device_memory.is_none() {
            let allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                allocation_size: block_size,
                memory_type_index: memory_type_index,
                ..Default::default()
            };
            self.device_memory = Some(unsafe {
                device.allocate_memory(&allocate_info, None)?
            });
        }
        let device_memory = self.device_memory.unwrap();
        if map && self.mapped_pointer.is_none() {
            let ptr = unsafe {
                device.map_memory(device_memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::from_raw(0))?
            };
            self.mapped_pointer = NonNull::new(ptr as *mut u8);
        }
        let used = self.used;
        let align = memory_requirements.alignment.max(granularity);
        let offset = (used + align - 1) & !(align - 1);
        let end = offset + memory_requirements.size;
        if block_size < end {
            return Err(OutOfDeviceMemory { size: memory_requirements.size, align, avail: block_size - used } )
        }
        unsafe {
            device.bind_buffer_memory(buffer, device_memory, offset)?;
        };
        self.used = end;
        Ok(Memory::new(device_memory, self.mapped_pointer, offset, memory_requirements.size))
    }

    #[inline(always)]
    unsafe fn reset(&mut self) {
        self.used = 0;
    }

    #[inline(always)]
    unsafe fn free_memory(&mut self, device: &ash::Device) {
        if let Some(memory) = self.device_memory.take() {
            unsafe {
                device.free_memory(memory, None);
            }
        }
        self.used = 0;
    }
}

pub struct LinearDeviceAlloc {
    device: Arc<ash::Device>,
    blocks: GlobalVec<(GlobalVec<Block>, u32, usize)>,
    block_size: vk::DeviceSize,
    granularity: vk::DeviceSize,
    map_memory: bool,
}

impl LinearDeviceAlloc {

    pub(crate) fn new(
        device: Arc<ash::Device>,
        block_size: vk::DeviceSize,
        required_properties: vk::MemoryPropertyFlags,
        forbidden_properties: vk::MemoryPropertyFlags,
        physical_device_info: &PhysicalDeviceInfo,
        map_memory: bool,
    ) -> Result<Self>
    {
        let memory_properties = physical_device_info.memory_properties();
        let mut blocks = GlobalVec::with_capacity(4);
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize].iter().enumerate() {
            let property_flags = memory_type.property_flags;
            if has_bits!(property_flags, required_properties) && !property_flags.intersects(forbidden_properties) {
                blocks.push((GlobalVec::with_len(1, Block::new()), i as u32, 0));
            }
        }
        Ok(Self {
            device,
            blocks,
            block_size,
            granularity: physical_device_info.properties().limits.buffer_image_granularity,
           map_memory,
        })
    }

    pub fn default(block_size: u64, context: &RendererContext) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::ZeroSizeAlloc)
        }
        Self::new(
            context.device.clone(),
            block_size,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            &context.physical_device_info,
            false,
        )
    }

    pub fn memory_mappable(block_size: u64, context: &RendererContext) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::ZeroSizeAlloc)
        }
        Self::new(
            context.device.clone(),
            block_size,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::MemoryPropertyFlags::empty(),
            &context.physical_device_info,
            true,
        )
    }

    #[inline(always)]
    pub fn mappable(&self) -> bool {
        self.map_memory
    }

    #[inline(always)]
    pub fn block_size(&self) -> u64 {
        self.block_size
    }

    pub unsafe fn reset(&mut self) {
        unsafe {
            for (blocks, _, i) in self.blocks.iter_mut() {
                *i = 0;
                for block in blocks.iter_mut() {
                    block.reset();
                }
            }
        }
    }

    pub unsafe fn clean_up(&mut self) {
        unsafe {
            for (blocks, _, i) in self.blocks.iter_mut() {
                *i = 0;
                for block in blocks.iter_mut() {
                    block.free_memory(&self.device);
                }
            }
            self.blocks.clear();
        }
    }
}

pub struct Memory {
    memory: vk::DeviceMemory,
    map: Option<NonNull<u8>>,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl Memory {

    pub fn new(
        memory: vk::DeviceMemory,
        map: Option<NonNull<u8>>,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> Self
    {
        Self {
            memory,
            map,
            offset,
            size,
        }
    }

    pub unsafe fn get_mapped_memory(&self) -> Option<NonNull<u8>> {
        unsafe {
            Some(self.map?.add(self.offset as usize))
        }
    }
}

impl DeviceMemory for Memory {

    fn device_memory(&self) -> vk::DeviceMemory {
        self.memory
    }

    fn offset(&self) -> vk::DeviceSize {
        self.offset
    }

    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    unsafe fn free_memory(&self) {}

    unsafe fn map_memory(&mut self) -> Option<NonNull<u8>> {
        unsafe {
            self.get_mapped_memory()
        }
    }
}

impl MemoryBinder for LinearDeviceAlloc {

    fn bind_image_memory(
        &mut self,
        image: vk::Image,
        fall_back: Option<&mut dyn FnMut(vk::Image) -> Result<Box<dyn DeviceMemory>>>,
    ) -> Result<Box<dyn DeviceMemory>>
    {
        let device = &self.device;
        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        let block_size = self.block_size;
        for (blocks, type_index, free_index) in self.blocks.iter_mut() {
            if memory_requirements.memory_type_bits & (1 << *type_index) != 0 {
                if block_size < memory_requirements.size {
                    if let Some(fall_back) = fall_back {
                        return fall_back(image);
                    } else {
                        return Err(Error::OutOfDeviceMemory {
                            size: memory_requirements.size,
                            align: memory_requirements.alignment,
                            avail: block_size,
                        })
                    }
                }
                let mut res = blocks[*free_index]
                    .bind_image_memory(device, image, memory_requirements, self.block_size, *type_index, self.granularity,);
                if let Err(err) = &res {
                    if let Error::OutOfDeviceMemory { size: _, align: _, avail: _ } = err {
                        *free_index += 1;
                        if *free_index == blocks.len() {
                            blocks.push(Block::new());
                        }
                        res = blocks[*free_index].bind_image_memory(
                            device, image, memory_requirements, self.block_size, *type_index, self.granularity,
                        );
                    }
                }
                return Ok(Box::new(res.unwrap()))
            }
        }
        Err(IncompatibleMemoryRequirements)
    }

    fn bind_buffer_memory(
        &mut self,
        buffer: vk::Buffer,
        fall_back: Option<&mut dyn FnMut(vk::Buffer) -> Result<Box<dyn DeviceMemory>>>,
    ) -> Result<Box<dyn DeviceMemory>>
    {
        let device = &self.device;
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let block_size = self.block_size();
        for (blocks, type_index, free_index) in self.blocks.iter_mut() {
            if memory_requirements.memory_type_bits & (1 << *type_index) != 0 {
                if block_size < memory_requirements.size {
                    if let Some(fall_back) = fall_back {
                        return fall_back(buffer);
                    } else {
                        return Err(Error::OutOfDeviceMemory {
                            size: memory_requirements.size,
                            align: memory_requirements.alignment,
                            avail: block_size,
                        })
                    }
                }
                let mut res = blocks[*free_index]
                    .bind_buffer_memory(
                        device, buffer, memory_requirements, block_size, *type_index, self.granularity, self.map_memory
                    );
                if let Err(err) = &res {
                    if let Error::OutOfDeviceMemory { size: _, align: _, avail: _ } = err {
                        *free_index += 1;
                        if *free_index == blocks.len() {
                            blocks.push(Block::new());
                        }
                        res = blocks[*free_index].bind_buffer_memory(
                            device, buffer, memory_requirements, self.block_size, *type_index, self.granularity, self.map_memory
                        );
                    } else {
                        return Err(err.clone())
                    }
                }
                return Ok(Box::new(res.unwrap()))
            }
        }
        return Err(IncompatibleMemoryRequirements)
    }
}

impl Drop for LinearDeviceAlloc {

    fn drop(&mut self) {
        unsafe {
            let device = &*self.device;
            for (blocks, _, _) in self.blocks.iter_mut() {
                for block in blocks.iter_mut() {
                    block.free_memory(device);
                }
            }
            self.blocks.clear();
        }
    }
}
