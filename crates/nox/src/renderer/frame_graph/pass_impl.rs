use super::*;

use ash::vk;

use nox_mem::{Allocator, CapacityError, vec_types::{FixedVec, Vector}};

use crate::{
    renderer::{
        *,
        frame_state::*,
        image::ImageSubresourceRangeInfo,
    },
    has_not_bits,
};

pub(crate) struct Pass<'alloc, Alloc: Allocator> {
    pub id: PassId,
    pub reads: FixedVec<'alloc, ReadInfo, Alloc>,
    pub writes: FixedVec<'alloc, WriteInfo, Alloc>,
    pub depth_write: Option<(bool, WriteInfo)>,
    pub render_area: Option<vk::Rect2D>,
    pub msaa_samples: MSAA,
}

impl<'alloc, Alloc: Allocator> Pass<'alloc, Alloc> {
    
    pub fn new(
        id: PassId,
        info: PassInfo,
        alloc: &'alloc Alloc
    ) -> Result<Self, CapacityError> {
        let reads = FixedVec::with_capacity(info.max_reads as usize, alloc)?;
        let writes = FixedVec::with_capacity(info.max_color_writes as usize, alloc)?;
        Ok(Self {
            id,
            reads,
            writes,
            depth_write: None,
            render_area: None,
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
            let mut write_ranges = FixedVec::<(ResourceId, WriteAccess<Alloc>), Alloc>
                ::with_capacity(writes.len(), alloc)?;
            for write in writes {

                let id = write.main_id;

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

    fn with_read(&mut self, read_info: ReadInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        if has_not_bits!(read_info.resource_id.flags, ResourceFlags::Sampleable) {
            panic!("image read must be sampleable")
        }
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
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|v| v.0) {
                assert!(
                    resolve.samples() == MSAA::X1,
                    "write resolve image sample count must be 1, resolve samples: {:?}",
                    resolve.samples(),
                );
                assert!(
                    write.main_id.format == resolve.format,
                    "write resolve image format must be the same as main image format, main format {:?}, resolve format {:?}",
                    write.main_id.format, resolve.format,
                );
            }
        }
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
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|v| v.0) {
                assert!(
                    resolve.samples() == MSAA::X1,
                    "write resolve image sample count must be 1, resolve samples: {:?}",
                    resolve.samples(),
                );
                assert!(
                    write.main_id.format == resolve.format,
                    "write resolve image format must be the same as main image format, main format {:?}, resolve format {:?}",
                    write.main_id.format, resolve.format,
                );
            }
        }
        self.depth_write = Some((false, write));
        self
    }

    fn with_depth_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a> {
        assert!(write.samples() == self.msaa_samples,
            "write MSAA sample count must match pass sample count ( write: {:?}, pass: {:?} )",
            write.samples(), self.msaa_samples,
        );
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|v| v.0) {
                assert!(
                    resolve.samples() == MSAA::X1,
                    "write resolve image sample count must be 1, resolve samples: {:?}",
                    resolve.samples(),
                );
                assert!(
                    write.main_id.format == resolve.format,
                    "write resolve image format must be the same as main image format, main format {:?}, resolve format {:?}",
                    write.main_id.format, resolve.format,
                );
            }
        }
        self.depth_write = Some((true, write));
        self
    }

    fn with_render_area(&mut self, render_area: RenderArea) -> &mut dyn PassAttachmentBuilder<'a> {
        self.render_area = Some(render_area.into());
        self
    }
}
