use core::ptr::{self, NonNull};

use nox_ash::vk::{self, ptr_chain_iter_const, TaggedStructure};

use nox_mem::{
    vec::Vec32,
    option::OptionExt,
};

use crate::{
    gpu::prelude::LogicalDevice,
    sync::RwLock,
};

use super::*;

use MemoryBinderError::{self, *};

struct Allocation {
    device: LogicalDevice,
    device_memory: vk::DeviceMemory,
    mapped_pointer: Option<RwLock<*mut ()>>,
}

impl Allocation {

    #[inline(always)]
    fn get_mapped_ptr(&self) -> Result<Option<NonNull<u8>>> {
        let Some(ptr) = &self.mapped_pointer else {
            return Ok(None)
        };
        let mut ptr = ptr.upgradable_read();
        if ptr.is_null() &&
            let Some(err) = ptr.with_upgraded(|ptr| unsafe
            {
                match self.device.map_memory(
                    self.device_memory,
                    0, vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty()
                ) {
                    Ok(p) => { *ptr = p; None },
                    Err(err) => Some(err)
                }
            })
        {
            return Err(err.into())
        }
        Ok(Some(NonNull::new(ptr.cast()).unwrap()))
    }
}

impl Drop for Allocation {

    fn drop(&mut self) {
        unsafe {
            self.device
                .free_memory(self.device_memory, None);
        }
    }
}

unsafe impl Send for Allocation {}
unsafe impl Sync for Allocation {}

struct Block {
    allocation: Option<Arc<Allocation>>,
    used: vk::DeviceSize,
}

impl Block {

    #[inline(always)]
    fn new() -> Self {
        Self {
            allocation: None,
            used: 0,
        }
    }

    #[inline(always)]
    unsafe fn alloc(
        &mut self,
        device: &LogicalDevice,
        memory_requirements: &vk::MemoryRequirements2,
        block_size: vk::DeviceSize,
        memory_type_index: u32,
        granularity: vk::DeviceSize,
        is_mappable: bool,
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
                device: device.clone(),
                mapped_pointer: (is_mappable).then(|| RwLock::new(ptr::null_mut())),
                device_memory: unsafe {
                    device.allocate_memory(&allocate_info, None)?
                },
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
        Ok(Memory::new(allocation.clone(), offset, memory_requirements.memory_requirements.size))
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
    device: LogicalDevice,
    blocks: Vec32<(Vec32<Block>, u32, usize)>,
    block_size: vk::DeviceSize,
    is_mappable: bool,
    fallback: DefaultBinder,
}

impl LinearBinder {

    pub(crate) fn new(
        device: LogicalDevice,
        block_size: vk::DeviceSize,
        required_properties: vk::MemoryPropertyFlags,
        is_mappable: bool,
    ) -> Result<Self>
    {
        let physical_device = device.physical_device();
        let memory_properties = physical_device.memory_properties();
        let mut blocks = Vec32::with_capacity(4);
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize]
            .iter()
            .enumerate()
        {
            let property_flags = memory_type.property_flags;
            if property_flags.contains(required_properties) {
                blocks.push((Vec32::with_len_with(1, |_| Block::new()), i as u32, 0));
            }
        }
        Ok(Self {
            fallback: DefaultBinder::new(device.clone(), required_properties),
            device,
            blocks,
            block_size,
            is_mappable,
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
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl Memory {

    fn new(
        allocation: Arc<Allocation>,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) -> Self
    {
        Self {
            allocation,
            offset,
            size,
        }
    }
}

unsafe impl DeviceMemory for Memory {

    fn device_memory(&self) -> vk::DeviceMemory {
        self.allocation.device_memory
    }

    fn offset(&self) -> vk::DeviceSize {
        self.offset
    }

    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    fn map_memory(&mut self) -> Result<(*mut u8, usize)> {
        self.allocation
            .get_mapped_ptr()?
            .ok_or(UnmappableMemory)
            .map(|ptr| unsafe {
                (ptr.add(self.offset as usize).as_ptr(), self.size as usize)
            })
    }
}

unsafe impl MemoryBinder for LinearBinder {
    
    #[inline(always)]
    fn max_alloc_size(&self) -> vk::DeviceSize {
        self.block_size()
    }

    #[inline(always)]
    fn is_mappable(&self) -> bool {
        self.is_mappable
    }

    unsafe fn alloc(
        &mut self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> Result<DeviceMemoryObj> {
        let block_size = self.block_size;
        let granularity = self.device
            .physical_device()
            .limits().buffer_image_granularity;
        unsafe {
            if let Some(dedicated_requirements) = ptr_chain_iter_const(memory_requirements)
                .find(|ptr| (**ptr).s_type == vk::MemoryDedicatedRequirements::STRUCTURE_TYPE)
            {
                let dedicated_requirements = dedicated_requirements
                    .cast::<vk::MemoryDedicatedRequirements>()
                    .as_ref().unwrap();
                if dedicated_requirements.prefers_dedicated_allocation != 0 {
                    return self.fallback.alloc(memory_requirements)
                }
            }
        }
        let is_mappable = self.is_mappable();
        for (blocks, type_index, free_index) in self.blocks.iter_mut() {
            if memory_requirements.memory_requirements.memory_type_bits & (1 << *type_index) != 0 {
                if block_size < memory_requirements.memory_requirements.size {
                    return unsafe {
                        self.fallback.alloc(memory_requirements)
                    };
                }
                let mut res = unsafe { blocks[*free_index]
                    .alloc(
                        &self.device, memory_requirements,
                        self.block_size, *type_index,
                        granularity,
                        is_mappable,
                    )
                };
                if let Err(err) = res {
                    if let OutOfDeviceMemory { size: _, align: _, } = err {
                        *free_index += 1;
                        if *free_index == blocks.len() as usize {
                            blocks.push(Block::new());
                        }
                        res = unsafe { blocks[*free_index].alloc(
                            &self.device, memory_requirements, self.block_size,
                            *type_index, granularity,
                            self.is_mappable,
                        ) };
                    } else {
                        return Err(err)
                    }
                }
                return Ok(DeviceMemoryObj::new(res.unwrap()))
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
    fn build(self, device: LogicalDevice) -> Result<Self::Binder> {
        if self.map_memory {
            LinearBinder::new(
                device,
                self.block_size,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                true,
            )
        } else {
            LinearBinder::new(
                device,
                self.block_size,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                false,
            )
        }
    }
}
