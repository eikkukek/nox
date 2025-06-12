use core::{
    mem::MaybeUninit, ops::{Index, IndexMut}, ptr, slice
};

use super::{
    traits::MemoryStrategy, CapacityError, Fixed, Vector
};

pub struct ArrayVec<T, const N: usize>
    where
        T: MemoryStrategy
{
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N>
    where
        T: MemoryStrategy,
{

    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<T, const N: usize> Vector<T> for ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    type CapacityPol = Fixed;

    type Iter<'a> = slice::Iter<'a, T>
        where T: 'a, Self: 'a;

    type IterMut<'a> = slice::IterMut<'a, T>
        where T: 'a, Self: 'a;

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
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

    fn reserve(&mut self, _: usize) -> Result<(), CapacityError> {
        Err(CapacityError::Fixed)
    }

    fn resize(&mut self, len: usize, value: T) -> Result<(), CapacityError>
        where
            T: Clone
    {
        if len > N { return Err(CapacityError::Fixed) }
        let ptr = self.as_mut_ptr();
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i)).write(value.clone()) };
            }
        }
        else if len < self.len {
            for i in len..self.len {
                unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
            }
        }
        self.len = len;
        Ok(())
    }

    fn resize_with<F>(&mut self, len: usize, mut f: F) -> Result<(), CapacityError>
        where
            F: FnMut() -> T
    {
        if len > N { return Err(CapacityError::Fixed) }
        let ptr = self.as_mut_ptr();
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i)).write(f()) };
            }
        }
        else if len < self.len {
            for i in len..self.len {
                unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
            }
        }
        self.len = len;
        Ok(())
    }

    fn push(&mut self, value: T) -> Result<&mut T, CapacityError> {
        if self.len >= N { return Err(CapacityError::Fixed) }
        let ptr = unsafe { self.as_mut_ptr().add(self.len) };
        unsafe { std::ptr::write(ptr, MaybeUninit::new(value)) };
        self.len += 1;
        Ok(unsafe { &mut *(ptr as *mut T) })
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        let ptr = unsafe { self.as_mut_ptr().add(self.len).cast::<T>() };
        self.len -= 1;
        Some(unsafe { ptr.read() })
    }

    fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe { Some(self.data[self.len - 1].assume_init_ref()) }
        }
    }

    fn back_mut(&mut self) -> Option<&mut T> {
         if self.len == 0 {
            None
        }
        else {
            unsafe { Some(self.data[self.len - 1].assume_init_mut()) }
        }       
    }

    fn insert(&mut self, value: T, index: usize) -> Result<&mut T, CapacityError> {
        if index >= self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len == N { return Err(CapacityError::Fixed) }
        unsafe {
            let data = self.as_mut_ptr();
            for i in (index + 1..=self.len).rev() {
                data.add(i).write(data.add(i - 1).read());
            }
            let ptr = data.add(index) as *mut T;
            ptr.write(value);
            self.len += 1;
            Ok(&mut *ptr)
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

    fn clone_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            V: Vector<T>,
            T: Clone + Default,
    {
        if N < from.len() {
            return Err(CapacityError::Fixed)
        }
        self.clear();
        let ptr = self.as_mut_ptr();
        for (i, val) in from.iter().enumerate() {
            unsafe { ptr.add(i).write(MaybeUninit::new(val.clone())) }
        }
        self.len = from.len();
        Ok(())
    }

    fn copy_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            V: Vector<T>,
            T: Copy + Default,
    {
        if N < from.len() {
            return Err(CapacityError::Fixed)
        }
        self.len = from.len();
        self.as_mut_slice().copy_from_slice(from.as_slice());
        Ok(())
    }

    fn contains(&self, value: &T) -> bool
        where
            T: Eq
    {
        let ptr = self.as_ptr() as *const T;
        for i in 0..self.len {
            if unsafe { *ptr.add(i) == *value } {
                return true 
            }
        }
        return false
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.as_mut_slice().iter_mut()
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    fn drop(&mut self) {
        debug_assert!(self.len <= N);
        let ptr = self.as_mut_ptr();
        for i in 0..self.len {
            unsafe { ptr::drop_in_place((*ptr.add(i)).as_mut_ptr()); }
        }
    }
}

impl<T, const N: usize> Index<usize> for ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { &*self.as_ptr().cast::<T>().add(index) }
    }
}

impl<T, const N: usize> IndexMut<usize> for ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index {} out of bounds for length {}", index, self.len)
        }
        unsafe { &mut *self.as_mut_ptr().cast::<T>().add(index) }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N>
    where
        T: MemoryStrategy
{

    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
