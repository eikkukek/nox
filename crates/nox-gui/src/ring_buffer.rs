use core::ptr::NonNull;

use compact_str::format_compact;

use nox::{
    mem::{
        vec::Vec32,
        align_up,
        vec32,
    },
    gpu,
    error::*,
};

#[derive(Default, Clone, Copy, Debug)]
struct RingBufReg {
    head: usize,
    tail: usize,
}

fn ring_buf_reg(head: usize, tail: usize) -> RingBufReg {
    RingBufReg { head, tail }
}

pub struct RingBuf {
    gpu: gpu::Gpu,
    buffer: gpu::BufferId,
    size: usize,
    map: NonNull<u8>,
    desired_region_count: u32,
    frame_regions: Vec32<RingBufReg>,
    current_reg: RingBufReg,
    semaphore: gpu::TimelineSemaphoreId,
    semaphore_value: u64,
}

pub struct RingBufMem<T> {
    pub ptr: NonNull<T>,
    pub offset: u64,
}

impl RingBuf {

    pub fn new(
        gpu: gpu::Gpu,
        size: usize,
        usage: gpu::BufferUsages,
        desired_region_count: u32,
    ) -> Result<Self> {
        let mut buffer = Default::default();
        gpu.create_resources(
            [gpu::BufferCreateInfo::new(
                &mut buffer,
                size as gpu::DeviceSize,
                usage
            )],
            []
        )?;
        let mut semaphore = Default::default();
        gpu.create_timeline_semaphores(
            [(&mut semaphore, 0)]
        )?;
        let (map, size) = gpu.map_buffer(buffer)?;
        Ok(Self {
            gpu,
            buffer,
            map: NonNull::new(map).unwrap(),
            size,
            desired_region_count,
            frame_regions: vec32![],
            current_reg: Default::default(),
            semaphore,
            semaphore_value: 0,
        })
    }

    #[inline(always)]
    pub fn id(&self) -> gpu::BufferId {
        self.buffer
    }

    #[inline(always)]
    pub fn swapchain_recreated(&mut self, image_count: u32) {
        self.frame_regions.resize(
            image_count.min(self.desired_region_count),
            ring_buf_reg(0, 0),
        );
    }

    pub unsafe fn allocate<T>(
        &mut self,
        count: usize,
    ) -> Result<RingBufMem<T>>
    {
        let RingBufReg { head, tail } = self.current_reg;
        let size = count * size_of::<T>();
        let mut offset = align_up(tail, align_of::<T>());
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
            if self.gpu.wait_for_semaphores(
                &[(self.semaphore, self.semaphore_value)],
                self.gpu.device().frame_timeout(),
            )? {
                for reg in &mut self.frame_regions {
                    *reg = ring_buf_reg(0, 0);
                }
            } else {
                return Err(Error::just_context(format_compact!(
                    "frame timeout {} nanosecods at {}",
                    self.gpu.device().frame_timeout(), location!(),
                )))
            }
        }
        self.current_reg = ring_buf_reg(head, new_tail);
        Ok(RingBufMem {
            offset: offset as u64,
            ptr: unsafe { self.map.add(offset).cast() },
        })
    }

    pub fn finish_frame(
        &mut self,
    ) -> (gpu::TimelineSemaphoreId, u64)
    {
        self.frame_regions.pop();
        self.frame_regions.insert(0, self.current_reg);
        self.current_reg = ring_buf_reg(self.current_reg.tail, self.current_reg.tail);
        self.semaphore_value += 1;
        (self.semaphore, self.semaphore_value)
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
