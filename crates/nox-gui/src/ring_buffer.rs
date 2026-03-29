use core::{
    ptr::NonNull,
    time::Duration,
};

use compact_str::format_compact;

use nox::{
    error::*, gpu, mem::{
        align_down_u64, align_up, align_up_u64, vec::Vec32, vec32
    }
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
    buffer_id: gpu::BufferId,
    map: gpu::MemoryMap,
    frame_regions: Vec32<RingBufReg>,
    current_reg: RingBufReg,
    semaphore: gpu::TimelineSemaphoreId,
    semaphore_value: u64,
}

pub struct RingBufMem<T> {
    pub ptr: NonNull<T>,
    pub offset: gpu::DeviceSize,
    pub size: gpu::DeviceSize,
}

impl RingBuf {

    pub fn new(
        gpu: gpu::Gpu,
        memory_binder: &dyn gpu::MemoryBinder,
        size: usize,
        usage: gpu::BufferUsages,
    ) -> Result<Self> {
        let mut buffer_id = Default::default();
        gpu.create_resources(
            [gpu::BufferCreateInfo::new(
                &mut buffer_id,
                memory_binder,
                size as gpu::DeviceSize,
                usage
            ).unwrap()],
            []
        )?;
        let mut semaphore = Default::default();
        gpu.create_timeline_semaphores(
            [(&mut semaphore, 0)]
        )?;
        let map = gpu.map_buffer(buffer_id)?;
        Ok(Self {
            gpu,
            buffer_id,
            map,
            frame_regions: vec32![],
            current_reg: Default::default(),
            semaphore,
            semaphore_value: 0,
        })
    }

    #[inline(always)]
    pub fn id(&self) -> gpu::BufferId {
        self.buffer_id
    }

    #[inline(always)]
    pub fn allocate_frame_regions(&mut self, image_count: u32) {
        self.frame_regions.clear();
        self.frame_regions.resize(
            image_count.min(self.gpu.desired_buffered_frames()),
            ring_buf_reg(0, 0),
        );
    }

    pub fn allocate<T>(
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
        if new_tail > self.map.size
        {
            offset = 0;
            new_tail = size;
        }
        let oldest_region = self.frame_regions.last().unwrap();
        if tail < oldest_region.tail && tail > oldest_region.head
        {
            if self.gpu.wait_for_semaphores(
                &[(self.semaphore, self.semaphore_value)],
                Duration::from_nanos(self.gpu.device().frame_timeout()),
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
            ptr: NonNull::new(unsafe { self.map.map.add(offset).cast() }).unwrap(),
            offset: offset as gpu::DeviceSize,
            size: size as gpu::DeviceSize,
        })
    }

    pub fn finish_frame(
        &mut self,
    ) -> Result<(gpu::TimelineSemaphoreId, u64)>
    {
        self.frame_regions.pop();
        self.frame_regions.insert(0, self.current_reg);
        if !self.map.is_coherent {
            let atom_size = self.gpu.device_limits().non_coherent_atom_size();
            if self.current_reg.tail < self.current_reg.head {
                let mut offset = self.current_reg.head as gpu::DeviceSize;
                let add_size;
                offset = {
                    let tmp = align_down_u64(offset, atom_size);
                    add_size = offset - tmp;
                    tmp
                };
                let size1 = (self.map.size - self.current_reg.head) as gpu::DeviceSize + add_size;
                let size2 = align_up_u64(
                    self.current_reg.tail as gpu::DeviceSize,
                    atom_size,
                );
                self.gpu.flush_mapped_memory_ranges(&[
                    gpu::MappedBufferMemoryRange {
                        buffer_id: self.buffer_id,
                        offset,
                        size: size1,
                    },
                    gpu::MappedBufferMemoryRange {
                        buffer_id: self.buffer_id,
                        offset: 0,
                        size: size2,
                    },
                ])?;
            } else {
                let mut offset = self.current_reg.head as gpu::DeviceSize;
                let add_size;
                offset = {
                    let tmp = align_down_u64(offset, atom_size);
                    add_size = offset - tmp;
                    tmp
                };
                let size = align_up_u64(
                    (self.current_reg.tail - self.current_reg.head) as gpu::DeviceSize + add_size,
                    atom_size,
                ).min(self.map.size as gpu::DeviceSize - offset);
                self.gpu.flush_mapped_memory_ranges(&[
                    gpu::MappedBufferMemoryRange {
                        buffer_id: self.buffer_id,
                        offset,
                        size,
                    }
                ])?;
            }
        }
        self.current_reg = ring_buf_reg(self.current_reg.tail, self.current_reg.tail);
        self.semaphore_value += 1;
        Ok((self.semaphore, self.semaphore_value))
    }
}

impl<T> Default for RingBufMem<T> {

    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            offset: 0,
            size: 0,
        }
    }
}

impl Drop for RingBuf {

    fn drop(&mut self) {
        self.gpu.destroy_resources([self.buffer_id], []).ok();
    }
}
