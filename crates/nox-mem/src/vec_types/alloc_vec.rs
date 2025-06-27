use core::{
    marker::PhantomData,
    ops::{Index, IndexMut, Deref},
    ptr::NonNull,
    slice,
};

use std::ops::DerefMut;

use crate::{
    allocator::Allocator,
    capacity_policy::{CapacityPolicy, Dyn, Fixed},
    conditional::{Conditional, True, False},
    errors::CapacityError,
    option_alloc::OptionAlloc,
    global_alloc::{GlobalAlloc, GLOBAL_ALLOC},
    size_of,
    const_assert,
    impl_traits,
};

use super::{
    Vector,
    MemoryStrategy,
    CloneStrategy,
    Iter,
    IterMut,
};

use CapacityError::{FixedCapacity, InvalidReservation, AllocFailed, ZeroSizedElement};

pub struct AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{
    data: NonNull<T>,
    capacity: usize,
    len: usize,
    alloc: OptionAlloc<'alloc, Alloc>,
    _markers: PhantomData<(CapacityPol, IsGlobal)>,
}

pub type AllocVecImpl<'alloc, T, Alloc, CapacityPol> = AllocVec<'alloc, T, Alloc, CapacityPol, False>;
pub type DynVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, Dyn, False>;
pub type FixedVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, Fixed, False>;
pub type GlobalVec<T> = AllocVec<'static, T, GlobalAlloc, Dyn, True>;

const_assert!(size_of!(GlobalVec<u32>) == size_of!(Option<GlobalVec<u32>>));

impl<'alloc, T, Alloc, CapacityPol> AllocVec<'alloc, T, Alloc, CapacityPol, False>
    where
        CapacityPol: CapacityPolicy,
        Alloc: Allocator,
{

    pub fn new(alloc: &'alloc Alloc) -> Option<Self> {
        if !CapacityPol::can_grow() {
            return None
        }
        Some(Self {
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::Some(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_no_alloc() -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::None,
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: usize,
        alloc: &'alloc Alloc,
    ) -> Result<Self, CapacityError> {
        if capacity == 0 {
            return Err(InvalidReservation {
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
        let data = unsafe { alloc
            .allocate_uninit(true_capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: true_capacity }
                }
            })?
        };
        Ok(Self {
            data,
            capacity: true_capacity,
            len: 0,
            alloc: OptionAlloc::Some(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_len(
        len: usize,
        value: T,
        alloc: &'alloc Alloc,
    ) -> Result<Self, CapacityError>
        where
            T: Clone
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { alloc
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(value.clone()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_len_with<F: FnMut() -> T>(
        len: usize,
        mut f: F,
        alloc: &'alloc Alloc,
    ) -> Result<Self, CapacityError>
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { alloc
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(f()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(alloc),
            _markers: PhantomData,
        })
    }
}

impl<T> GlobalVec<T> {

    pub fn new() -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: usize,
    ) -> Result<Self, CapacityError> {
        if capacity == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let true_capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                capacity.next_power_of_two()
            }
            else {
                capacity
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(true_capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: true_capacity }
                }
            })?
        };
        Ok(Self {
            data,
            capacity: true_capacity,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }

    pub fn with_len(
        len: usize,
        value: T,
    ) -> Result<Self, CapacityError>
        where
            T: Clone
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(value.clone()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }

    pub fn with_len_with<F>(
        len: usize,
        mut f: F,
    ) -> Result<Self, CapacityError>
        where
            F: FnMut() -> T,
    {
        if len == 0 {
            return Err(InvalidReservation {
                current: 0, requested: 0
            })
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?
        };
        for i in 0..len {
            unsafe { data.add(i).write(f()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        })
    }
}

impl<'alloc, T, Alloc, CapacityPol, IsGlobal> Vector<T> for AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{

    type CapacityPol = CapacityPol;

    type Iter<'a> = Iter<'a, T>
        where T: 'a, Self: 'a;

    type IterMut<'a> = IterMut<'a, T>
        where T: 'a, Self: 'a;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_ptr()
    }

    #[inline(always)]
    fn as_non_null(&self) -> NonNull<T> {
        self.data
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len) }
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, len: usize) {
        if len > self.capacity { panic!("len was larger than capacity") }
        self.len = len;
    }

    fn reserve(&mut self, capacity: usize) -> Result<(), CapacityError>
    {
        if capacity <= self.capacity {
            return Ok(())
        }
        if !CapacityPol::can_grow() {
            return Err(FixedCapacity { capacity: self.capacity })
        }
        let new_capacity = match CapacityPol::grow(self.capacity, capacity) {
            Some(r) => r,
            None => return Err(InvalidReservation { current: self.capacity, requested: capacity }),
        };
        let tmp = match unsafe { self.alloc.allocate_uninit(new_capacity) } {
            Some(r) => r,
            None => return Err(
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity }
                }
            ),
        };
        debug_assert!(self.len <= self.capacity);
        unsafe {
            Self::move_elements(self.data, tmp, self.len);
        }
        if self.capacity != 0 {
            unsafe { self.alloc.free_uninit(self.data, self.capacity); }
        }
        self.data = tmp;
        self.capacity = new_capacity;
        Ok(())
    }

    fn resize(&mut self, len: usize, value: T) -> Result<(), CapacityError> 
        where
            T: Clone
    {
        if len > self.capacity {
            self.reserve(len)?
        }
        if len > self.len {
            for i in self.len..len {
                unsafe { self.data.add(i).write(value.clone()) }
            }
        }
        else if len < self.len {
            unsafe {
                Self::drop_in_place(self.data.add(len), self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn resize_with<F>(&mut self, len: usize, mut f: F) -> Result<(), CapacityError>
        where
            F: FnMut() -> T
    {
        if len > self.capacity {
            self.reserve(len)?
        }
        if len > self.len {
            for i in self.len..len {
                unsafe { self.data.add(i).write(f()) }
            }
        }
        else if len < self.len {
            unsafe {
                Self::drop_in_place(self.data.add(len), self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, value: T) -> Result<&mut T, CapacityError> {
        if self.len >= self.capacity {
            if self.capacity == 0 {
                self.reserve(2)?
            }
            else {
                self.reserve(self.capacity * 2)?
            }
        }
        let mut ptr = unsafe { self.data.add(self.len) };
        unsafe { ptr.write(value) };
        self.len += 1;
        Ok(unsafe { ptr.as_mut() })
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        let ptr = unsafe { self.data.add(self.len) };
        self.len -= 1;
        Some(unsafe { ptr.read() })
    }

    #[inline(always)]
    fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(
                    self.data.add(self.len - 1).as_ref()
                )
            }
        }
    }

    #[inline(always)]
    fn back_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(
                    self.data.add(self.len - 1).as_mut()
                )
            }
        }
    }

    fn insert(&mut self, value: T, index: usize) -> Result<&mut T, CapacityError> {
        if index > self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len >= self.capacity {
            if self.capacity == 0 {
                self.reserve(2)?
            }
            else {
                self.reserve(self.capacity * 2)?
            }
        }
        unsafe {
            let mut ptr = Self::insert_element(self.data, value, index, self.len);
            self.len += 1;
            Ok(ptr.as_mut())
        }
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index == self.len { debug_assert!(false); return None }
        let removed = unsafe { self.data.add(index).read() };
        for i in index..self.len - 1 {
            unsafe { self.data.add(i).write(
                self.data.add(i + 1).read()
            )}
        }
        self.len -= 1;
        Some(removed)
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index == self.len { return None }
        let removed = unsafe { self.data.add(index).read() };
        self.len -= 1;
        if index != self.len {
            unsafe { self.data.add(index).write(self.data.add(self.len).read()) }
        }
        Some(removed)
    }

    fn clear(&mut self) {
        debug_assert!(self.len <= self.capacity);
        if self.capacity == 0 { return }
        unsafe {
            Self::drop_in_place(self.data, self.len);
        }
        unsafe { self.alloc.free_uninit(
            self.data,
            self.capacity)
        }
        self.len = 0;
        self.capacity = 0;
        self.data = NonNull::dangling();
    }

    fn clone_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            T: Clone,
            V: Vector<T>,
    {
        if self.capacity < from.len() {
            if !CapacityPol::can_grow() {
                return Err(FixedCapacity { capacity: self.capacity })
            }
            self.clear();
            self.reserve(from.len())?
        }
        else {
            unsafe { Self::drop_in_place(self.data, self.len); }
            self.len = 0;
        }
        unsafe { V::clone_elements(from.as_non_null(), self.data, from.len()); }
        self.len = from.len();
        Ok(())
    }

    fn move_from<V>(&mut self, from: &mut V) -> Result<(), CapacityError>
        where
            V: Vector<T>,
    {
        if self.capacity < from.len() {
            self.clear();
            self.reserve(from.len())?
        }
        else {
            unsafe { Self::drop_in_place(self.data, self.len); }
            self.len = 0;
        }
        unsafe { V::move_elements(from.as_non_null(), self.data, from.len()); }
        self.len = from.len();
        unsafe { from.set_len(0); }
        Ok(())
    }

    fn contains(&self, value: &T) -> bool
        where
            T: PartialEq
    {
        for i in 0..self.len {
            if unsafe { self.data.add(i).as_ref() == value } {
                return true 
            }
        }
        return false
    }

    #[inline(always)]
    fn iter(&self) -> Iter<'_, T> {
        unsafe {
            let ptr = self.data;
            let end = self.data.add(self.len);
            Iter::new(ptr, end)
        }
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            let ptr = self.data;
            let end = self.data.add(self.len);
            IterMut::new(ptr, end)
        }
    }
}

impl_traits!{
    for AllocVec<'alloc, T, Alloc: Allocator, CapacityPol: CapacityPolicy, IsGlobal: Conditional>
    Drop =>

        #[inline(always)]
        fn drop(&mut self) -> () {
            self.clear()
        }
    ,
    Index<usize> =>

        type Output = T;

        #[inline(always)]
        fn index(&self, index: usize) -> &Self::Output {
            if index >= self.len {
                panic!("index {} out of bounds for length {}", index, self.len)
            }
            unsafe { self.data.add(index).as_ref() }
        }
    ,
    IndexMut<usize> =>

        #[inline(always)]
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            if index >= self.len {
                panic!("index {} out of bounds for length {}", index, self.len)
            }
            unsafe { self.data.add(index).as_mut() }
        }
    ,
    AsRef<[T]> =>

        #[inline(always)]
        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    ,
    AsMut<[T]> =>

        #[inline(always)]
        fn as_mut(&mut self) -> &mut [T] {
            self.as_mut_slice()
        }
    ,
    Deref =>

        type Target = [T];

        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            self.as_ref()
        }
    ,
    DerefMut =>

        #[inline(always)]
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.as_mut()
        }
    ,
    IntoIterator &'vec =>

        type Item = &'vec T;
        type IntoIter = Iter<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator &'vec mut =>

        type Item = &'vec mut T;
        type IntoIter = IterMut<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
}
