use core::{
    slice,
    ptr::NonNull,
    marker::PhantomData,
    cmp::Ordering,
};

use nox_mem::{
    const_fn::align_up,
    vec_types::{self, GlobalVec, MemoryStrategy},
    Allocator,
    CapacityError,
    CapacityPolicy,
    impl_traits,
};

use super::{Iter, IterMut};

pub struct AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
{
    data: NonNull<u8>,
    capacity: usize,
    len: usize,
    allocator: &'alloc Alloc,
    _marker: PhantomData<(Key, Val)>,
    _cap_pol: PhantomData<CapacityPol>,
}

impl<'alloc, Key, Val, Alloc, CapacityPol> AllocMap<'alloc, Key, Val, Alloc, CapacityPol>
    where
        Key: PartialOrd,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
{
    pub fn new(allocator: &'alloc Alloc) -> Option<Self> {
        if !CapacityPol::can_grow() {
            return None
        }
        Some(Self {
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            allocator,
            _marker: PhantomData,
            _cap_pol: PhantomData,
        })
    }

    pub fn with_capacity(
        capacity: usize,
        allocator: &'alloc Alloc,
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
        let data = unsafe { allocator
            .allocate_raw(Self::alloc_size(true_capacity), Self::align())
            .ok_or_else(|| CapacityError::AllocFailed { new_capacity: true_capacity })? };
        Ok(Self{
            data,
            capacity: true_capacity,
            len: 0,
            allocator,
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
            return Err(CapacityError::FixedCapacity { capacity: self.capacity })
        }
        let new_capacity = match CapacityPol::grow(self.capacity, capacity) {
            Some(r) => r,
            None => return Err(CapacityError::InvalidReservation { current: self.capacity, requested: capacity }),
        };
        let tmp = match unsafe { self.allocator.allocate_raw(Self::alloc_size(new_capacity), Self::align()) } {
            Some(r) => r,
            None => return Err(CapacityError::AllocFailed { new_capacity }),
        };
        debug_assert!(self.len <= self.capacity);
        unsafe {
            GlobalVec::<Key>::move_elements(self.data.cast(), tmp.cast(), self.len);
            GlobalVec::<Val>::move_elements(
                self.data.add(Self::val_offset(self.capacity)).cast(),
                tmp.add(Self::val_offset(new_capacity)).cast(),
                self.len,
            );
        }
        if self.capacity != 0 {
            unsafe {
                self.allocator.free_raw(
                    self.data,
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

    pub fn insert_or_modify<Ins, Mod>(
        &mut self,
        key: Key,
        mut insert: Ins,
        mut modify: Mod,
    ) -> Result<Option<&mut Val>, CapacityError>
        where
            Ins: FnMut() -> Val,
            Mod: FnMut(&mut Val),
    {
        if self.len == 0 {
            let val = self.insert_internal(0, key, insert())?;
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
                Ordering::Equal => {
                    let elem = unsafe { self.data
                        .add(Self::val_offset(self.capacity))
                        .cast::<Val>()
                        .add(index)
                        .as_mut()
                    };
                    modify(
                        elem
                    );
                    return Ok(Some(elem))
                },
            }
        }
        let val = self.insert_internal(left, key, insert())?;
        Ok(Some(val))
    }

    pub fn get(&self, key: &Key) -> Option<&Val>
    {
        unsafe {
            Some(self.get_ptr(key)?.cast().as_ref())
        }
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Val>
    {
        unsafe {
            Some(self.get_ptr(key)?.cast().as_mut())
        }
    }

    pub fn clear(&mut self) {
        debug_assert!(self.len <= self.capacity);
        if self.capacity == 0 { return }
        unsafe {
            GlobalVec::<Key>::drop_in_place(self.data.cast(), self.len);
            GlobalVec::<Val>::drop_in_place(
                self.data.add(Self::val_offset(self.capacity)).cast(), self.len
            );
        }
        unsafe {
            self.allocator.free_raw(
                self.data,
                Self::alloc_size(self.capacity),
                Self::align()
            );
        }
        self.len = 0;
        self.capacity = 0;
        self.data = NonNull::dangling();
    }

    #[inline(always)]
    fn val_offset(capacity: usize) -> usize {
        let key_size = size_of::<Key>() * capacity;
        align_up(key_size, align_of::<Val>())
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
        unsafe { slice::from_raw_parts(self.data.cast().as_ptr(), self.len) }
    }

    #[inline]
    fn insert_internal(
        &mut self,
        index: usize,
        key: Key,
        value: Val,
    ) -> Result<&mut Val, CapacityError> {
        debug_assert!(self.len >= index);
        if self.len >= self.capacity {
            if self.capacity == 0 {
                self.reserve(2)?
            }
            else {
                self.reserve(self.capacity * 2)?
            }
        }
        unsafe {
            GlobalVec::<Key>::insert_element(self.data.cast(), key, index, self.len);
            let mut res = GlobalVec::<Val>::insert_element(
                self.data.add(Self::val_offset(self.capacity)).cast(),
                value,
                index,
                self.len
            );
            self.len += 1;
            Ok(res.as_mut())
        }
    }

    #[inline]
    unsafe fn get_ptr(&self, key: &Key) -> Option<NonNull<Val>> {
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

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Key, Val> {
        unsafe {
            let key_ptr = self.data.cast();
            let key_iter = vec_types::Iter::new(key_ptr, key_ptr.add(self.len));
            let val_ptr = self.data.add(Self::val_offset(self.capacity)).cast();
            let val_iter = vec_types::Iter::new(val_ptr, val_ptr.add(self.len));
            Iter::new(key_iter, val_iter)
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, Key, Val> {
        unsafe {
            let key_ptr = self.data.cast();
            let key_iter = vec_types::Iter::new(key_ptr, key_ptr.add(self.len));
            let val_ptr = self.data.add(Self::val_offset(self.capacity)).cast();
            let val_iter = vec_types::IterMut::new(val_ptr, val_ptr.add(self.len));
            IterMut::new(key_iter, val_iter)
        }
    }
}

impl_traits! {
    for AllocMap<'alloc, Key: PartialOrd, Val, Alloc: Allocator, CapacityPol: CapacityPolicy>
    Drop =>
        fn drop(&mut self) {
            self.clear();
        }
    ,
    IntoIterator &'map =>

        type Item = (&'map Key, &'map Val);
        type IntoIter = Iter<'map, Key, Val>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator &'map mut =>

        type Item = (&'map Key, &'map mut Val);
        type IntoIter = IterMut<'map, Key, Val>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
}
