use nox_mem::{Vector, CapacityError, capacity_policy::Dyn};

use super::{
    physical_device::PhysicalDeviceInfo,
    handle::Handle,
};

use crate::{
    array_format,
    string_types::{ArrayString, SmallError},
    has_bits,
    has_not_bits,
};

use ash::vk;

use core::{
    cell::RefCell,
    marker::PhantomData,
};

#[derive(Default, Clone, Copy)]
pub struct Block {
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

pub trait BufferAllocator<'r> {

    unsafe fn allocate(&mut self, size: vk::DeviceSize, align: vk::DeviceSize ) -> Option<Block>;

    unsafe fn free(&mut self, allocation: Block) -> Result<(), CapacityError>;

    fn device(&self) -> &'_ Handle<'r, ash::Device>;

    fn memory(&self) -> vk::DeviceMemory;

    fn properties(&self) -> vk::MemoryPropertyFlags;
}

pub struct DeviceMemory<'alloc, 'r, Alloc>
    where
        Alloc: BufferAllocator<'r>
{
    block: Block,
    allocator: &'alloc RefCell<Alloc>,
    _marker: PhantomData<&'r ()>
}

impl<'alloc, 'r, Alloc> DeviceMemory<'alloc, 'r, Alloc>
    where
        Alloc: BufferAllocator<'r>
{

    pub fn new_for_image(image: Handle<'r, vk::Image>, allocator: &'alloc RefCell<Alloc>) -> Result<Self, SmallError>
    {
        let mut a = allocator.borrow_mut();
        let memory_requirements = unsafe { a.device().get_image_memory_requirements(*image) };
        if has_not_bits!(a.properties().as_raw(), memory_requirements.memory_type_bits) {
            return Err(ArrayString::from_str("incompatible allocator memory properties"))
        }
        let block = unsafe {
            a.allocate(memory_requirements.size, memory_requirements.alignment)
                .ok_or_else(|| ArrayString::from_str("out of memory")
                )?
        };
        unsafe {
            a.device()
                .bind_image_memory(*image, a.memory(), block.offset)
                .map_err(|e| ArrayString::format(format_args!(
                    "failed to bind memory {:?}", e))
                )?
        };
        Ok(Self {
            block,
            allocator,
            _marker: PhantomData,
        })
    }

    pub fn new_for_buffer(buffer: Handle<'r, vk::Buffer>, allocator: &'alloc RefCell<Alloc>) -> Result<Self, SmallError>
    {
        let mut a = allocator.borrow_mut();
        let memory_requirements = unsafe { a.device().get_buffer_memory_requirements(*buffer) };
        if has_not_bits!(a.properties().as_raw(), memory_requirements.memory_type_bits) {
            return Err(ArrayString::from_str("incompatible allocator memory properties"))
        }
        let block = unsafe { a 
            .allocate(memory_requirements.size, memory_requirements.alignment)
            .ok_or_else(|| ArrayString::from_str("out of memory"))?};
        unsafe {
            a.device()
                .bind_buffer_memory(*buffer, a.memory(), block.offset)
                .map_err(|e| ArrayString::format(format_args!(
                    "failed to bind memory {:?}", e
                )))?;
        }
        Ok(Self {
            block,
            allocator,
            _marker: PhantomData,
        })
    }
}

impl<'alloc, 'r, Alloc> Drop for DeviceMemory<'alloc, 'r, Alloc>
    where
        Alloc: BufferAllocator<'r>,
{

    fn drop(&mut self) {
        unsafe {
            let _ =
            self.allocator
                .borrow_mut()
                .free(self.block)
                .map_err(|e| eprintln!("failed to free GPU memory ( {:?} )", e));
        }
    }
}

pub struct BufferAlloc<'r, DynVec>
    where
        DynVec: Vector<Block, CapacityPol = Dyn>,
{
    device: Handle<'r, ash::Device>,
    memory: Handle<'r, vk::DeviceMemory>,
    size: vk::DeviceSize,
    free_list: DynVec,
    properties: vk::MemoryPropertyFlags,
}

impl<'r, DynVec> BufferAlloc<'r, DynVec>
    where
        DynVec: Vector<Block, CapacityPol = Dyn>,
{

    pub fn new(
        device: Handle<'r, ash::Device>,
        physical_device_info: &PhysicalDeviceInfo,
        size: vk::DeviceSize,
        properties: vk::MemoryPropertyFlags,
        mut free_list: DynVec,
    ) -> Result<Self, SmallError> {
        let memory_properties = physical_device_info.memory_properties();
        let mut maybe_index = None;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize].iter().enumerate() {
            if has_bits!(memory_type.property_flags, properties) {
                maybe_index = Some(i as u32);
                break;
            }
        }
        let memory_type_index = maybe_index
            .ok_or_else(||
                ArrayString::from_str("could not find requested memory properties"
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
                    array_format!("failed to allocate memory {:?}", e)
                })?
        });
        free_list.resize(0, Default::default()).expect("should not happen");
        free_list
            .push(Block::new(0, size))
            .map_err(|e|
                array_format!("failed to push to free list ( {:?} )", e)
            )?;
        Ok(Self {
            memory: Handle::new(*memory),
            size,
            device: Handle::new((*device).clone()),
            free_list,
            properties,
        })
    }

    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }

    pub fn properties(&self) -> vk::MemoryPropertyFlags {
        self.properties
    }
}

impl<'r, DynVec> Drop for BufferAlloc<'r, DynVec>
    where
        DynVec: Vector<Block, CapacityPol = Dyn>
{
    fn drop(&mut self) {
        unsafe { self.device.free_memory(*self.memory, None); }
    }
}

impl<'r, DynVec> BufferAllocator<'r> for BufferAlloc<'r, DynVec>
    where
        DynVec: Vector<Block, CapacityPol = Dyn>
{

    unsafe fn allocate(&mut self, size: vk::DeviceSize, align: vk::DeviceSize ) -> Option<Block> {
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
                return Some(Block::new(aligned_offset, size))
            }
        }
        None
    }

    unsafe fn free(&mut self, allocation: Block) -> Result<(), CapacityError> {
        let end = allocation.offset + allocation.size;
        let mut i = 0;
        while i < self.free_list.len() {
            let block = &mut self.free_list[i];
            if end < block.offset {
                break;
            }
            let block_end = block.offset + block.size;
            if allocation.offset < block_end && end > block.offset {
                panic!("invalid block")
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
        self.free_list.insert(Block::new(new_offset, new_size), i)?;
        Ok(())
    }

    fn device(&self) -> &'_ Handle<'r, ash::Device> {
        &self.device
    }

    fn memory(&self) -> vk::DeviceMemory {
        (*self.memory).clone()
    }

    fn properties(&self) -> vk::MemoryPropertyFlags {
        self.properties
    }
}
