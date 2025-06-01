use super::{
    vec_iter::IterConstruct, Iter, IterMut, Vector,
};

use crate::allocator_traits::AllocateExt;

use core::{
    ops::{Index, IndexMut},
    slice,
    marker::PhantomData,
    mem::MaybeUninit,
};

pub struct FixedVec<'mem, T> {
    data: *mut T,
    size: usize,
    len: usize,
    _marker: PhantomData<&'mem mut T>,
}

impl<'mem, T> FixedVec<'mem, T> {

    pub fn new<'short, U>(size: usize, allocator: &'short mut U) -> Option<Self>
        where
            U: AllocateExt<'mem>
    {
        let ptr = unsafe { allocator.allocate_uninit(size)? };
        Some(
            Self {
                data: ptr.as_ptr(),
                size,
                len: 0,
                _marker: PhantomData,
            }
        )
    }

    pub fn new_with<'short, U, F>(size: usize, len: usize, f: F, allocator: &'short mut U) -> Option<Self>
        where
            F: FnMut(usize) -> T,
            U: AllocateExt<'mem>
    {
        if size < len { return None }
        let ptr = allocator.allocate_with(len, f)?.as_ptr();
        Some(
            Self {
                data: ptr,
                size,
                len,
                _marker: PhantomData,
            }
        )
    }

    pub fn new_with_default<'short, U>(size: usize, len: usize, allocator: &'short mut U) -> Option<Self>
        where
            T: Default,
            U: AllocateExt<'mem>
    {
        if size < len { return None }
        let ptr: *mut T = unsafe { allocator.allocate_uninit(size)?.as_ptr() };
        for i in 0..len {
            unsafe { ptr.add(i).write(T::default()); }
        }
        Some(
            Self {
                data: ptr,
                size,
                len,
                _marker: PhantomData,
            }
        )
    }
}

impl<'mem, T> Vector<T> for FixedVec<'mem, T> {

    type Iter<'a> = Iter<'a, T>
        where T: 'a, Self: 'a;

    type IterMut<'a> = IterMut<'a, T>
        where T: 'a, Self: 'a;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.data as *const MaybeUninit<T>
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T> {
        self.data as *mut MaybeUninit<T>
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    fn resize(&mut self, len: usize) -> bool
        where
            T: Default
    {
        if len > self.size { panic!("given length was larger than vector size") }
        if len > self.len {
            for i in self.len..len {
                unsafe { std::ptr::write(self.data.add(i), T::default()) }
            }
        }
        else if len < self.len {
            for i in len..self.len {
                unsafe { std::ptr::drop_in_place(self.data.add(i)); }
            }
        }
        self.len = len;
        true
    }

    fn push_back(&mut self, value: T) -> Option<&mut T> {
        if self.len == self.size { return None }
        let ptr = unsafe { self.data.add(self.len) };
        unsafe { std::ptr::write(ptr, value) };
        self.len += 1;
        Some(unsafe { &mut *ptr })
    }

    fn pop_back(&mut self) -> Option<T> {
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

    fn insert(&mut self, value: T, index: usize) -> Option<&mut T> {
        if index > self.len || self.len == self.size {
            None
        }
        else if index == self.len {
            self.push_back(value)
        }
        else {
            unsafe {
                for i in (index + 1..=self.len).rev() {
                    self.data.add(i).write(self.data.add(i - 1).read());
                }
                let ptr = self.data.add(index);
                ptr.write(value);
                Some(ptr.as_mut()?)
            }
        }
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { debug_assert!(false); return None }
        let removed = unsafe { std::ptr::read(self.data.add(index)) };
        for i in index..self.len - 1 {
            unsafe { std::ptr::write(self.data.add(i), std::ptr::read(self.data.add(i + 1))) }
        }
        self.len -= 1;
        Some(removed)
    }

    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }
        let removed = unsafe { std::ptr::read(self.data.add(index)) };
        self.len -= 1;
        if index != self.len {
            unsafe { std::ptr::write(self.data.add(index), std::ptr::read(self.data.add(self.len))) }
        }
        Some(removed)
    }

    fn clear(&mut self) {
        debug_assert!(self.len <= self.size);
        for i in 0..self.len {
            unsafe {
                self.data.add(i).drop_in_place();
            }
        }
        self.len = 0;
    }

    fn clone_from<V: Vector<T>>(&mut self, from: &V) -> bool
            where
                T: Clone + Default {
        if self.size() < from.len() {
            return false
        }
        self.clear();
        for (i, val) in from.iter().enumerate() {
            unsafe { self.data.add(i).write(val.clone()) }
        }
        self.len = from.len();
        true
    }

    fn copy_from<V: Vector<T>>(&mut self, from: &V) -> bool
            where
                T: Copy + Default {
        if self.size() < from.len() {
            return false
        }
        self.len = from.len();
        self.as_mut_slice().copy_from_slice(from.as_slice());
        true
    }

    fn contains(&self, value: &T) -> bool
        where
            T: Eq
    {
        for i in 0..self.len - 1 {
            if unsafe { *self.data.add(i) == *value } {
                return true 
            }
        }
        return false
    }

    fn iter(&self) -> Iter<'_, T> {
        let ptr = self.data;
        let end = unsafe { self.data.add(self.len) };
        Iter::new(ptr, end, PhantomData)
    }

    fn iter_mut(&mut self) -> IterMut<'_, T> {
        let ptr = self.data;
        let end = unsafe { self.data.add(self.len) };
        IterMut::new(ptr, end, PhantomData)
    }
}

impl<'mem, T> Drop for FixedVec<'mem, T> {

    fn drop(&mut self) {
        debug_assert!(self.len <= self.size);
        for i in 0..self.len {
            unsafe {
                self.data.add(i).drop_in_place();
            }
        }
    }
}

impl<'mem, T> Index<usize> for FixedVec<'mem, T> {

    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_ref().expect("failed to read data") }
    }
}

impl<'mem, T> IndexMut<usize> for FixedVec<'mem, T> {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_mut().expect("failed to read data") }
    }
}

impl<'vec, 'mem, T> IntoIterator for &'vec FixedVec<'mem, T> {

    type Item = &'vec T;
    type IntoIter = Iter<'vec, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'vec, 'mem, T> IntoIterator for &'vec mut FixedVec<'mem, T> {

    type Item = &'vec mut T;
    type IntoIter = IterMut<'vec, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
