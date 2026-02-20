use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::{self},
    fmt::{self, Debug, Display, Formatter},
    borrow::{Borrow, BorrowMut},
    hash::{Hash, Hasher},
    slice,
};

use crate::{
    collections::TryReserveError,
    impl_traits, num::{FromUsize, IntoUsize},
};

use super::{
    Vector,
    FallibleVec,
    Pointer,
    alloc_vec::FixedPolicy,
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

    pub fn with_len(len: usize, value: T) -> Result<Self, TryReserveError<()>>
        where
            T: Clone,
    {
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for t in data.iter_mut().take(len) {
            t.write(value.clone());
        }
        Ok(Self {
            data,
            len,
        })
    }

    pub fn with_len_with<F>(len: usize, mut f: F) -> Result<Self, TryReserveError<()>>
        where
            F: FnMut() -> T
    {
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for t in data.iter_mut().take(len) {
            t.write(f());
        }
        Ok(Self {
            data,
            len,
        })
    }

    pub fn mapped<U>(&self, f: impl FnMut(&T) -> U) -> ArrayVec<U, N>
    {
        let mut vec = ArrayVec::new();
        vec.append_map(self, f);
        vec
    }
}

impl<T, const N: usize> Vector<T> for ArrayVec<T, N>
{

    type Iter<'a> = slice::Iter<'a, T>
        where T: 'a, Self: 'a;

    type IterMut<'a> = slice::IterMut<'a, T>
        where T: 'a, Self: 'a;

    type ReservePol = FixedPolicy;

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
        self.data.as_ptr() as *const T
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
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
    fn reserve(&mut self, capacity: usize) { 
        self.fallible_reserve(capacity).unwrap()
    }

    #[inline(always)]
    fn reserve_exact(&mut self, capacity: usize) {
        self.fallible_reserve_exact(capacity).unwrap()
    }

    fn resize(&mut self, len: usize, value: T)
        where
            T: Clone
    {
        self.fallible_resize(len, value).unwrap();
    }

    fn resize_with<F>(&mut self, len: usize, f: F)
        where
            F: FnMut() -> T
    {
        self.fallible_resize_with(len, f).unwrap();
    }

    fn try_resize_with<F, E>(
        &mut self,
        len: usize,
        mut f: F,
    ) -> Result<(), E>
        where
            F: FnMut() -> Result<T, E>,
    {
        if len > N {
            panic!("max capacity {N} exceeded, with len {len}")
        }
        let ptr = unsafe {
            Pointer::new(self.as_mut_ptr()).unwrap_unchecked()
        };
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i))
                    .write(f().inspect_err(|_| {
                        self.len = i;
                    })?)
                };
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

    #[inline(always)]
    fn push(&mut self, value: T) {
        self.fallible_push(value).unwrap()
    }

    fn append(&mut self, slice: &[T])
        where
            T: Clone
    {
        self.fallible_append(slice).unwrap()
    }

    fn append_map<U, F>(&mut self, slice: &[U], f: F)
        where
            F: FnMut(&U) -> T
    {
        self.fallible_append_map(slice, f).unwrap()
    }

    fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("index {} was out of bounds with len {} when removing", index, self.len())
        }
        let ptr = self.as_mut_ptr();
        let removed = unsafe { ptr::read(ptr.add(index)) };
        for i in index..self.len - 1 {
            unsafe { ptr::write(ptr.add(i), ptr::read(ptr.add(i + 1))) }
        }
        self.len -= 1;
        removed
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

    fn insert(&mut self, index: usize, value: T) {
        self.fallible_insert(index, value).unwrap()
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

    fn move_from_vec<V, S>(&mut self, from: &mut V)
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize,
    { 
        self.fallible_move_from_vec(from).unwrap()
    }

    fn move_from_vec_map<U, V, S, F>(&mut self, from: &mut V, f: F)
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T
    {
        self.fallible_move_from_vec_map(from, f).unwrap()
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

impl<T, const N: usize> FallibleVec<T> for ArrayVec<T, N> {

    fn fallible_reserve(&mut self, capacity: usize) -> Result<(), TryReserveError<()>> {
        if capacity > N {
            Err(TryReserveError::max_capacity_exceeded(N, capacity, ()))
        } else {
            Ok(())
        }
    }

    fn fallible_reserve_exact(&mut self, capacity: usize) -> Result<(), TryReserveError<()>> {
        if capacity > N {
            Err(TryReserveError::max_capacity_exceeded(N, capacity, ()))
        } else {
            Ok(())
        }
    }

    fn fallible_resize(&mut self, len: usize, value: T) -> Result<(), TryReserveError<()>>
        where
            T: Clone
    {
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        };
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

    fn fallible_resize_with<F>(&mut self, len: usize, mut f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut() -> T
    {
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
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

    fn fallible_try_resize_with<F, E, MapE>(
        &mut self,
        len: usize,
        mut f: F,
        mut map_reserve_err: MapE,
    ) -> Result<(), E>
        where
            F: FnMut() -> Result<T, E>,
            MapE: FnMut(TryReserveError<()>) -> E
    {
        if len > N {
            return Err(map_reserve_err(
                TryReserveError::max_capacity_exceeded(N, len, ())
            ))
        }
        let ptr = unsafe {
            Pointer::new(self.as_mut_ptr()).unwrap_unchecked()
        };
        if len > self.len {
            for i in self.len..len {
                unsafe { (*ptr.add(i))
                    .write(f().inspect_err(|_| {
                        self.len = i;
                    })?)
                };
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

    fn fallible_push(&mut self, value: T) -> Result<(), TryReserveError<T>> {
        if self.len >= N {
            return Err(TryReserveError::max_capacity_exceeded(N, N + 1, value))
        }
        let ptr = unsafe { self.as_mut_ptr().add(self.len) };
        unsafe { ptr::write(ptr, value) };
        self.len += 1;
        Ok(())
    }

    fn fallible_append(&mut self, slice: &[T]) -> Result<(), TryReserveError<()>>
        where
            T: Clone
    {
        let len = self.len + slice.len();
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
        unsafe {
            Pointer
                ::new(slice.as_ptr().cast_mut())
                .unwrap()
                .clone_elements(
                    Pointer::new(self.as_mut_ptr()).unwrap_unchecked().add(self.len),
                    slice.len()
                );
        }
        self.len = len;
        Ok(())
    }

    fn fallible_append_map<U, F>(&mut self, slice: &[U], mut f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut(&U) -> T
    {
        let len = self.len + slice.len();
        if len > N {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
        let len = self.len;
        let ptr = self.as_mut_ptr();
        for (i, u) in slice.iter().enumerate() {
            unsafe {
                ptr.add(len + i).write(f(u));
            }
        }
        self.len = len;
        Ok(())
    }

    fn fallible_insert(&mut self, index: usize, value: T) -> Result<(), TryReserveError<T>> {
        if index >= self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len >= N {
            return Err(TryReserveError::max_capacity_exceeded(N, N + 1, value))
        }
        unsafe {
            Pointer
                ::new(self.as_mut_ptr())
                .unwrap_unchecked()
                .insert_element(value, index, self.len);
            self.len += 1;
            Ok(())
        }
    }

    fn fallible_move_from_vec<V, S>(&mut self, from: &mut V) -> Result<(), TryReserveError<()>>
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize,
    {
        let slice = from.as_mut_slice();
        if N < slice.len() {
            return Err(TryReserveError::max_capacity_exceeded(N, slice.len(), ()))
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
        unsafe { from.set_len(S::ZERO); }
        Ok(())    
    }

    fn fallible_move_from_vec_map<U, V, S, F>(
        &mut self,
        from: &mut V,
        mut f: F
    ) -> Result<(), TryReserveError<()>>
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T,
    {
        let len = from.len().into_usize();
        if N > len {
            return Err(TryReserveError::max_capacity_exceeded(N, len, ()))
        }
        self.clear();
        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        unsafe {
            for i in 0..len {
                dst.add(i)
                .write(f(src.add(i).read()))
            }
        }
        self.len = len;
        unsafe { from.set_len(S::ZERO); }
        Ok(())
    }
}

pub struct IterArrayVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    start: usize,
    end: usize,
}

impl<T, const N: usize> Iterator for IterArrayVec<T, N> {

    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start != self.end {
            let t = unsafe {
                self.data
                    .as_ptr()
                    .add(self.start)
                    .read()
                    .assume_init_read()
            };
            self.start += 1;
            Some(t)
        } else {
            None
        }
    } 

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.start, Some(self.end))
    }
}

impl<T, const N: usize> DoubleEndedIterator for IterArrayVec<T, N> {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start != self.end {
            self.end -= 1;
            let t = unsafe {
                self.data
                    .as_ptr()
                    .add(self.end)
                    .read()
                    .assume_init_read()
            };
            Some(t)
        } else {
            None
        }
    }
}

impl<T, const N: usize> ExactSizeIterator for IterArrayVec<T, N> {}

impl<T, const N: usize> Drop for IterArrayVec<T, N> {

    fn drop(&mut self) {
        unsafe {
            Pointer::new(self.data.as_mut_ptr())
                .unwrap()
                .add(self.start)
                .cast::<T>()
                .drop_in_place(self.end - self.start);
        }
    }
}

impl_traits! {
    for ArrayVec<T, N: [usize] [const]>
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
    Borrow<[T]> =>

        #[inline(always)]
        fn borrow(&self) -> &[T] {
            self.as_slice()
        }
    ,
    BorrowMut<[T]> =>

        #[inline(always)]
        fn borrow_mut(&mut self) -> &mut [T] {
            self.as_mut_slice()
        }
    ,
    From<&[T]> where T: Clone =>
        
        fn from(slice: &[T]) -> Self {
            let mut vec = Self::new();
            vec.append(slice);
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
    IntoIterator =>

        type Item = T;
        type IntoIter = IterArrayVec<T, N>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            unsafe {
                let s = MaybeUninit::new(self);
                let data: *const [_; N] = &s.assume_init_ref().data;
                IterArrayVec {
                    end: s.assume_init_ref().len,
                    data: data.read(),
                    start: 0,
                }
            }
        }
    ,
    Clone where T: Clone =>
    
        #[inline(always)]
        fn clone(&self) -> Self {
            let mut vec = ArrayVec::new();
            vec.append(self);
            vec
        }
    ,
    Eq where T: Eq =>,
    Debug where T: Debug =>

        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            self.as_slice().fmt(f)
        }
    ,
    Display where T: Display =>

        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            if self.len == 0 {
                <str as Display>::fmt("[]", f)?;
                return Ok(())
            }
            <char as Display>::fmt(&'[', f)?;
            for value in &self.as_slice()[0..self.len - 1] {
                value.fmt(f)?;
                <str as Display>::fmt(", ", f)?;
            }
            self.last().unwrap().fmt(f)?;
            <char as Display>::fmt(&']', f)
        }
    ,
    Hash where T: Hash =>

        #[inline(always)]
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_slice().hash(state)
        }
    ,
}

impl<T, A, const N: usize> PartialEq<T> for ArrayVec<A, N>
    where
        T: AsRef<[A]>,
        A: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T, const N: usize> PartialEq<ArrayVec<T, N>> for [T]
    where
        T: PartialEq
{

    fn eq(&self, other: &ArrayVec<T, N>) -> bool {
        self == other.as_ref()
    }
}

impl<T, const N: usize> PartialEq<ArrayVec<T, N>> for &[T]
    where
        T: PartialEq
{

    fn eq(&self, other: &ArrayVec<T, N>) -> bool {
        *self == other.as_ref()
    }
}

impl<const N: usize, T> From<[T; N]> for ArrayVec<T, N> {
    
    fn from(value: [T; N]) -> Self {
        let mut vec = ArrayVec::new();
        for v in value {
            vec.push(v);
        }
        vec
    }
}
