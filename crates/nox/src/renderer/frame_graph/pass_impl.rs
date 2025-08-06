use super::*;

use ash::vk;

use nox_mem::{Allocator, CapacityError, vec_types::{FixedVec, Vector}};

use crate::renderer::{
    *,
    frame_state::ResourceID,
    image::ImageSubresourceRangeInfo,
};

pub(crate) struct Pass<'alloc, Alloc: Allocator> {
    pub reads: FixedVec<'alloc, ReadInfo, Alloc>,
    pub writes: FixedVec<'alloc, WriteInfo, Alloc>,
    pub depth_write: Option<WriteInfo>,
    pub stencil_write: Option<WriteInfo>,
    pub dependencies: FixedVec<'alloc, usize, Alloc>,
    pub render_area: Option<vk::Rect2D>,
    pub callback: Option<PassCallback>,
    pub msaa_samples: MSAA,
}

impl<'alloc, Alloc: Allocator> Pass<'alloc, Alloc> {
    
    pub fn new(
        info: PassInfo,
        alloc: &'alloc Alloc
    ) -> Result<Self, CapacityError> {
        let reads =
            if info.max_reads != 0 {
                FixedVec::with_capacity(info.max_reads as usize, alloc)?
            }
            else {
                FixedVec::with_no_alloc()
            };
        let writes =
            if info.max_writes != 0 {
                FixedVec::with_capacity(info.max_writes as usize, alloc)?
            }
            else {
                FixedVec::with_no_alloc()
            };
        let dependencies =
            if info.max_dependencies != 0 {
                FixedVec::with_capacity(info.max_dependencies as usize, alloc)?
            }
            else {
                FixedVec::with_no_alloc()
            };
        Ok(Self {
            reads,
            writes,
            depth_write: None.into(),
            stencil_write: None.into(),
            dependencies,
            render_area: None,
            callback: None,
            msaa_samples: info.msaa_samples,
        })
    }

    pub fn validate(&self, alloc: &Alloc) -> Result<bool, CapacityError> {

        enum WriteAccess<'a, Alloc: Allocator> {
            Whole,
            Partial(FixedVec<'a, ImageSubresourceRangeInfo, Alloc>),
        }

        if self.writes.len() != 0 {
            let reads = &self.reads;
            let writes = &self.writes;
            let mut write_ranges = FixedVec::<(ResourceID, WriteAccess<Alloc>), Alloc>
                ::with_capacity(writes.len(), alloc)?;
            for write in writes {

                let id = write.resource_id;

                if write.range_info.is_none() {
                    if reads.iter().any(|r| r.resource_id == id) ||
                        write_ranges.iter().any(|(v_id, _)| *v_id == id)
                    {
                        // Whole write / Partial read overlap
                        return Ok(false)
                    }
                    write_ranges.push((id, WriteAccess::Whole)).unwrap();
                    continue
                }

                let current_range = write.range_info.unwrap();

                if let Some((_, access)) = write_ranges
                    .iter_mut()
                    .find(|(v_id, _)| *v_id == id)
                {
                    match access {
                        WriteAccess::Whole => {
                            // Whole write / Partial write overlap
                            return Ok(false)
                        }
                        WriteAccess::Partial(ranges) => {
                            for r in ranges.iter() {
                                if r.overlaps(current_range.subresource_info) {
                                    // Partial write / Partial write overlap
                                    return Ok(false)
                                }
                            }
                            ranges.push(current_range.subresource_info).unwrap();
                        },
                    }
                }
                else {
                    let mut ranges = FixedVec::with_capacity(writes.len(), alloc)?;
                    ranges.push(current_range.subresource_info).unwrap();
                    write_ranges.push((id, WriteAccess::Partial(ranges))).unwrap();
                }
            }
        }
        Ok(true)
    }
}

impl<'a, Alloc: Allocator> PassAttachmentBuilder<'a> for Pass<'a, Alloc> {

    fn with_callback(&mut self, callback: PassCallback) -> &mut dyn PassAttachmentBuilder<'a> {
        self.callback = Some(callback);
        self
    }

    fn with_read(&mut self, read_info: ReadInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        self.reads
            .push(read_info)
            .expect("read capacity exceeded");
        self
    }

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.samples() == self.msaa_samples,
            "write MSAA sample count must match pass sample count ( write: {:?}, pass: {:?} )",
            write.samples(), self.msaa_samples,
        );
        self.writes
            .push(write)
            .expect("write capacity exceeded");
        self
    }

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.samples() == self.msaa_samples,
            "write MSAA sample count must match pass sample count ( write: {:?}, pass: {:?} )",
            write.samples(), self.msaa_samples,
        );
        self.depth_write = Some(write);
        self
    }

    fn with_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.resource_id.samples == self.msaa_samples,
            "write MSAA sample count must match pass sample count ( write: {:?}, pass: {:?} )",
            write.resource_id.samples, self.msaa_samples,
        );
        self.stencil_write = Some(write);
        self
    }

    fn with_render_area(&mut self, render_area: RenderArea) -> &mut dyn PassAttachmentBuilder<'a> {
        self.render_area = Some(render_area.into());
        self
    }

    fn with_dependency(&mut self, pass_index: usize) -> &mut dyn PassAttachmentBuilder<'a> {
        self.dependencies
            .push(pass_index)
            .expect("dependency capacity exceeded");
        self
    }
}
