use ash::vk;

use nox_mem::{Allocator, Vector, FixedVec,};

use crate::renderer::Error;

use super::DescriptorType;

#[derive(Clone, Copy)]
pub struct DescriptorPoolSize {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

impl DescriptorPoolSize {

    pub fn new(descriptor_type: DescriptorType, descriptor_count: u32) -> Self {
        Self {
            descriptor_type,
            descriptor_count,
        }
    }
}

impl From<DescriptorPoolSize> for vk::DescriptorPoolSize {

    fn from(value: DescriptorPoolSize) -> Self {
        Self {
            ty: value.descriptor_type.into(),
            descriptor_count: value.descriptor_count,
        }
    }
}

pub struct DescriptorPoolInfo<'alloc, Alloc: Allocator> {
    pool_sizes: FixedVec<'alloc, vk::DescriptorPoolSize, Alloc>,
    max_sets: u32,
}

impl<'alloc, Alloc: Allocator> DescriptorPoolInfo<'alloc, Alloc> {

    pub fn new(max_pool_sizes: u32, max_sets: u32, alloc: &'alloc Alloc) -> Result<Self, Error> {
        Ok(Self {
            pool_sizes: FixedVec::with_capacity(max_pool_sizes as usize, alloc)?,
            max_sets,
        })
    }

    pub fn with_pool_sizes(mut self, pool_sizes: &[DescriptorPoolSize]) -> Self {
        self.pool_sizes
            .append_map(pool_sizes, |e| (*e).into())
            .expect("pool size capacity exceeded");
        self
    }

    pub fn build_raw(&self, device: &ash::Device) -> Result<vk::DescriptorPool, Error> {
        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            max_sets: self.max_sets,
            pool_size_count: self.pool_sizes.len() as u32,
            p_pool_sizes: self.pool_sizes.as_ptr(),
            ..Default::default()
        };
        unsafe {
            Ok(device.create_descriptor_pool(&create_info, None)?)
        }
    }
}
