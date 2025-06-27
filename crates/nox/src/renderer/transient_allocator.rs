use core::ops::Range;

use ash::vk;

use crate::utility::has_not_bits;

use super::Handle;

pub enum Error {
    IncompatibleMemoryRequirements,
    VkError(vk::Result),
}

use {Error::IncompatibleMemoryRequirements, Error::VkError};

#[derive(Default, Clone, Copy)]
pub struct Block {
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
    align: vk::DeviceSize,
}

pub struct TransientAllocator<'r> {
    device_memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    device: Handle<'r, ash::Device>,
    free_blocks: Vec<Block>,
    alloc_blocks: Vec<(Block, Vec<Range<u64>>)>,
    timeline: Vec<Vec<usize>>,
    properties: vk::MemoryPropertyFlags,
}

impl<'r> TransientAllocator<'r> {

    pub fn bind_image_memory(&mut self, image: vk::Image, lifetime: Range<u64>) -> Result<(), Error> {
        let memory_requirements = unsafe { self.device.get_image_memory_requirements(image) };
        if has_not_bits!(self.properties.as_raw(), memory_requirements.memory_type_bits) {
            return Err(IncompatibleMemoryRequirements)
        }
        let start = lifetime.start as usize;
        if self.timeline.len() > start {
            let indices = &self.timeline[start];
            for index in indices {
                let (block, lifetimes) = &mut self.alloc_blocks[*index];
                if block.size != memory_requirements.size ||
                    block.align != memory_requirements.alignment {
                    continue
                }
                for block_lifetime in &mut *lifetimes {
                    if block_lifetime.start < lifetime.end &&
                        block_lifetime.end < lifetime.start
                    {
                        continue
                    }
                }
                lifetimes.push(lifetime);
                unsafe { self.device
                    .bind_image_memory(image, self.device_memory, block.offset)
                    .map_err(VkError)
                };
                return Ok(())
            }
        }
        self.timeline.resize(lifetime.end as usize, Default::default());
        Ok(())
    }
}
