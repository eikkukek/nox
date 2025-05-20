use super::allocator_traits::AllocateExt;

use std::{
    marker::PhantomData, slice,
    ops::{Index, IndexMut},
};

pub struct VecIter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for VecIter<'a, T> {

    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            let item = unsafe { &*self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(item)
        }
    }
}

impl<'a, T> DoubleEndedIterator for VecIter<'a, T> {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            self.end = unsafe { self.ptr.sub(1) };
            Some(unsafe { &*self.end })
        }
    }
}

pub struct FixedVec<'a, T> {
    data: *mut T,
    size: usize,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> FixedVec<'a, T> {

    pub fn new<'short, U>(size: usize, allocator: &'short mut U) -> Option<Self>
        where
            U: AllocateExt<'a>
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

    pub fn new_with_len<'short, U>(size: usize, len: usize, allocator: &'short mut U) -> Option<Self>
        where
            T: Default,
            U: AllocateExt<'a>
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn as_ptr(&self) -> *const T {
        self.data
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data, self.len) }
    }

    pub fn resize(&mut self, len: usize)
        where
            T: Default
    {
        if len > self.size { return; }
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
    }

    pub fn push(&mut self, value: T) -> Option<&mut T> {
        if self.len == self.size { return None; }
        let ptr = unsafe { self.data.add(self.len) };
        unsafe { std::ptr::write(ptr, value) };
        self.len += 1;
        Some(unsafe { &mut *ptr })
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { debug_assert!(false); return None }
        let removed = unsafe { std::ptr::read(self.data.add(index)) };
        for i in index..self.len - 1 {
            unsafe { std::ptr::write(self.data.add(i), std::ptr::read(self.data.add(i + 1))) }
        }
        self.len -= 1;
        Some(removed)
    }

    pub fn clear(&mut self) {
        debug_assert!(self.len <= self.size);
        for i in 0..self.len {
            unsafe {
                self.data.add(i).drop_in_place();
            }
        }
        self.len = 0;
    }

    pub fn iter(&self) -> VecIter<'_, T> {
        let ptr = self.data;
        let end = unsafe { self.data.add(self.len) };
        VecIter {
            ptr,
            end,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Drop for FixedVec<'a, T> {

    fn drop(&mut self) {
        debug_assert!(self.len <= self.size);
        for i in 0..self.len {
            unsafe {
                self.data.add(i).drop_in_place();
            }
        }
    }
}

impl<'a, T> Index<usize> for FixedVec<'a, T> {

    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_ref().expect("failed to read data") }
    }
}

impl<'a, T> IndexMut<usize> for FixedVec<'a, T> {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { self.data.add(index).as_mut().expect("failed to read data") }
    }
}
