use super::*;

pub struct DescriptorPool {
    device: Arc<ash::Device>,
    handle: vk::DescriptorPool,
    allocated_sets: u32,
    max_sets: u32,
    _pool_sizes: [vk::DescriptorPoolSize; 3],
}

impl DescriptorPool {

    pub fn new(
        device: Arc<ash::Device>,
        memory_layout: MemoryLayout,
    ) -> Result<Self, Error>
    {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: memory_layout.uniform_sampled_images(),
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: memory_layout.uniform_buffers(),
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: memory_layout.uniform_storage_buffers(),
            },
        ];
        let info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            flags: vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET,
            max_sets: memory_layout.max_descriptor_sets(),
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };
        let handle = unsafe {
            device.create_descriptor_pool(&info, None)?
        };
        Ok(Self {
            device,
            handle,
            allocated_sets: 0,
            max_sets: info.max_sets,
            _pool_sizes: pool_sizes,
        })
    }

    pub fn allocate<'a, Alloc: Allocator>(
        &mut self,
        set_layouts: &[vk::DescriptorSetLayout],
        alloc: &'a Alloc,
    ) -> Result<FixedVec<'a, vk::DescriptorSet, Alloc>, Error>
    {
        let count = set_layouts.len() as u32;
        if self.allocated_sets + count > self.max_sets {
            return Err(Error::DescriptorPoolFull { max_sets: self.max_sets, allocated_sets: self.allocated_sets })
        }
        let info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.handle,
            descriptor_set_count: count,
            p_set_layouts: set_layouts.as_ptr(),
            ..Default::default()
        };
        let mut sets = FixedVec::with_capacity(count as usize, alloc)?;
        let device = &self.device;
        let res = unsafe {
            (device.fp_v1_0().allocate_descriptor_sets)(device.handle(), &info, sets.as_mut_ptr())
        };
        if res != vk::Result::SUCCESS {
            return Err(res.into())
        }
        unsafe {
            sets.set_len(count as usize);
        }
        self.allocated_sets += count;
        Ok(sets)
    }

    pub fn free(
        &mut self,
        descriptor_sets: &[vk::DescriptorSet]
    ) -> Result<(), Error>
    {
        unsafe {
            self.device.free_descriptor_sets(self.handle, descriptor_sets)?;
        }
        self.allocated_sets -= descriptor_sets.len() as u32;
        Ok(())
    }

    pub(super) fn clean_up(&mut self) {
        unsafe {
            self.device.destroy_descriptor_pool(
                self.handle, None,
            );
        }
    }
}
