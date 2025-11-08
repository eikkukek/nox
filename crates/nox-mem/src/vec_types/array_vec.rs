use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{self},
    fmt::{Debug, Display},
    slice
};

use crate::{
    errors::CapacityError,
    impl_traits,
};

use super::{
    Vector,
    Pointer,
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

    pub fn with_len(value: T, len: usize) -> Result<Self, CapacityError>
        where
            T: Clone,
    {
        if len > N {
            return Err(CapacityError::FixedCapacity { capacity: N })
        }
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..len {
            data[i].write(value.clone());
        }
        Ok(Self {
            data,
            len,
        })
    }

    pub fn mapped<U>(&self, f: impl FnMut(&T) -> U) -> ArrayVec<U, N>
    {
        let mut vec = ArrayVec::new();
        vec.append_map(self, f).unwrap();
        vec
    }
}

impl<T, const N: usize> Vector<T> for ArrayVec<T, N>
{
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
        Err(CapacityError::FixedCapacity { capacity: N })
    }

    fn resize(&mut self, len: usize, value: T) -> Result<(), CapacityError>
        where
            T: Clone
    {
        if len > N { return Err(CapacityError::FixedCapacity { capacity: N }) }
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
        if len > N { return Err(CapacityError::FixedCapacity { capacity: N }) }
        let ptr = unsafe {
            Pointer::new(self.as_mut_ptr()).unwrap_unchecked()
        };
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i)).write(f()) };
            }
        }
        else if len < self.len {
            unsafe {
                ptr.add(len).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn append(&mut self, slice: &[T]) -> Result<(), CapacityError>
        where
            T: Clone
    {
        if slice.len() > N { return Err(CapacityError::FixedCapacity { capacity: N }) }
        let new_len = self.len + slice.len();
        if new_len > N {
            return Err(CapacityError::FixedCapacity { capacity: N })
        }
        unsafe {
            Pointer
                ::new(slice.as_ptr() as _)
                .unwrap()
                .clone_elements(
                    Pointer::new(self.as_mut_ptr()).unwrap_unchecked().add(self.len),
                    slice.len()
                );
        }
        self.len = new_len;
        Ok(())
    }

    fn append_map<U, F>(&mut self, slice: &[U], mut f: F) -> Result<(), CapacityError>
        where
            F: FnMut(&U) -> T
    {
        if slice.len() > N { return Err(CapacityError::FixedCapacity { capacity: N }) }
        let new_len = self.len + slice.len();
        if new_len > N {
            return Err(CapacityError::FixedCapacity { capacity: N })
        }
        let len = self.len;
        let ptr = self.as_mut_ptr();
        for (i, u) in slice.iter().enumerate() {
            unsafe {
                ptr.add(len + i).write(f(u));
            }
        }
        self.len = new_len;
        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, value: T) -> Result<&mut T, CapacityError> {
        if self.len >= N { return Err(CapacityError::FixedCapacity { capacity: N }) }
        let ptr = unsafe { self.as_mut_ptr().add(self.len) };
        unsafe { ptr::write(ptr, value) };
        self.len += 1;
        Ok(unsafe { &mut *(ptr as *mut T) })
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None }
        self.len -= 1;
        let ptr = unsafe { self.as_mut_ptr().add(self.len).cast::<T>() };
        Some(unsafe { ptr.read() })
    }

    #[inline(always)]
    fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe { Some(self.data[self.len - 1].assume_init_ref()) }
        }
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
         if self.len == 0 {
            None
        }
        else {
            unsafe { Some(self.data[self.len - 1].assume_init_mut()) }
        }       
    }

    fn insert(&mut self, index: usize, value: T) -> Result<&mut T, CapacityError> {
        if index >= self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len == N { return Err(CapacityError::FixedCapacity { capacity: N }) }
        unsafe {
            let mut ptr = Pointer
                ::new(self.as_mut_ptr())
                .unwrap_unchecked()
                .insert_element(value, index, self.len);
            self.len += 1;
            Ok(ptr.as_mut())
        }
    }

    fn remove(&mut self, index: usize) -> T {
        if index >= self.len { panic!("index {} was out of bounds with len {} when removing", index, self.len()) }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { ptr::read(ptr.add(index)) };
        for i in index..self.len - 1 {
            unsafe { ptr::write(ptr.add(i), ptr::read(ptr.add(i + 1))) }
        }
        self.len -= 1;
        removed
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len { panic!("index {} was out of bounds with len {} when removing", index, self.len()) }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { ptr::read(ptr.add(index)) };
        self.len -= 1;
        if index != self.len {
            unsafe { ptr::write(ptr.add(index), ptr::read(ptr.add(self.len))) }
        }
        removed
    }

    #[inline(always)]
    fn clear(&mut self) {
        debug_assert!(self.len <= N);
        unsafe { Pointer::new(self.as_mut_ptr()).unwrap_unchecked().drop_in_place(self.len); }
        self.len = 0;
    }

    fn clone_from_slice(&mut self, from: &[T]) -> Result<(), CapacityError>
        where
            T: Clone
    {
        if N < from.len() {
            return Err(CapacityError::FixedCapacity { capacity: N })
        }
        let ptr = unsafe {
            Pointer::new(self.as_mut_ptr()).unwrap_unchecked()
        };
        unsafe { ptr.drop_in_place(self.len); }
        self.len = 0;
        unsafe { Pointer
            ::new(from.as_ptr() as _)
            .unwrap()
            .clone_elements(ptr, from.len()); }
        self.len = from.len();
        Ok(())
    }

    fn move_from_vec<V>(&mut self, from: &mut V) -> Result<(), CapacityError>
        where
            V: Vector<T>
    {
        let slice = from.as_mut_slice();
        if N < slice.len() {
            return Err(CapacityError::FixedCapacity { capacity: N })
        }
        let ptr = unsafe {
            Pointer::new(self.as_mut_ptr()).unwrap_unchecked()
        };
        unsafe { ptr.drop_in_place(self.len); }
        self.len = 0;
        unsafe { Pointer
            ::new(slice.as_mut_ptr())
            .unwrap()
            .move_elements(ptr, slice.len());
        }
        self.len = slice.len();
        unsafe { from.set_len(0); }
        Ok(())
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
            unsafe { Pointer::new(self.as_mut_ptr()).unwrap_unchecked().drop_in_place(self.len); }
            self.len = 0;
        }
    ,
    Default =>
        
        fn default() -> Self {
            Self::new()
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
    From<&[T]> where T: Clone =>
        
        fn from(slice: &[T]) -> Self {
            let mut vec = Self::new();
            vec.append(slice).unwrap();
            vec
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
    IntoIterator for &'vec =>

        type Item = &'vec T;
        type IntoIter = slice::Iter<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator for mut &'vec =>

        type Item = &'vec mut T;
        type IntoIter = slice::IterMut<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
    Clone where T: Clone =>
    
        #[inline(always)]
        fn clone(&self) -> Self {
            let mut vec = ArrayVec::new();
            vec.clone_from_slice(self).unwrap();
            vec
        }
    ,
    PartialEq where T: PartialEq =>

        fn eq(&self, rhs: &Self) -> bool {
            self.as_slice() == rhs.as_slice()
        }
    ,
    Eq where T: Eq =>,
    Debug where T: Debug =>

        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            self.as_slice().fmt(f)
        }
    ,
    Display where T: Display =>

        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.len == 0 {
                <str as Display>::fmt(&"[]", f)?;
                return Ok(())
            }
            <char as Display>::fmt(&'[', f)?;
            for value in &self[0..self.len - 1] {
                value.fmt(f)?;
                <str as Display>::fmt(&", ", f)?;
            }
            self.last().unwrap().fmt(f)?;
            <char as Display>::fmt(&']', f)
        }
    ,
}

impl<const N: usize, T: Clone> From<[T; N]> for ArrayVec<T, N> {
    
    fn from(value: [T; N]) -> Self {
        let mut vec = ArrayVec::new();
        for v in value {
            vec.push(v.clone()).unwrap();
        }
        vec
    }
}
