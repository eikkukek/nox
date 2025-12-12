use super::*;

use ash::vk;

use compact_str::format_compact;

use nox_mem::{vec_types::{FixedVec, Vector}};

use nox_alloc::arena_alloc::ArenaAlloc;

use crate::dev::{
    error::{Error, Context, Location, location, caller, Tracked, ErrorContext},
    has_not_bits,
};

use super::ResourceFlags;

use crate::gpu::*;

pub(super) struct Pass<'a> {
    pub id: PassId,
    pub reads: FixedVec<'a, ReadInfo, ArenaAlloc>,
    pub writes: FixedVec<'a, WriteInfo, ArenaAlloc>,
    pub wait_semaphores: FixedVec<'a, (TimelineSemaphoreId, u64, PipelineStage), ArenaAlloc>,
    pub signal_semaphores: FixedVec<'a, (TimelineSemaphoreId, u64), ArenaAlloc>,
    pub depth_write: Option<(bool, WriteInfo)>,
    pub render_area: Option<vk::Rect2D>,
    pub msaa_samples: MSAA,
    loc: Location,
}

impl<'a> Pass<'a> {
    
    pub fn new(
        id: PassId,
        info: PassInfo,
        alloc: &'a ArenaAlloc,
        loc: Location,
    ) -> Result<Self> {
        let reads = FixedVec
            ::with_capacity(info.max_reads as usize, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        let writes = FixedVec
            ::with_capacity(info.max_color_writes as usize, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        let signal_semaphores = FixedVec
            ::with_capacity(info.signal_semaphores as usize, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        let wait_semaphores = FixedVec
            ::with_capacity(info.wait_semaphores as usize, alloc)
            .context_with(|| ErrorContext::VecError(location!()))?;
        Ok(Self {
            id,
            reads,
            writes,
            signal_semaphores,
            wait_semaphores,
            depth_write: None,
            render_area: None,
            msaa_samples: info.msaa_samples,
            loc,
        })
    }

    pub fn validate(&self, alloc: &ArenaAlloc) -> Result<Option<Error>> {

        enum WriteAccess<'a> {
            Whole,
            Partial(FixedVec<'a, ImageSubresourceRangeInfo, ArenaAlloc>),
        }

        if self.writes.len() != 0 {
            let reads = &self.reads;
            let writes = &self.writes;
            let mut write_ranges = FixedVec::<(ResourceId, WriteAccess), ArenaAlloc>
                ::with_capacity(writes.len(), alloc)
                .context_with(|| ErrorContext::VecError(location!()))?;
            for write in writes {

                let id = write.id;

                let Some(current_range) = write.range else {
                    if reads.iter().any(|r| r.id == id) ||
                        write_ranges.iter().any(|(v_id, _)| *v_id == id)
                    {
                        return Ok(Some(Error::just_context(format_compact!(
                            "write at {} has overlapping ranges while processing pass at {} (whole write overlaps with a read or another write)",
                            write.location_or_this(), self.location_or_this(),
                        ))))
                    }
                    write_ranges.push((id, WriteAccess::Whole)).unwrap();
                    continue
                };

                if let Some((_, access)) = write_ranges
                    .iter_mut()
                    .find(|(v_id, _)| *v_id == id)
                {
                    match access {
                        WriteAccess::Whole => {
                            return Ok(Some(Error::just_context(format_compact!(
                                "write at {} has overlapping ranges while processing pass at {} (whole write overlaps with another write)",
                                write.location_or_this(), self.location_or_this(),
                            ))))
                        }
                        WriteAccess::Partial(ranges) => {
                            for range in ranges.iter() {
                                if range.overlaps(current_range.subresource_info) {
                                    return Ok(Some(Error::just_context(format_compact!(
                                        "write at {} has overlapping ranges while processing pass at {} (partial write overlaps with another partial write)",
                                        write.location_or_this(), self.location_or_this(),
                                    ))))
                                }
                            }
                            for range in reads
                                .iter()
                                .filter(|read| read.id == id)
                                .map(|read| read.range)
                            {
                                if let Some(range) = range {
                                    if range.subresource_info.overlaps(current_range.subresource_info) {
                                        return Ok(Some(Error::just_context(format_compact!(
                                            "write at {} has overlapping ranges while processing pass at {} (partial write overlaps with a partial read)",
                                            write.location_or_this(), self.location_or_this(),
                                        ))))
                                    }
                                } else {
                                    return Ok(Some(Error::just_context(format_compact!(
                                        "write at {} has overlapping ranges while processing pass at {} (partial write overlaps with a whole read)",
                                        write.location_or_this(), self.location_or_this(),
                                    ))))
                                }
                            }
                            ranges.push(current_range.subresource_info).unwrap();
                        },
                    }
                }
                else {
                    let mut ranges = FixedVec
                        ::with_capacity(writes.len(), alloc)
                        .context_with(|| ErrorContext::VecError(location!()))?;
                    ranges.push(current_range.subresource_info).unwrap();
                    write_ranges.push((id, WriteAccess::Partial(ranges))).unwrap();
                }
            }
        }
        Ok(None)
    }
}

impl<'a> Tracked for Pass<'a> {

    #[inline(always)]
    fn location(&self) -> Option<Location> {
        Some(self.loc)
    }
}

pub struct PassBuilder<'a, 'b> {
    pub(super) pass: &'a mut Pass<'b>,
}

impl<'a, 'b> PassBuilder<'a, 'b> {

    #[track_caller]
    pub fn with_read(&mut self, read_info: ReadInfo) -> Result<&mut Self> {
        if has_not_bits!(read_info.id.flags, ResourceFlags::Sampleable) {
            return Err(Error::just_context("image read must be sampleable")
            ).context_with(|| ErrorContext::EventError(caller!()))
        }
        self.pass.reads
            .push(read_info)
            .context("read capacity exceeded")
            .context_with(|| ErrorContext::EventError(caller!()))?;
        Ok(self)
    }

    #[track_caller]
    pub fn with_write(&mut self, write: impl Into<WriteInfo>) -> Result<&mut Self> {
        let write = write.into();
        if write.samples() != self.pass.msaa_samples {
            return Err(Error::just_context(
                format_compact!("write MSAA sample count {} must match pass sample count {}",
                    write.samples(), self.pass.msaa_samples,
                )
            )).context(ErrorContext::EventError(caller!()))
        }
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|r| r.id) {
                if resolve.samples() != MSAA::X1 {
                    return Err(Error::just_context(
                        format_compact!("write resolve image sample count must be 1, given sample count was {}",
                            resolve.samples(),
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
                if write.id.format != resolve.format {
                    return Err(Error::just_context(
                        format_compact!("write resolve image format {:?} must be the same as the main image format {:?}",
                            resolve.samples(), write.id.format,
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
            }
        }
        self.pass.writes
            .push(write)
            .context("write capacity exceeded")
            .context_with(|| ErrorContext::EventError(caller!()))?;
        Ok(self)
    }

    #[track_caller]
    pub fn with_depth_write(&mut self, write: impl Into<WriteInfo>) -> Result<&mut Self> {
        let write = write.into();
        if write.samples() != self.pass.msaa_samples {
            return Err(Error::just_context(
                format_compact!("write MSAA sample count {} must match pass sample count {}",
                    write.samples(), self.pass.msaa_samples,
                )
            )).context(ErrorContext::EventError(caller!()))
        }
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|r| r.id) {
                if resolve.samples() != MSAA::X1 {
                    return Err(Error::just_context(
                        format_compact!("write resolve image sample count must be 1, given sample count was {}",
                            resolve.samples(),
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
                if write.id.format != resolve.format {
                    return Err(Error::just_context(
                        format_compact!(
                            "write resolve image format {:?} must be the same as the main image format {:?}",
                            resolve.samples(), write.id.format,
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
            }
        }
        self.pass.depth_write = Some((false, write));
        Ok(self)
    }

    #[track_caller]
    pub fn with_depth_stencil_write(&mut self, write: impl Into<WriteInfo>) -> Result<&mut Self> {
        let write = write.into();
        if write.samples() != self.pass.msaa_samples {
            return Err(Error::just_context(
                format_compact!("write MSAA sample count {} must match pass sample count {}",
                    write.samples(), self.pass.msaa_samples,
                )
            )).context(ErrorContext::EventError(caller!()))
        }
        if write.samples() != MSAA::X1 {
            if let Some(resolve) = write.resolve.map(|r| r.id) {
                if resolve.samples() != MSAA::X1 {
                    return Err(Error::just_context(
                        format_compact!("write resolve image sample count must be 1, given sample count was {}",
                            resolve.samples(),
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
                if write.id.format != resolve.format {
                    return Err(Error::just_context(
                        format_compact!(
                            "write resolve image format {:?} must be the same as the main image format {:?}",
                            resolve.samples(), write.id.format,
                        )
                    )).context(ErrorContext::EventError(caller!()))
                }
            }
        }
        self.pass.depth_write = Some((true, write));
        Ok(self)
    }

    pub fn with_render_area(&mut self, render_area: RenderArea) -> &mut Self {
        self.pass.render_area = Some(render_area.into());
        self
    }

    #[track_caller]
    pub fn with_wait_semaphore(
        &mut self,
        id: TimelineSemaphoreId,
        value: u64,
        stage: PipelineStage
    ) -> Result<&mut Self>
    {
        self.pass.wait_semaphores
            .push((id, value, stage))
            .context("wait semaphore capacity exceeded")
            .context_with(|| ErrorContext::EventError(caller!()))?;
        Ok(self)
    }

    #[track_caller]
    pub fn with_signal_semaphore(
        &mut self,
        id: TimelineSemaphoreId,
        value: u64
    ) -> Result<&mut Self> {
        self.pass.signal_semaphores
            .push((id, value))
            .context("signal semaphore capacity exceeded")
            .context_with(|| ErrorContext::EventError(caller!()))?;
        Ok(self)
    }
}
