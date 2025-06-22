use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut, Deref, DerefMut},
    ptr,
    slice
};

use crate::{
    capacity_policy::Fixed,
    errors::CapacityError,
    impl_traits,
};

use super::{
    Vector,
    MemoryStrategy,
    CloneStrategy,
};

pub struct ArrayVec<T, const N: usize>
{
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N>
{

    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<T, const N: usize> Vector<T> for ArrayVec<T, N>
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
    fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as _
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as _
    }

    #[inline(always)]
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const T, self.len) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, self.len) }
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, len: usize) {
        if len > N { panic!("len was larger than capacity") }
        self.len = len;
    }

    #[inline(always)]
    fn reserve(&mut self, _: usize) -> Result<(), CapacityError> {
        Err(CapacityError::Fixed { capacity: N })
    }

    fn resize(&mut self, len: usize, value: T) -> Result<(), CapacityError>
        where
            T: Clone
    {
        if len > N { return Err(CapacityError::Fixed { capacity: N }) }
        let ptr = self.data.as_mut_ptr();
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
        if len > N { return Err(CapacityError::Fixed { capacity: N }) }
        let ptr = self.data.as_mut_ptr();
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

    #[inline(always)]
    fn push(&mut self, value: T) -> Result<&mut T, CapacityError> {
        if self.len >= N { return Err(CapacityError::Fixed { capacity: N }) }
        let ptr = unsafe { self.as_mut_ptr().add(self.len) };
        unsafe { ptr::write(ptr, value) };
        self.len += 1;
        Ok(unsafe { &mut *(ptr as *mut T) })
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        let ptr = unsafe { self.as_mut_ptr().add(self.len).cast::<T>() };
        self.len -= 1;
        Some(unsafe { ptr.read() })
    }

    #[inline(always)]
    fn back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe { Some(self.data[self.len - 1].assume_init_ref()) }
        }
    }

    #[inline(always)]
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
        if self.len == N { return Err(CapacityError::Fixed { capacity: N }) }
        unsafe {
            let data = self.as_mut_ptr();
            let ptr = Self::insert_element(data as _, value, index, self.len);
            self.len += 1;
            Ok(&mut *ptr)
        }
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { ptr::read(ptr.add(index)) };
        for i in index..self.len - 1 {
            unsafe { std::ptr::write(ptr.add(i), std::ptr::read(ptr.add(i + 1))) }
        }
        self.len -= 1;
        Some(removed)
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len { return None }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { std::ptr::read(ptr.add(index)) };
        self.len -= 1;
        if index != self.len {
            unsafe { std::ptr::write(ptr.add(index), std::ptr::read(ptr.add(self.len))) }
        }
        Some(removed)
    }

    #[inline(always)]
    fn clear(&mut self) {
        debug_assert!(self.len <= N);
        unsafe { Self::drop_in_place(self.as_mut_ptr(), self.len); }
        self.len = 0;
    }

    fn clone_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            T: Clone,
            V: Vector<T>,
    {
        if N < from.len() {
            return Err(CapacityError::Fixed { capacity: N })
        }
        else {
            unsafe { Self::drop_in_place(self.as_mut_ptr(), self.len); }
            self.len = 0;
        }
        unsafe { V::clone_elements(from.as_ptr(), self.as_mut_ptr(), from.len()); }
        self.len = from.len();
        Ok(())
    }

    fn move_from<V>(&mut self, from: &mut V) -> Result<(), CapacityError>
        where
            V: Vector<T>
    {
        if N < from.len() {
            return Err(CapacityError::Fixed { capacity: N })
        }
        else {
            unsafe { Self::drop_in_place(self.as_mut_ptr(), self.len); }
            self.len = 0;
        }
        unsafe { V::move_elements(from.as_ptr(), self.as_mut_ptr(), from.len()); }
        self.len = from.len();
        unsafe { from.set_len(0); }
        Ok(())
    }

    fn contains(&self, value: &T) -> bool
        where
            T: PartialEq
    {
        let ptr = self.as_ptr() as *const T;
        for i in 0..self.len {
            if unsafe { *ptr.add(i) == *value } {
                return true 
            }
        }
        return false
    }

    #[inline(always)]
    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.as_mut_slice().iter_mut()
    }
}

impl_traits! {
    for ArrayVec<T, N: usize [const]>
    Drop =>

        #[inline(always)]
        fn drop(&mut self) -> () {
            debug_assert!(self.len <= N);
            unsafe { Self::drop_in_place(self.as_mut_ptr(), self.len); }
            self.len = 0;
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
    Index<usize> =>
        
        type Output = T;

        #[inline(always)]
        fn index(&self, index: usize) -> &Self::Output {
            if index >= self.len {
                panic!("index {} out of bounds for length {}", index, self.len)
            }
            unsafe { &*self.as_ptr().cast::<T>().add(index) }
        }       
    ,
    IndexMut<usize> =>
        #[inline(always)]
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            if index >= self.len {
                panic!("index {} out of bounds for length {}", index, self.len)
            }
            unsafe { &mut *self.as_mut_ptr().cast::<T>().add(index) }
        }
    ,
    IntoIterator &'vec =>

        type Item = &'vec T;
        type IntoIter = slice::Iter<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator &'vec mut =>

        type Item = &'vec T;
        type IntoIter = slice::Iter<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
}
