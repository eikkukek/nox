use super::allocator_traits::AllocateExt;

use core::{
    marker::PhantomData, mem::MaybeUninit, ops::{Index, IndexMut}, ptr, slice
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

pub struct MutVecIter<'a, T> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for MutVecIter<'a, T> {

    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            let item = unsafe { &mut *self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(item)
        }
    }
}

impl<'a, T> DoubleEndedIterator for MutVecIter<'a, T> {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            self.end = unsafe { self.ptr.sub(1) };
            Some(unsafe { &mut *self.end })
        }
    }
}

pub trait VecOperations<T> {

    fn len(&self) -> usize;

    fn size(&self) -> usize;

    fn as_ptr(&self) -> *const MaybeUninit<T>;

    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T>;

    fn as_slice(&self) -> &[T];

    fn as_mut_slice(&mut self) -> &mut [T];

    fn push_back(&mut self, value: T) -> Option<&mut T>; 

    fn pop_back(&mut self) -> Option<T>;

    fn insert(&mut self, value: T, index: usize) -> Option<&mut T>;

    fn remove(&mut self, index: usize) -> Option<T>;

    fn swap_remove(&mut self, index: usize) -> Option<T>;

    fn clear(&mut self);

    fn contains(&self,
        value: &T
    ) -> bool
        where
            T: Eq;

    fn push_back_if_unique(
        &mut self,
        value: T
    ) -> Option<&mut T>
        where
            T: Eq
    {
        if self.contains(&value) { None }
        else { self.push_back(value) }
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

    pub fn new_with<'short, U, F>(size: usize, len: usize, f: F, allocator: &'short mut U) -> Option<Self>
        where
            F: FnMut(usize) -> T,
            U: AllocateExt<'a>
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

    pub fn back(&mut self) -> Option<&mut T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(&mut *self.data.add(self.len - 1))
            }
        }
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

    pub fn iter(&self) -> VecIter<'_, T> {
        let ptr = self.data;
        let end = unsafe { self.data.add(self.len) };
        VecIter {
            ptr,
            end,
            _marker: PhantomData,
        }
    }

    pub fn mut_iter(&self) -> MutVecIter<'_, T> {
        let ptr = self.data;
        let end = unsafe { self.data.add(self.len) };
        MutVecIter {
            ptr,
            end,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> VecOperations<T> for FixedVec<'a, T> {

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

    fn insert(&mut self, value: T, index: usize) -> Option<&mut T> {
        if index >= self.len || self.len == self.size {
            None
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

impl<'a, 'b, T> IntoIterator for &'a FixedVec<'b, T> {

    type Item = &'a T;
    type IntoIter = VecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'b, T> IntoIterator for &'a mut FixedVec<'b, T> {

    type Item = &'a mut T;
    type IntoIter = MutVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.mut_iter()
    }
}

pub struct ArrayVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {

    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    pub fn new_with_len<'short, U>(len: usize) -> Option<Self>
        where
            T: Default,
    {
        if N < len { return None }
        let mut data = unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };
        for i in 0..len {
            data[i].write(T::default());
        }
        Some(
            Self {
                data,
                len,
            }
        )
    }

    pub fn resize(&mut self, len: usize)
        where
            T: Default
    {
        if len > N { return; }
        let ptr = self.as_mut_ptr();
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i)).write(T::default()) };
            }
        }
        else if len < self.len {
            for i in len..self.len {
                unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
            }
        }
        self.len = len;
    }

    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> VecOperations<T> for ArrayVec<T, N> {

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn size(&self) -> usize {
        N
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.data.as_ptr()
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T> {
        self.data.as_mut_ptr()
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, self.len) }
    }

    fn push_back(&mut self, value: T) -> Option<&mut T> {
        if self.len == N { return None }
        let ptr = unsafe { self.as_mut_ptr().add(self.len) };
        unsafe { std::ptr::write(ptr, MaybeUninit::new(value)) };
        self.len += 1;
        Some(unsafe { &mut *(ptr as *mut T) })
    }

    fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        let ptr = unsafe { self.as_mut_ptr().add(self.len).cast::<T>() };
        self.len -= 1;
        Some(unsafe { ptr.read() })
    }

    fn insert(&mut self, value: T, index: usize) -> Option<&mut T> {
        if index >= self.len || self.len == N {
            None
        }
        else {
            unsafe {
                let data = self.as_mut_ptr();
                for i in (index + 1..=self.len).rev() {
                    data.add(i).write(data.add(i - 1).read());
                }
                let ptr = data.add(index) as *mut T;
                ptr.write(value);
                Some(ptr.as_mut()?)
            }
        }
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { std::ptr::read(ptr.add(index)).assume_init() };
        for i in index..self.len - 1 {
            unsafe { std::ptr::write(ptr.add(i), std::ptr::read(ptr.add(i + 1))) }
        }
        self.len -= 1;
        Some(removed)
    }

    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { std::ptr::read(ptr.add(index)).assume_init() };
        self.len -= 1;
        if index != self.len {
            unsafe { std::ptr::write(ptr.add(index), std::ptr::read(ptr.add(self.len))) }
        }
        Some(removed)
    }

    fn clear(&mut self) {
        debug_assert!(self.len <= N);
        let ptr = self.as_mut_ptr();
        for i in 0..self.len {
            unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
        }
        self.len = 0;
    }

    fn contains(
        &self,
        value: &T
    ) -> bool
        where
            T: Eq
    {
        let ptr = self.as_ptr() as *const T;
        for i in 0..self.len - 1 {
            if unsafe { *ptr.add(i) == *value } {
                return true 
            }
        }
        return false
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {

    fn drop(&mut self) {
        debug_assert!(self.len <= N);
        let ptr = self.as_mut_ptr();
        for i in 0..self.len {
            unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
        }
    }
}

impl<T, const N: usize> Index<usize> for ArrayVec<T, N> {

    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { &*self.as_ptr().cast::<T>().add(index) }
    }
}

impl<T, const N: usize> IndexMut<usize> for ArrayVec<T, N> {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { &mut *self.as_mut_ptr().cast::<T>().add(index) }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {

    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {

    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
