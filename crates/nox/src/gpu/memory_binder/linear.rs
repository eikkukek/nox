use std::sync::Arc;

use core::{
    ptr::NonNull,
    slice,
};

use nox_ash::vk;

use nox_mem::{
    vec::{Vec32, Vector},
    option::OptionExt,
};

use crate::dev::has_bits;

use crate::gpu::Vulkan;

use super::*;

use MemoryBinderError::{self, *};

struct Allocation {
    vk: Arc<Vulkan>,
    device_memory: vk::DeviceMemory,
}

impl Drop for Allocation {

    fn drop(&mut self) {
        unsafe {
            self.vk.device()
                .free_memory(self.device_memory, None);
        }
    }
}

struct Block {
    allocation: Option<Arc<Allocation>>,
    mapped_pointer: Option<NonNull<u8>>,
    used: vk::DeviceSize,
}

unsafe impl Send for Block {}
unsafe impl Sync for Block {}

impl Block {

    #[inline(always)]
    fn new() -> Self {
        Self {
            allocation: None,
            mapped_pointer: None,
            used: 0,
        }
    }

    #[inline(always)]
    unsafe fn alloc(
        &mut self,
        vk: &Arc<Vulkan>,
        memory_requirements: &vk::MemoryRequirements2,
        block_size: vk::DeviceSize,
        memory_type_index: u32,
        granularity: vk::DeviceSize,
    ) -> Result<Memory>
    {
        let allocation = self.allocation.get_or_try_insert_with::<MemoryBinderError, _>(|| {
            let allocate_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                allocation_size: block_size,
                memory_type_index,
                ..Default::default()
            };
            Ok(Arc::new(Allocation {
                vk: vk.clone(),
                device_memory: unsafe {
                    vk.device().allocate_memory(&allocate_info, None)?
                }
            }))
        })?;
        let used = self.used;
        let align = memory_requirements.memory_requirements.alignment.max(granularity);
        let offset = (used + align - 1) & !(align - 1);
        let end = offset + memory_requirements.memory_requirements.size;
        if block_size < end {
            return Err(OutOfDeviceMemory { size: memory_requirements.memory_requirements.size, align, } )
        }
        self.used = end;
        Ok(Memory::new(allocation.clone(), None, offset, memory_requirements.memory_requirements.size))
    }

    #[inline(always)]
    unsafe fn reset(&mut self) {
        self.used = 0;
    }

    #[inline(always)]
    unsafe fn drop_alloc(&mut self) {
        self.allocation.take();
        self.used = 0;
    }
}

pub struct LinearBinder {
    vk: Arc<Vulkan>,
    blocks: Vec32<(Vec32<Block>, u32, usize)>,
    block_size: vk::DeviceSize,
    map_memory: bool,
    fallback: DefaultBinder,
}

impl LinearBinder {

    pub(crate) fn new(
        vk: Arc<Vulkan>,
        block_size: vk::DeviceSize,
        required_properties: vk::MemoryPropertyFlags,
        forbidden_properties: vk::MemoryPropertyFlags,
        map_memory: bool,
    ) -> Result<Self>
    {
        let physical_device_info = vk.physical_device_info();
        let memory_properties = physical_device_info.memory_properties();
        let mut blocks = Vec32::with_capacity(4);
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize]
            .iter()
            .enumerate()
        {
            let property_flags = memory_type.property_flags;
            if has_bits!(property_flags, required_properties) && !property_flags.intersects(forbidden_properties) {
                blocks.push((Vec32::with_len_with(1, |_| Block::new()), i as u32, 0));
            }
        }
        Ok(Self {
            fallback: DefaultBinder::new(vk.clone(), required_properties, forbidden_properties),
            vk,
            blocks,
            block_size,
            map_memory,
        })
    }
    
    #[inline(always)]
    pub fn default_attributes(block_size: u64) -> LinearBinderAttributes {
        LinearBinderAttributes {
            block_size,
            map_memory: false,
        }
    }

    #[inline(always)]
    pub fn is_mappable(&self) -> bool {
        self.map_memory
    }

    #[inline(always)]
    pub fn block_size(&self) -> u64 {
        self.block_size
    }

    unsafe fn reset(&mut self) {
        unsafe {
            for (blocks, _, i) in self.blocks.iter_mut() {
                *i = 0;
                for block in blocks.iter_mut() {
                    block.reset();
                }
            }
        }
    } 
}

struct Memory {
    allocation: Arc<Allocation>,
    map: Option<NonNull<u8>>,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl Memory {

    fn new(
        allocation: Arc<Allocation>,
        map: Option<NonNull<u8>>,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> Self
    {
        Self {
            allocation,
            map,
            offset,
            size,
        }
    }

    fn get_mapped_memory(&mut self) -> Option<&mut [u8]> {
        unsafe {
            Some(slice::from_raw_parts_mut(
                self.map?.add(self.offset as usize).as_ptr(),
                self.size as usize,
            ))
        }
    }
}

impl DeviceMemory for Memory {

    fn device_memory(&self) -> vk::DeviceMemory {
        self.allocation.device_memory
    }

    fn offset(&self) -> vk::DeviceSize {
        self.offset
    }

    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    fn map_memory(&mut self) -> Result<&mut [u8]> {
        self.get_mapped_memory()
            .ok_or(UnmappableMemory)
    }
}

unsafe impl MemoryBinder for LinearBinder {
    
    #[inline(always)]
    fn max_alloc_size(&self) -> vk::DeviceSize {
        self.block_size()
    }

    #[inline(always)]
    fn is_mappable(&self) -> bool {
        self.is_mappable()
    }

    unsafe fn alloc(
        &mut self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> Result<Box<dyn DeviceMemory>> {
        let block_size = self.block_size;
        let granularity = self.vk
            .physical_device_info()
            .properties().limits.buffer_image_granularity;
        for (blocks, type_index, free_index) in self.blocks.iter_mut() {
            if memory_requirements.memory_requirements.memory_type_bits & (1 << *type_index) != 0 {
                if block_size < memory_requirements.memory_requirements.size {
                    return unsafe {
                        self.fallback.alloc(memory_requirements)
                    };
                }
                let mut res = unsafe { blocks[*free_index]
                    .alloc(
                        &self.vk, memory_requirements,
                        self.block_size, *type_index,
                        granularity,
                    )
                };
                if let Err(err) = res {
                    if let OutOfDeviceMemory { size: _, align: _, } = err {
                        *free_index += 1;
                        if *free_index == blocks.len() as usize {
                            blocks.push(Block::new());
                        }
                        res = unsafe { blocks[*free_index].alloc(
                            &self.vk, memory_requirements, self.block_size,
                            *type_index, granularity,
                        ) };
                    } else {
                        return Err(err)
                    }
                }
                return Ok(Box::new(res.unwrap()))
            }
        }
        Err(IncompatibleMemoryRequirements)
    }

    #[inline(always)]
    unsafe fn release_resources(&mut self) {
        unsafe {
            self.reset();
        }
    }
}

impl Drop for LinearBinder {

    fn drop(&mut self) {
        unsafe {
            for (blocks, _, _) in self.blocks.iter_mut() {
                for block in blocks.iter_mut() {
                    block.drop_alloc();
                }
            }
            self.blocks.clear();
        }
    }
}

pub struct LinearBinderAttributes {
    block_size: vk::DeviceSize,
    map_memory: bool,
}

impl LinearBinderAttributes {

    #[inline(always)]
    pub fn with_map_memory(mut self, map_memory: bool) -> Self {
        self.map_memory = map_memory;
        self
    }
}

impl MemoryBinderAttributes for LinearBinderAttributes  {

    type Binder = LinearBinder;

    const NAME: &str = "Linear binder";

    #[inline(always)]
    fn build(self, vulkan: Arc<Vulkan>) -> Result<Self::Binder> {
        if self.map_memory {
            LinearBinder::new(
                vulkan,
                self.block_size,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vk::MemoryPropertyFlags::DEVICE_LOCAL, 
                true,
            )
        } else {
            LinearBinder::new(
                vulkan,
                self.block_size,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
                true,
            )
        }
    }
}
