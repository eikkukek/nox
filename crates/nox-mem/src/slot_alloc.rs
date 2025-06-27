use core::{
    any::TypeId,
    num::NonZero,
    ptr,
    marker::PhantomData,
};

use crate::{
    type_registery::TypeRegistery,
    allocator::Allocator,
    const_assert,
    const_fn::align_up,
    impl_inherent,
    size_of,
    align_of,
};

pub enum Error {
    InvalidSizeAlign { size: usize, align: usize, },
    AllocFailed { new_capacity: u32, },
    BadVersion { slot_version: u32, index_version: u32, },
}

use Error::{InvalidSizeAlign, AllocFailed};

pub type Result<T> = core::result::Result<T, Error>;

#[repr(C)]
struct Slot<const MAX_SIZE: usize>
{
    data: [u8; MAX_SIZE],
    version: u32,
    next_free_index: Option<NonZero<u32>>,
    type_index: u32,
}

impl_inherent! {
    Slot<MAX_SIZE: usize [const]> {

        fn empty(next_free_index: u32) -> Self {
            Self {
                data: [0u8; MAX_SIZE],
                version: 0,
                next_free_index: NonZero::new(next_free_index),
                type_index: 0,
            }
        }

        #[inline(always)]
        [const] fn pad_bytes(max_align: usize) -> usize {
            let total_size = size_of!(Self);
            align_up(total_size, max_align) - total_size
        }
    }
}

const_assert!(size_of!(TypeId) == 16);

const_assert!(size_of::<Slot<8>>() == 8 + 12 && align_of::<Slot<64>>() == 4);

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
    free_head: Option<u32>,
    allocator: &'alloc A,
    registry: &'static mut TypeRegistery,
}

impl<'alloc, const MAX_SIZE: usize, const MAX_ALIGN: usize, A: Allocator> SlotAlloc<'alloc, MAX_SIZE, MAX_ALIGN, A> {

    pub fn new(registry: &'static mut TypeRegistery, allocator: &'alloc A) -> Result<Self> {
        Self::max_size_align_ok()?;
        Ok(Self {
            data: ptr::dangling::<u8>() as _,
            capacity: 0,
            len: 0,
            free_head: None,
            allocator,
            registry,
        })
    }

    pub fn with_capacity(capacity: u32, registry: &'static mut TypeRegistery, allocator: &'alloc A) -> Result<Self>
    {
        Self::max_size_align_ok()?;
        let data = unsafe { allocator
            .allocate_raw(Self::slot_size() * capacity as usize, MAX_ALIGN)
            .ok_or(AllocFailed { new_capacity: capacity })?
            .as_ptr()
        };
        for i in 0..capacity - 1 {
            unsafe { *Self::get_slot(data, i) = Slot::empty(i + 1) }
        }
        unsafe { *Self::get_slot(data, capacity - 1) = Slot::empty(0); }
        Ok(Self {
            data,
            capacity,
            len: 0,
            free_head: Some(0),
            allocator,
            registry,
        })
    }

    pub fn insert<T: 'static>(&mut self, value: T) -> Result<Index<T>>
    {
        Self::size_align_ok(size_of!(T), align_of!(T))?;
        let Some(index) = self.free_head else {
            todo!()
        };
        let free_slot = unsafe { &mut *Self::get_slot(self.data, index) };
        self.free_head = free_slot.next_free_index.map_or(None, |i| Some(i.get()));
        free_slot.next_free_index = None;
        free_slot.type_index = self.registry.register::<T>();
        unsafe {
            free_slot.data
                .as_mut_ptr()
                .cast::<T>()
                .write(value)
        };
        self.len += 1;
        Ok(Index {
            version: NonZero::new(free_slot.version).unwrap(),
            index: index,
            _marker: PhantomData,
        })
    }

    pub fn delete<T>(&mut self, index: Index<T>) -> Result<T>
    {
        Self::size_align_ok(size_of!(T), align_of!(T))?;
        if index.index >= self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let free_slot = unsafe { &mut *Self::get_slot(self.data, index.index) };
        let index_version = index.version.get();
        if free_slot.version != index_version {
            panic!("stale index: slot version {}, index version {}", free_slot.version, index.version)
        }
        let value = unsafe {
            free_slot.data
                .as_ptr()
                .cast::<T>()
                .read()
        };
        free_slot.version += 1;
        if let Some(free_head) = self.free_head {
            if free_head != 0 {
                free_slot.next_free_index = NonZero::new(free_head);
                self.free_head = Some(index.index);
            }
            else {
                let slot = unsafe { &mut *Self::get_slot(self.data, 0) };
                free_slot.next_free_index = slot.next_free_index;
                slot.next_free_index = NonZero::new(index.index);
            }
        }
        else {
            self.free_head = Some(index.index)
        }
        self.len -= 1;
        Ok(value)
    }

    #[inline(always)]
    const fn max_size_align_ok() -> Result<()> {
        if 
            MAX_ALIGN.is_power_of_two() &&
            MAX_ALIGN >= 4
        {
            Ok(())
        }
        else
        {
            Err(InvalidSizeAlign { size: MAX_SIZE, align: MAX_ALIGN })
        }
    }

    #[inline(always)]
    const fn size_align_ok(size: usize, align: usize) -> Result<()> {
        if 
            size <= MAX_SIZE &&
            align.is_power_of_two() &&
            align <= MAX_ALIGN
        {
            Ok(())
        }
        else
        {
            Err(InvalidSizeAlign { size, align })
        }
    }

    #[inline(always)]
    const fn slot_size() -> usize {
        size_of::<Slot<MAX_SIZE>>() + Slot::<MAX_SIZE>::pad_bytes(MAX_ALIGN)
    }

    #[inline(always)]
    unsafe fn get_slot(data: *mut u8, index: u32) -> *mut Slot<MAX_SIZE> {
        unsafe { data.add(Self::slot_size() * index as usize) as *mut Slot<MAX_SIZE> }
    }
}
