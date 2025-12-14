use core::ptr::NonNull;

use nox::{
    mem::{
        vec_types::{Vector, ArrayVec},
        align_up,
        align_of,
        size_of,
    },
    gpu,
};

use crate::error::*;

#[derive(Default, Clone, Copy, Debug)]
struct RingBufReg {
    head: usize,
    tail: usize,
}

fn ring_buf_reg(head: usize, tail: usize) -> RingBufReg {
    RingBufReg { head, tail }
}

pub struct RingBuf {
    buffer: gpu::BufferId,
    map: NonNull<u8>,
    current_reg: RingBufReg,
    frame_regions: ArrayVec<RingBufReg, {gpu::MAX_BUFFERED_FRAMES as usize}>,
    size: usize,
}

pub struct RingBufMem<T> {
    pub ptr: NonNull<T>,
    pub offset: u64,
}

impl RingBuf {

    pub fn new(
        buffer: gpu::BufferId,
        map: NonNull<u8>,
        buffered_frames: u32,
        size: usize,
    ) -> Result<Self> {
        Ok(Self {
            buffer,
            map,
            current_reg: Default::default(),
            frame_regions: ArrayVec
                ::with_len(Default::default(), buffered_frames as usize)
                .context("vec error")?,
            size,
        })
    }

    #[inline(always)]
    pub fn id(&self) -> gpu::BufferId {
        self.buffer
    }

    pub unsafe fn allocate<T>(
        &mut self,
        render_commands: &mut gpu::RenderCommands,
        count: usize,
    ) -> Result<RingBufMem<T>>
    {
        let RingBufReg { head, tail } = self.current_reg;
        let size = count * size_of!(T);
        let mut offset = align_up(tail, align_of!(T));
        let mut new_tail = offset + size;
        // wrapped around to current head
        if tail < head && new_tail > head {
            return Err(Error::just_context("ring buffer is out of memory"))
        }
        // wrap around
        if new_tail > self.size
        {
            offset = 0;
            new_tail = size;
        }
        let oldest_region = self.frame_regions.last().unwrap();
        if tail < oldest_region.tail && tail > oldest_region.head
        {
            if render_commands
                .wait_for_previous_frame(1_000_000_000)
                .context("failed to wait for previous frame")?
            {
                for reg in &mut self.frame_regions {
                    *reg = ring_buf_reg(0, 0);
                }
            } else {
                return Err(Error::just_context("frame timed out"))
            }
        }
        self.current_reg = ring_buf_reg(head, new_tail);
        Ok(RingBufMem {
            offset: offset as u64,
            ptr: unsafe { self.map.add(offset).cast() },
        })
    }

    pub fn finish_frame(&mut self) {
        self.frame_regions.pop();
        self.frame_regions.insert(0, self.current_reg).unwrap();
        self.current_reg = ring_buf_reg(self.current_reg.tail, self.current_reg.tail);
    }
}

impl<T> Default for RingBufMem<T> {

    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            offset: 0,
        }
    }
}
