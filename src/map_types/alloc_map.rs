use core::{
    slice,
    ptr::NonNull,
    cell::RefCell,
    marker::PhantomData,
    cmp::Ordering,
};

use crate::{
    allocator_traits::{Allocate, Free},
    utility::next_align,
    vec_types::{CapacityError, CapacityPolicy, MemoryStrategy, Iter, IterMut}
};

pub struct AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd + MemoryStrategy,
        Val: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    data: *mut u8,
    capacity: usize,
    len: usize,
    alloc: &'alloc RefCell<Alloc>,
    _marker: PhantomData<(Key, Val)>,
    _cap_pol: PhantomData<CapacityPol>,
}

impl<'alloc, Key, Val, Alloc, CapacityPol> AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd + MemoryStrategy,
        Val: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    pub fn new(alloc: &'alloc RefCell<Alloc>) -> Option<Self> {
        if !CapacityPol::can_grow() {
            return None
        }
        Some(Self {
            data: std::ptr::dangling::<Key>() as _,
            capacity: 0,
            len: 0,
            alloc,
            _marker: PhantomData,
            _cap_pol: PhantomData,
        })
    }

    pub fn with_capacity(
        capacity: usize,
        alloc: &'alloc RefCell<Alloc>
    ) -> Result<Self, CapacityError> {
        if capacity == 0 {
            return Err(CapacityError::InvalidReservation {
                current: 0, requested: 0
            })
        }
        let true_capacity =
            if CapacityPol::power_of_two() {
                capacity.next_power_of_two()
            }
            else {
                capacity
            };
        let mut a = alloc.borrow_mut();
        let data = unsafe { a
            .allocate_raw(Self::alloc_size(true_capacity), Self::align())
            .ok_or_else(|| CapacityError::AllocFailed)? }
            .as_ptr();
        Ok(Self{
            data,
            capacity: true_capacity,
            len: 0,
            alloc,
            _marker: PhantomData,
            _cap_pol: PhantomData,
        })
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve(&mut self, capacity: usize) -> Result<(), CapacityError> {
        if capacity <= self.capacity {
            return Ok(())
        }
        if !CapacityPol::can_grow() {
            return Err(CapacityError::Fixed)
        }
        let new_capacity = match CapacityPol::grow(self.capacity, capacity) {
            Some(r) => r,
            None => return Err(CapacityError::InvalidReservation { current: self.capacity, requested: capacity }),
        };
        let mut a = self.alloc.borrow_mut();
        let tmp = match unsafe { a.allocate_raw(Self::alloc_size(new_capacity), Self::align()) } {
            Some(r) => r.as_ptr(),
            None => return Err(CapacityError::AllocFailed),
        };
        debug_assert!(self.len <= self.capacity);
        unsafe {
            <Key as MemoryStrategy>::copy(self.data as _, tmp as _, self.len);
            <Val as MemoryStrategy>::copy(
                self.data.add(Self::val_offset(self.capacity)) as _,
                tmp.add(Self::val_offset(new_capacity)) as _,
                self.len,
            );
        }
        if self.capacity != 0 {
            unsafe {
                a.free_raw(
                    NonNull::new(self.data).unwrap(),
                    Self::alloc_size(self.capacity),
                    Self::align()
                );
            }
        }
        self.data = tmp;
        self.capacity = new_capacity;
        Ok(())
    }

    pub fn insert(
        &mut self,
        key: Key,
        value: Val,
    ) -> Result<Option<&mut Val>, CapacityError>
    {
        if self.len == 0 {
            let val = self.insert_internal(0, key, value)?;
            return Ok(Some(val))
        }
        let keys = self.keys_as_slice();
        let mut left: usize = 0;
        let mut right = self.len;
        while left < right {
            let index = left + (right - left) / 2;
            let Some(ord) = keys[index].partial_cmp(&key) else {
                return Ok(None)
            };
            match ord {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => return Ok(None),
            }
        }
        let val = self.insert_internal(left, key, value)?;
        Ok(Some(val))
    }

    pub fn get(&self, key: &Key) -> Option<&Val>
    {
        unsafe {
            let ptr = self.get_ptr(key)?;
            Some(&*ptr)
        }
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Val>
    {
        unsafe {
            let ptr = self.get_ptr(key)? as *mut Val;
            Some(&mut *ptr)
        }
    }

    #[inline(always)]
    fn val_offset(capacity: usize) -> usize {
        let key_size = size_of::<Key>() * capacity;
        next_align(key_size, align_of::<Val>())
    }

    #[inline(always)]
    fn alloc_size(capacity: usize) -> usize {
        Self::val_offset(capacity) + size_of::<Val>() * capacity
    }

    #[inline(always)]
    fn align() -> usize {
        align_of::<Key>().max(align_of::<Val>())
    }

    #[inline(always)]
    fn keys_as_slice(&self) -> &[Key] {
        unsafe { slice::from_raw_parts(self.data as _, self.len) }
    }

    #[inline]
    fn insert_internal(
        &mut self,
        index: usize,
        key: Key,
        value: Val,
    ) -> Result<&mut Val, CapacityError> {
        debug_assert!(self.len > index);
        if self.len >= self.capacity {
            if self.capacity == 0 {
                self.reserve(2)?
            }
            else {
                self.reserve(self.capacity * 2)?
            }
        }
        unsafe {
            <Key as MemoryStrategy>::insert(self.data as _, key, index, self.len);
            let res = <Val as MemoryStrategy>::insert(
                self.data.add(Self::val_offset(self.capacity)) as _,
                value,
                index,
                self.len
            );
            self.len += 1;
            Ok(&mut *res)
        }
    }

    unsafe fn get_ptr(&self, key: &Key) -> Option<*const Val> {
        let keys = self.keys_as_slice();
        let mut left: usize = 0;
        let mut right = self.len;
        while left < right {
            let index = left + (right - left) / 2;
            match keys[index].partial_cmp(&key)? {
                Ordering::Less => { left = index + 1; continue },
                Ordering::Greater => { right = index; continue },
                Ordering::Equal => {
                    return unsafe {
                        Some(self.data
                            .add(Self::val_offset(self.capacity))
                            .cast::<Val>()
                            .add(index)
                        )
                    }
                }
            }
        }
        None
    }

    pub fn iter(&self) -> Iter<'_, Val> {
        unsafe {
            let begin = self.data.add(Self::val_offset(self.capacity)) as _;
            Iter::new(begin, begin.add(self.len), PhantomData)
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Val> {
        unsafe {
            let begin = self.data.add(Self::val_offset(self.capacity)) as _;
            IterMut::new(begin, begin.add(self.len), PhantomData)
        }
    }
}

impl<'map, 'alloc, Key, Val, Alloc, CapacityPol> IntoIterator for
        &'map AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd + MemoryStrategy,
        Val: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{

    type Item = &'map Val;
    type IntoIter = Iter<'map, Val>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'map, 'alloc, Key, Val, Alloc, CapacityPol> IntoIterator for
        &'map mut AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd + MemoryStrategy,
        Val: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{

    type Item = &'map mut Val;
    type IntoIter = IterMut<'map, Val>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
