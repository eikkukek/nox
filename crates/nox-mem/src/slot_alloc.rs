use core::{
    num::NonZero,
    ptr,
    marker::PhantomData,
};

use crate::{
    const_fn::max_usize,
    allocator::Allocator,
    size_of,
    align_of,
    impl_inherent,
    const_assert,
};

pub enum Error {
    InvalidSizeAlign { size: usize, align: usize, },
    AllocFailed { new_capacity: u32, },
    BadVersion { slot_version: u32, index_version: u32, },
}

use Error::{InvalidSizeAlign, AllocFailed, BadVersion};

pub type Result<T> = core::result::Result<T, Error>;

#[repr(packed)]
struct Slot<const MAX_SIZE: usize>
{
    data: [u8; MAX_SIZE],
    version: u32,
    next_free_index: Option<NonZero<u32>>,
}

impl_inherent! {
    Slot<MAX_SIZE: usize [const]> {

        fn empty(next_free_index: u32) -> Self {
            Self {
                data: [0u8; MAX_SIZE],
                version: 0,
                next_free_index: NonZero::new(next_free_index),
            }
        }

        [const] fn pad_bytes(max_align: usize) -> usize {
            max_usize(max_align, 8) - 8
        }
    }
}

const_assert!(size_of::<Slot<64>>() == 64 + 8 && align_of::<Slot<64>>() == 1);

pub struct Index<T> {
    version: NonZero<u32>,
    index: u32,
    _marker: PhantomData<T>,
}

const_assert!(size_of::<Option<Index<()>>>() == 8);

pub struct SlotAlloc<'alloc, const MAX_SIZE: usize, const MAX_ALIGN: usize, A: Allocator> {
    data: *mut u8,
    capacity: u32,
    len: u32,
    free_head: u32,
    allocator: &'alloc A,
}

impl_inherent! {
    SlotAlloc<'alloc, MAX_SIZE: usize [const], MAX_ALIGN: usize [const], A: Allocator> {

        [pub] fn new(allocator: &'alloc A) -> Result<Self> {
            Self::max_size_align_ok()?;
            Ok(Self {
                data: ptr::dangling::<u8>() as _,
                capacity: 0,
                len: 0,
                free_head: 0,
                allocator: allocator,
            })
        }

        [pub] fn with_capacity(capacity: u32, allocator: &'alloc A) -> Result<Self>
        {
            Self::max_size_align_ok()?;
            let data = unsafe {
                    allocator.allocate_raw(Self::slot_size() * capacity as usize, MAX_ALIGN)
                }
                .ok_or(AllocFailed { new_capacity: capacity })?
                .as_ptr();
            for i in 0..capacity - 1 {
                unsafe { *Self::get_slot(data, i) = Slot::empty(i + 1) }
            }
            unsafe { *Self::get_slot(data, capacity - 1) = Slot::empty(0); }
            Ok(Self {
                data,
                capacity,
                len: 0,
                free_head: 0,
                allocator,
            })
        }

        [pub] fn insert<T>(&mut self, value: T) -> Result<Index<T>>
        {
            Self::size_align_ok(size_of!(T), align_of!(T))?;
            let index = self.free_head;
            if index == self.capacity {
                todo!()
            }
            let ptr = unsafe { Self::get_slot(self.data, index) };
            let mut free_slot = unsafe { ptr.read() };
            free_slot.version += 1;
            self.free_head = free_slot.next_free_index.map_or(self.capacity, |i| i.get());
            unsafe { Self::write_to_slot(ptr as _, value, free_slot.version); }
            self.len += 1;
            Ok(Index {
                version: NonZero::new(free_slot.version).unwrap(),
                index: index,
                _marker: PhantomData,
            })
        }

        [pub] fn delete<T>(&mut self, index: Index<T>) -> Result<T>
        {
            Self::size_align_ok(size_of!(T), align_of!(T))?;
            if index.index >= self.capacity {
                panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
            }
            let ptr = unsafe { Self::get_slot(self.data, index.index) };
            let mut free_slot = unsafe { ptr.read() };
            let index_version = index.version.get();
            if free_slot.version != index_version {
                return Err(BadVersion { slot_version: free_slot.version, index_version })
            }
            let value = unsafe { Self::read_slot_value(ptr as _) };
            free_slot.version += 1;
            if self.free_head != 0 {
                free_slot.next_free_index = NonZero::new(self.free_head);
                self.free_head = index.index;
            }
            else {
                let other_ptr = unsafe { Self::get_slot(self.data, 0) };
                let mut slot = unsafe { other_ptr.read() };
                free_slot.next_free_index = slot.next_free_index;
                slot.next_free_index = NonZero::new(index.index);
                unsafe {
                    other_ptr.write(slot);
                }
            }
            unsafe {
                ptr.write(free_slot);
            }
            self.len -= 1;
            Ok(value)
        }

        #[inline(always)]
        [const] fn max_size_align_ok() -> Result<()> {
            if 
                MAX_SIZE.is_power_of_two() &&
                MAX_ALIGN.is_power_of_two() &&
                MAX_SIZE >= 4
            {
                Ok(())
            }
            else
            {
                Err(InvalidSizeAlign { size: MAX_SIZE, align: MAX_ALIGN })
            }
        }

        #[inline(always)]
        [const] fn size_align_ok(size: usize, align: usize) -> Result<()> {
            if 
                size.is_power_of_two() &&
                align.is_power_of_two() &&
                size >= 4
            {
                Ok(())
            }
            else
            {
                Err(InvalidSizeAlign { size, align })
            }
        }

        #[inline(always)]
        [const] fn slot_size() -> usize {
            size_of::<Slot<MAX_SIZE>>() + Slot::<MAX_SIZE>::pad_bytes(MAX_ALIGN)
        }

        #[inline(always)]
        [unsafe] fn get_slot(data: *mut u8, index: u32) -> *mut Slot<MAX_SIZE> {
            unsafe { data.add(Self::slot_size() * index as usize) as *mut Slot<MAX_SIZE> }
        }

        #[inline(always)]
        [unsafe] fn write_to_slot<T>(ptr: *mut u8, value: T, version: u32) {
            unsafe {
                ptr
                    .cast::<T>()
                    .write(value);
                ptr
                    .add(MAX_SIZE)
                    .cast::<[u32; 2]>()
                    .write([version, 0]);
            }
        }

        #[inline(always)]
        [unsafe] fn read_slot_value<T>(ptr: *mut u8) -> T {
            unsafe {
                ptr
                    .cast::<T>()
                    .read()
            }
        }
    }
}
