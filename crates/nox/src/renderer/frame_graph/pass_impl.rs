use super::*;

use ash::vk;

use nox_mem::{Allocator, CapacityError, slot_map::SlotIndex, vec_types::{FixedVec, Vector}};

use crate::renderer::{
    frame_state::ResourceID,
    MSAA,
    pipeline::{PipelineID, PipelineCache, PipelineTypeInfo},
    image::ImageSubresourceRangeInfo,
};

pub(crate) struct Pass<'alloc, Alloc: Allocator> {
    pub reads: FixedVec<'alloc, ReadInfo, Alloc>,
    pub writes: FixedVec<'alloc, WriteInfo, Alloc>,
    pub depth_write: Option<WriteInfo>,
    pub stencil_write: Option<WriteInfo>,
    pub dependencies: Option<FixedVec<'alloc, usize, Alloc>>,
    pub render_area: Option<vk::Rect2D>,
    pub pipeline_cache: &'alloc PipelineCache,
    pub pipelines: Option<FixedVec<'alloc, PipelineID, Alloc>>,
    pub callback: Option<fn(usize)>,
    pub last_pipeline_type_index: Option<SlotIndex<PipelineTypeInfo>>,
    pub msaa_samples: MSAA,
}

impl<'alloc, Alloc: Allocator> Pass<'alloc, Alloc> {
    
    pub fn new(
        info: PassInfo,
        pipeline_cache: &'alloc PipelineCache,
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
                Some(FixedVec::with_capacity(info.max_dependencies as usize, alloc)?)
            }
            else {
                None
            };
        let pipelines =
            if info.max_pipelines != 0 {
                Some(FixedVec::with_capacity(info.max_pipelines as usize, alloc)?)
            }
            else {
                None
            };
        Ok(Self {
            reads,
            writes,
            depth_write: None.into(),
            stencil_write: None.into(),
            dependencies,
            render_area: None,
            pipeline_cache,
            pipelines,
            callback: None,
            last_pipeline_type_index: None,
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

    fn as_pipeline_builder(&mut self) -> &mut dyn PassPipelineBuilder<'a> {
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
            .as_mut()
            .expect("dependency capacity exceeded")
            .push(pass_index)
            .expect("dependency capacity exceeded");
        self
    }
}

impl<'pass, Alloc: Allocator> PassPipelineBuilder<'pass> for Pass<'pass, Alloc> {

    fn with_pipeline(&mut self, id: PipelineID) -> &mut dyn PassPipelineBuilder<'pass> {
        if self.last_pipeline_type_index.is_none_or(|v| v != id.type_index())
        {
            let type_info = self.pipeline_cache.get_type_info(id);

            assert!(type_info.msaa_samples() == self.msaa_samples.into(), "pipeline MSAA sample count must match pass sample count");

            assert!(type_info.depth_format() == self.depth_write.as_ref().map_or(vk::Format::UNDEFINED, |r| r.vk_format()),
                "pipeline depth format must match pass depth format");

            assert!(type_info.stencil_format() == self.stencil_write.as_ref().map_or(vk::Format::UNDEFINED, |r| r.vk_format()),
                "pipeline stencil format must match pass stencil format");

            let format_count = type_info.color_formats().len();
            assert!(format_count <= self.writes.len());
            for (i, format) in type_info.color_formats()[0..format_count].iter().enumerate() {
                assert!(self.writes[i].vk_format() == *format, "pipeline color formats must match pass color write formats")
            }

            self.last_pipeline_type_index = Some(id.type_index());
        } 
        self.pipelines
            .as_mut()
            .expect("pipeline capacity exceeded")
            .push(id)
            .expect("pipeline capacity exceeded");
        self
    }
}
