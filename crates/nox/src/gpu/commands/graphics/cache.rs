use super::*;

#[derive(Default)]
pub(super) struct BufferCache {
    pub ranges: Vec32<(vk::DeviceSize, vk::DeviceSize, vk::PipelineStageFlags2, vk::AccessFlags2, CommandOrdering)>,
}

impl BufferCache {

    pub fn touch(
        &mut self,
        mut offset: vk::DeviceSize,
        mut size: vk::DeviceSize,
        mut stage: vk::PipelineStageFlags2,
        mut access: vk::AccessFlags2,
        mut ordering: CommandOrdering,
    ) -> bool {
        let new = self.ranges.is_empty();
        let mut index = 0;
        for i in (0..self.ranges.len()).rev() {
            let dst_end = offset + size;
            let &(src_offset, src_size, src_stage, src_access, src_ordering) = unsafe {
                self.ranges.get_unchecked(i as usize)
            };
            let src_end = src_offset + src_size;
            if src_offset < dst_end && offset < src_end {
                let (new_offset, new_size) =
                if src_offset >= offset && src_end <= dst_end {
                    (offset, size)
                } else if
                    offset >= src_offset &&
                    dst_end <= src_end
                {
                    (src_offset, src_size)
                } else if src_offset < offset {
                    let d = offset - src_offset;
                    let end = (src_offset + d).max(dst_end);
                    let size = end - src_offset;
                    (src_offset, size)
                } else {
                    let d = src_offset - offset;
                    let end = (offset + d).max(src_end);
                    let size = end - offset;
                    (offset, size)
                };
                self.ranges.remove(i);
                offset = new_offset;
                size = new_size;
                stage |= src_stage;
                access |= src_access;
                if src_ordering == CommandOrdering::Strict
                {
                    ordering = CommandOrdering::Strict;
                }
            }
            index = i;
        }
        self.ranges.insert(index, (offset, size, stage, access, ordering));
        new
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.ranges.clear();
    }
}

#[derive(Default)]
pub(super) struct ImageCache {
    pub layout: Option<ShaderImageLayout>,
    pub access: Option<u8>,
    pub shader_stage_mask: vk::ShaderStageFlags,
    pub subresource_ranges: AHashSet<Option<ImageSubresourceRange>>,
}

impl ImageCache {

    #[inline(always)]
    pub fn touch(
        &mut self,
        layout: ShaderImageLayout,
        access: Option<ShaderAccess>,
        shader_stage_mask: vk::ShaderStageFlags,
    ) -> bool
    {
        if let Some(access) = access {
            let current = self.access.get_or_insert(access.as_raw());
            *current |= access;
        }
        self.shader_stage_mask |= shader_stage_mask;
        match &mut self.layout {
            Some(current) => {
                *current = current.combine(layout);
                false
            },
            None => {
                self.layout = Some(layout);
                true
            },
        }
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.layout = None;
        self.access = None;
        self.subresource_ranges.clear();
    }
}

#[derive(Default)]
pub(crate) struct GraphicsCommandCache {
    pub image_memory_barrier_cache: ImageMemoryBarrierCache,
    pub(super) image_cache: AHashMap<ImageId, ImageCache>,
    pub(super) image_id_cache: Vec32<ImageId>,
    pub(super) buffer_memory_barrier_cache: BufferMemoryBarrierCache,
    pub(super) buffer_cache: AHashMap<BufferId, BufferCache>,
    pub(super) buffer_id_cache: Vec32<BufferId>,
    pub(super) draw_cache: UnsafeCell<DrawCommandCache>,
}
