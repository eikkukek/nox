use core::{
    cell::RefCell,
    marker::PhantomData,
    ops::{Index, IndexMut},
    mem::MaybeUninit,
    ptr::NonNull,
    slice,
};

use crate::allocator_traits::{Allocate, Free};

use super::{
    CapacityError, CapacityPolicy, Vector, MemoryStrategy, DuplicateStrategy, Iter, IterMut,
};

pub struct AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    data: *mut T,
    capacity: usize,
    len: usize,
    alloc: &'alloc RefCell<Alloc>,
    _cap_pol: PhantomData<CapacityPol>,
}

impl<'alloc, T, Alloc, CapacityPol> AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{

    pub fn new(alloc: &'alloc RefCell<Alloc>) -> Option<Self> {
        if !CapacityPol::can_grow() {
            None
        }
        else {
            Some(Self {
                data: core::ptr::dangling::<T>() as *mut T,
                capacity: 0,
                len: 0,
                alloc,
                _cap_pol: PhantomData,
            })
        }
    }

    pub fn with_capacity(
        capacity: usize,
        alloc: &'alloc RefCell<Alloc>,
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
        let data: *mut T = unsafe { a
            .allocate_uninit(true_capacity)
            .ok_or_else(|| CapacityError::AllocFailed { new_capacity: true_capacity })? }
            .as_ptr();
        Ok(Self {
            data,
            capacity: true_capacity,
            len: 0,
            alloc,
            _cap_pol: PhantomData,
        })
    }
}

impl<'alloc, T, Alloc, CapacityPol> Vector<T> for AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
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
    fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.data as _
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T> {
        self.data as _
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    fn reserve(&mut self, capacity: usize) -> Result<(), CapacityError>
    {
        if capacity <= self.capacity {
            return Ok(())
        }
        if !CapacityPol::can_grow() {
            return Err(CapacityError::Fixed { capacity: self.capacity })
        }
        let new_capacity = match CapacityPol::grow(self.capacity, capacity) {
            Some(r) => r,
            None => return Err(CapacityError::InvalidReservation { current: self.capacity, requested: capacity }),
        };
        let mut a = self.alloc.borrow_mut();
        let tmp: *mut T = match unsafe { a.allocate_uninit(new_capacity) } {
            Some(r) => r.as_ptr(),
            None => return Err(CapacityError::AllocFailed { new_capacity }),
        };
        debug_assert!(self.len <= self.capacity);
        unsafe {
            <T as MemoryStrategy>::move_elements(self.data, tmp, self.len);
        }
        if self.capacity != 0 {
            unsafe { a.free_uninit(NonNull::new(self.data).unwrap(), self.capacity); }
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
                unsafe { core::ptr::write(self.data.add(i), value.clone()) }
            }
        }
        else if len < self.len {
            unsafe {
                <T as MemoryStrategy>::drop_in_place(self.data.add(len), self.len - len);
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
                unsafe { core::ptr::write(self.data.add(i), f()) }
            }
        }
        else if len < self.len {
            unsafe {
                <T as MemoryStrategy>::drop_in_place(self.data.add(len), self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn push(&mut self, value: T) -> Result<&mut T, CapacityError> {
        if self.len >= self.capacity {
            if self.capacity == 0 {
                self.reserve(2)?
            }
            else {
                self.reserve(self.capacity * 2)?
            }
        }
        let ptr = unsafe { self.data.add(self.len) };
        unsafe { core::ptr::write(ptr, value) };
        self.len += 1;
        Ok(unsafe { &mut *ptr })
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        let ptr = unsafe { self.data.add(self.len) };
        self.len -= 1;
        Some(unsafe { ptr.read() })
    }

    fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(&mut *self.data.add(self.len - 1))
            }
        }
    }

    fn back_mut(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(&mut *self.data.add(self.len - 1))
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
            let ptr = <T as MemoryStrategy>::insert(self.data, value, index, self.len);
            self.len += 1;
            Ok(&mut *ptr)
        }
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index == self.len { debug_assert!(false); return None }
        let removed = unsafe { core::ptr::read(self.data.add(index)) };
        for i in index..self.len - 1 {
            unsafe { core::ptr::write(self.data.add(i), core::ptr::read(self.data.add(i + 1))) }
        }
        self.len -= 1;
        Some(removed)
    }

    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index == self.len { return None }
        let removed = unsafe { core::ptr::read(self.data.add(index)) };
        self.len -= 1;
        if index != self.len {
            unsafe { core::ptr::write(self.data.add(index), core::ptr::read(self.data.add(self.len))) }
        }
        Some(removed)
    }

    fn clear(&mut self) {
        debug_assert!(self.len <= self.capacity);
        if self.capacity == 0 { return }
        unsafe {
            <T as MemoryStrategy>::drop_in_place(self.data, self.len);
        }
        let mut a = self.alloc.borrow_mut();
        unsafe { a.free_uninit(
            NonNull::new(self.data).unwrap(),
            self.capacity)
        }
        self.len = 0;
        self.capacity = 0;
        self.data = core::ptr::dangling::<T>() as _;
    }

    fn duplicate_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            V: Vector<T>,
            T: DuplicateStrategy {
        if self.capacity < from.len() {
            self.clear();
            self.reserve(from.len())?
        }
        else {
            unsafe { <T as MemoryStrategy>::drop_in_place(self.data, self.len); }
            self.len = 0;
        }
        unsafe { <T as DuplicateStrategy>::duplicate(from.as_ptr() as _, self.data, from.len()); }
        self.len = from.len();
        Ok(())
    }

    fn contains(&self, value: &T) -> bool
        where
            T: PartialEq
    {
        for i in 0..self.len {
            if unsafe { *self.data.add(i) == *value } {
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
            Iter::new(ptr, end, PhantomData)
        }
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            let ptr = self.data;
            let end = self.data.add(self.len);
            IterMut::new(ptr, end, PhantomData)
        }
    }
}

impl<'alloc, T, Alloc, CapacityPol> Index<usize> for AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_ref().expect("failed to read data") }
    }
} 

impl<'alloc, T, Alloc, CapacityPol> IndexMut<usize> for AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_mut().expect("failed to read data") }
    }
}

impl<'vec, 'alloc, T, Alloc, CapacityPol> IntoIterator for &'vec AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{

    type Item = &'vec T;
    type IntoIter = Iter<'vec, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'vec, 'alloc, T, Alloc, CapacityPol> IntoIterator for &'vec mut AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{

    type Item = &'vec mut T;
    type IntoIter = IterMut<'vec, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'alloc, T, Alloc, CapacityPol> Drop for AllocVec<'alloc, T, Alloc, CapacityPol>
    where
        T: MemoryStrategy,
        Alloc: Allocate + Free,
        CapacityPol: CapacityPolicy,
{
    fn drop(&mut self) {
        self.clear();
    }
}
