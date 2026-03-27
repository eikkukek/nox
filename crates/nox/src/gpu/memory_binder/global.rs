use compact_str::format_compact;

use nox_mem::vec::FixedVec32;
use nox_ash::vk;

use crate::gpu::prelude::*;

use super::*;

use MemoryBinderError::*;

#[derive(Clone)]
pub struct GlobalBinder {
    gpu: Gpu,
    optimal_memory_type_bits: u32,
    suboptimal_memory_type_bits: u32,
    optimal_coherency: HostCoherency,
    suboptimal_coherency: HostCoherency,
}

impl GlobalBinder {

    /// Creates a new global binder, which always allocates new [`device memory`][1].
    ///
    /// # Parameters
    /// - `optimal`: optimal memory properties
    /// - `suboptimal`: suboptimal memory properties
    pub fn new(
        gpu: Gpu,
        optimal_properties: MemoryProperties,
        suboptimal_properties: MemoryProperties,
    ) -> Self
    {
        let optimal = optimal_properties.into();
        let suboptimal = suboptimal_properties.into();
        let memory_properties = gpu.device().physical_device().memory_properties();
        let mut optimal_memory_type_bits = 0;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize]
            .iter()
            .enumerate()
        {
            if memory_type.property_flags & optimal == optimal {
                optimal_memory_type_bits |= 1 << i;
            }
        }
        let mut suboptimal_memory_type_bits = 0;
        for (i, memory_type) in memory_properties.memory_types[..memory_properties.memory_type_count as usize]
            .iter()
            .enumerate()
        {
            if memory_type.property_flags.contains(suboptimal) {
                suboptimal_memory_type_bits |= 1 << i;
            }
        }
        Self {
            gpu,
            optimal_memory_type_bits,
            suboptimal_memory_type_bits,
            optimal_coherency:
                if optimal_properties.contains(MemoryProperties::HOST_COHERENT) {
                    HostCoherency::Coherent
                } else if optimal_properties.contains(MemoryProperties::HOST_VISIBLE) {
                    HostCoherency::Mappable
                } else {
                    HostCoherency::None
                },
            suboptimal_coherency:
                if suboptimal_properties.contains(MemoryProperties::HOST_COHERENT) {
                    HostCoherency::Coherent
                } else if suboptimal_properties.contains(MemoryProperties::HOST_VISIBLE) {
                    HostCoherency::Mappable
                } else {
                    HostCoherency::None
                },
        }
    }
}

pub struct Memory {
    gpu: Gpu,
    memory: vk::DeviceMemory,
    size: DeviceSize,
    map: *mut u8,
    host_coherency: HostCoherency,
    is_optimal: bool,
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

unsafe impl DeviceMemory for Memory {

    fn handle(&self) -> u64 {
        <_ as vk::Handle>::as_raw(self.memory)
    }

    fn offset(&self) -> vk::DeviceSize {
        0
    }
    
    fn size(&self) -> vk::DeviceSize {
        self.size
    }

    fn map_memory(&mut self) -> Result<MemoryMap> {
        if self.host_coherency == HostCoherency::None {
            return Err(Error::just_context(UnmappableMemory));
        }
        if !self.map.is_null() {
            return Ok(MemoryMap {
                map: self.map,
                size: self.size as usize,
                coherent: self.host_coherency == HostCoherency::Coherent,
            })
        }
        let ptr = unsafe {
            self.gpu.device().map_memory(
                self.memory,
                0,
                vk::WHOLE_SIZE,
                vk::MemoryMapFlags::from_raw(0)
            )
        }.context("failed to map memory")?;
        self.map = ptr as *mut u8;
        Ok(MemoryMap {
            map: self.map,
            size: self.size as usize,
            coherent: self.host_coherency == HostCoherency::Coherent,
        })
    }

    fn unmap_memory(&mut self) -> Result<()> {
        if self.map.is_null() {
            return Err(Error::just_context(format_compact!(
                "meory is not mapped"
            )))
        }
        self.map = core::ptr::null_mut();
        unsafe {
            self.gpu.device().unmap_memory(self.memory);
        }
        Ok(())
    }

    fn flush_mapped_ranges(&self, memory_ranges: &[MappedMemoryRange]) -> Result<()> {
        if self.map.is_null() {
            return Err(Error::just_context("memory is not mapped"))
        }
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut vk_ranges = FixedVec32::with_capacity(memory_ranges.len() as u32, &tmp_alloc)?;
        let memory = self.memory;
        let non_coherent_atom_size = self.gpu.device_limits().non_coherent_atom_size();
        vk_ranges.try_extend(memory_ranges.iter().map(|range| {
            if !range.offset.is_multiple_of(non_coherent_atom_size) {
                return Err(Error::just_context(format_compact!(
                    "range offset {} is not a multiple of non coherent atom size {}",
                    range.offset, non_coherent_atom_size,
                )))
            }
            if !range.size.is_multiple_of(non_coherent_atom_size) &&
                range.offset + range.size  != self.size
            {
                return Err(Error::just_context(format_compact!(
                    "range size {} is not amultiple of non coherent atom size {}",
                    range.size, non_coherent_atom_size
                )))
            }
            if range.offset + range.size > self.size {
                return Err(Error::just_context(format_compact!(
                    "range offset {} + size {} is greater than allocation size {}",
                    range.offset, range.size, self.size,
                )))
            }
            Ok(vk::MappedMemoryRange {
                memory,
                offset: range.offset,
                size: range.size,
                ..Default::default()
            })
        }))?;
        unsafe {
            self.gpu.device().flush_mapped_memory_ranges(&vk_ranges)
        }.context("failed to flush mapped memory ranges")
    }

    fn invalidate_mapped_ranges(&self, memory_ranges: &[MappedMemoryRange]) -> Result<()> {
        if self.map.is_null() {
            return Err(Error::just_context("memory is not mapped"))
        }
        let tmp_alloc = self.gpu.tmp_alloc();
        let tmp_alloc = tmp_alloc.guard();
        let mut vk_ranges = FixedVec32::with_capacity(memory_ranges.len() as u32, &tmp_alloc)?;
        let memory = self.memory;
        let non_coherent_atom_size = self.gpu.device_limits().non_coherent_atom_size();
        vk_ranges.try_extend(memory_ranges.iter().map(|range| {
            if !range.offset.is_multiple_of(non_coherent_atom_size) {
                return Err(Error::just_context(format_compact!(
                    "range offset {} is not a multiple of non coherent atom size {}",
                    range.offset, non_coherent_atom_size,
                )))
            }
            if !range.size.is_multiple_of(non_coherent_atom_size) &&
                range.offset + range.size != self.size
            {
                return Err(Error::just_context(format_compact!(
                    "range size {} is not amultiple of non coherent atom size {}",
                    range.size, non_coherent_atom_size
                )))
            }
            if range.offset + range.size > self.size {
                return Err(Error::just_context(format_compact!(
                    "range offset {} + size {} is greater than allocation size {}",
                    range.offset, range.size, self.size,
                )))
            }
            Ok(vk::MappedMemoryRange {
                memory,
                offset: range.offset,
                size: range.size,
                ..Default::default()
            })
        }))?;
        unsafe {
            self.gpu.device().invalidate_mapped_memory_ranges(&vk_ranges)
        }.context("failed to invalidate mapped memory ranges")
    }

    #[inline]
    fn is_optimal(&self) -> bool {
        self.is_optimal
    }
}

impl Drop for Memory {

    fn drop(&mut self) {
        unsafe {
            self.gpu.device()
                .free_memory(self.memory, None);
        }
    }
}

unsafe impl MemoryBinder for GlobalBinder {

    #[inline]
    fn max_alloc_size(&self) -> vk::DeviceSize {
        self.gpu.device().max_memory_allocation_size()
    }

    #[inline]
    fn optimal_coherency(&self) -> HostCoherency {
        self.optimal_coherency
    }

    #[inline]
    fn suboptimal_coherency(&self) -> HostCoherency {
        self.suboptimal_coherency
    }

    unsafe fn alloc(
        &self,
        memory_requirements: &vk::MemoryRequirements2,
    ) -> Result<DeviceMemoryObj> {
        let mut is_optimal = true;
        let mut host_coherency = self.optimal_coherency;
        let mut memory_type_bits = self.optimal_memory_type_bits &
            memory_requirements.memory_requirements.memory_type_bits;
        if memory_type_bits == 0 {
            memory_type_bits = self.suboptimal_memory_type_bits &
                memory_requirements.memory_requirements.memory_type_bits;
            if memory_type_bits == 0 {
                return Err(Error::just_context(IncompatibleMemoryRequirements))
            }
            host_coherency = self.suboptimal_coherency;
            is_optimal = false;
        }
        let memory_type_index = memory_type_bits.trailing_zeros();
        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };
        let memory = unsafe {
            self.gpu.device().allocate_memory(&allocate_info, None)
        }.context("failed to allocate device memory")?;
        Ok(DeviceMemoryObj::new(Memory {
            gpu: self.gpu.clone(),
            memory,
            size: memory_requirements.memory_requirements.size,
            map: core::ptr::null_mut(),
            host_coherency,
            is_optimal,
        }))
    }

    #[inline]
    unsafe fn release_resources(&self) {}
}
