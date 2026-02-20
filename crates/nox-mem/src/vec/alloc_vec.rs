use core::{
    marker::PhantomData,
    slice::{self, Iter, IterMut},
    ops::{Deref, DerefMut},
    hash::{Hash, Hasher},
    fmt::{Debug, Display},
    mem::ManuallyDrop,
    borrow::{Borrow, BorrowMut},
    result,
};

use crate::{
    collections::{TryReserveError, ReservePolicy},
    num::{Integer, FromUsize, IntoUsize},
    alloc::{LocalAlloc, LocalAllocExt, LocalAllocWrap},
    conditional::{Conditional, False},
    impl_traits,
};

use super::{
    Vector,
    FallibleVec,
    Pointer,
    NonNullVecBase,
};

pub struct DynPolicy;

unsafe impl ReservePolicy for DynPolicy {

    #[inline(always)]
    fn can_grow() -> bool {
        true
    }

    #[inline(always)]
    fn grow(current: usize, required: usize) -> Result<usize, TryReserveError<()>> {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 < current { Ok(current) }
        else { Ok(power_of_2) }
    }

    #[inline(always)]
    fn grow_infallible(current: usize, required: usize) -> usize {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 < current { current }
        else { power_of_2 }
    }
}

pub struct FixedPolicy;

unsafe impl ReservePolicy for FixedPolicy {

    #[inline(always)]
    fn can_grow() -> bool {
        false
    }

    #[inline]
    fn grow(current: usize, requested: usize) -> Result<usize, TryReserveError<()>> {
        if requested <= current {
            Ok(current)
        } else {
            Err(TryReserveError::max_capacity_exceeded(current, requested, ()))
        }
    }

    #[inline]
    fn grow_infallible(current: usize, required: usize) -> usize {
        if required > current.into_usize() { panic!("maximum capacity of {current} exceeded") }
        current
    }
}

pub struct DynPolicy32;

pub struct FixedPolicy32;

unsafe impl ReservePolicy<u32> for DynPolicy32 {

    fn can_grow() -> bool {
        true
    }

    #[inline(always)]
    fn grow(current: u32, required: usize) -> core::result::Result<u32, TryReserveError<()>> {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 > u32::MAX as usize {
            Err(TryReserveError::max_capacity_exceeded(u32::MAX as usize, power_of_2, ()))
        } else if power_of_2 <= current as usize {
            Ok(current)
        } else { Ok(power_of_2.max(2) as u32) }
    }

    #[inline(always)]
    fn grow_infallible(current: u32, required: usize) -> u32 {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 > u32::MAX as usize || power_of_2 <= current as usize {
            current
        } else {
            power_of_2.max(2) as u32
        }
    }
}

unsafe impl ReservePolicy<u32> for FixedPolicy32 {

    #[inline(always)]
    fn can_grow() -> bool {
        false
    }

    #[inline]
    fn grow(current: u32, requested: usize) -> Result<u32, TryReserveError<()>> {
        if requested <= current as usize {
            Ok(current)
        } else {
            Err(TryReserveError::max_capacity_exceeded(current, requested, ()))
        }
    }

    #[inline]
    fn grow_infallible(current: u32, required: usize) -> u32 {
        if required > current.into_usize() { panic!("maximum capacity of {current} exceeded") }
        current
    }
}

pub struct AllocVec<T, Alloc, ReservePol, IsStd, SizeType = usize>
    where
        T: Sized,
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{
    data: Pointer<T, SizeType>,
    capacity: SizeType,
    len: SizeType,
    alloc: Alloc,
    _markers: PhantomData<(ReservePol, IsStd)>,
}

pub type AllocVecBase<T, SizeType, Alloc, ReservePol> =
    AllocVec<T, Alloc, ReservePol, False, SizeType>;

pub type DynVec<'a, T, Alloc> = AllocVecBase<T, usize, LocalAllocWrap<Alloc, &'a Alloc>, DynPolicy>;
pub type DynVec32<'a, T, Alloc> = AllocVecBase<T, u32, LocalAllocWrap<Alloc, &'a Alloc>, DynPolicy32>;

pub type FixedVec<'a, T, Alloc> = AllocVecBase<T, usize, LocalAllocWrap<Alloc, &'a Alloc>, FixedPolicy>;
pub type FixedVec32<'a, T, Alloc> = AllocVecBase<T, u32, LocalAllocWrap<Alloc, &'a Alloc>, FixedPolicy32>;

impl<T, Alloc, Wrap, ReservePol, SizeType> AllocVec<T, LocalAllocWrap<Alloc, Wrap>, ReservePol, False, SizeType>
    where
        T: Sized,
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
        ReservePol: ReservePolicy<SizeType>,
        SizeType: IntoUsize + FromUsize
{

    #[inline(always)]
    pub fn new(alloc: Wrap) -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: SizeType::ZERO,
            len: SizeType::ZERO,
            alloc: LocalAllocWrap::new(alloc),
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: SizeType,
        alloc: Wrap,
    ) -> Result<Self, Alloc::Error> {
        if capacity == SizeType::ZERO {
            return Ok(Self::new(alloc))
        }
        let capacity = ReservePol::grow_infallible(capacity, capacity.into_usize());
        let data = unsafe { alloc
            .allocate_uninit(capacity.into_usize())?
        }.into();
        Ok(Self {
            data,
            capacity,
            len: SizeType::ZERO,
            alloc: LocalAllocWrap::new(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_len(
        len: SizeType,
        value: T,
        alloc: Wrap,
    ) -> Result<Self, Alloc::Error>
        where
            T: Clone
    {
        if len == SizeType::ZERO {
            return Ok(Self::new(alloc))
        }
        let capacity = ReservePol::grow_infallible(len, len.into_usize());
        let data: Pointer<T, SizeType> = unsafe { alloc
            .allocate_uninit(capacity.into_usize())?
        }.into();
        for i in 0..len.into_usize() {
            unsafe { data.add(i).write(value.clone()) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: LocalAllocWrap::new(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_len_with<F>(
        len: SizeType,
        mut f: F,
        alloc: Wrap,
    ) -> Result<Self, Alloc::Error>
        where F: FnMut(SizeType) -> T
    {
        if len == SizeType::ZERO {
            return Ok(Self::new(alloc))
        }
        let capacity = ReservePol::grow_infallible(len, len.into_usize());
        let data: Pointer<T, SizeType> = unsafe { alloc
            .allocate_uninit(capacity.into_usize())?
            .into()
        };
        for i in 0..len.into_usize() {
            unsafe { data.add(i).write(f(SizeType::from_usize_unchecked(i))) };
        }
        Ok(Self {
            data,
            capacity,
            len,
            alloc: LocalAllocWrap::new(alloc),
            _markers: PhantomData,
        })
    }
}

impl<T, Alloc, ReservePol, IsStd, SizeType> AllocVec<T, Alloc, ReservePol, IsStd, SizeType>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{

    /// Creates a new [`AllocVec`] from raw parts.
    /// 
    /// This function should only be used when implementing a custom [`AllocVec`] type.
    ///
    /// # Safety
    /// There are a multitude of reasons why this is unsafe. For example, the pointer needs to be valid up to
    /// capacity, the pointer needs to be a valid pointer for the given allocator and length needs to be
    /// less than capacity.
    #[inline(always)]
    pub unsafe fn from_raw_parts(
        data: Pointer<T, SizeType>,
        len: SizeType,
        capacity: SizeType,
        alloc: Alloc,
    ) -> Self {
        Self {
            data,
            len,
            capacity,
            alloc,
            _markers: PhantomData,
        }
    }
    
    /// Changes the [`LocalAlloc`] of the [`AllocVec`].
    ///
    /// This function should only be used when implementing a custom [`AllocVec`] type.
    ///
    /// # Safety
    /// The inner pointer of the source vector must be valid for the new allocator.
    #[inline(always)]
    pub unsafe fn with_alloc<A: LocalAlloc>(self, alloc: A) -> AllocVecBase<T, SizeType, A, ReservePol> {
        let vec = ManuallyDrop::new(self);
        AllocVec {
            data: vec.data,
            capacity: vec.capacity,
            len: vec.len,
            alloc,
            _markers: PhantomData
        }
    }

    #[inline(always)]
    pub unsafe fn into_non_null(self) -> NonNullVecBase<'static, T, SizeType> {
        let vec = ManuallyDrop::new(self);
        unsafe {
            NonNullVecBase::new(*vec.data, vec.capacity)
            .with_len(vec.len)
        }
    }
    
    /// Sets the capacity of [`AllocVec`].
    ///
    /// This function should only be used when implementing a custom [`AllocVec`] type.
    ///
    /// # Safety
    /// The inner pointer needs to be valid up to the `capacity` and `capacity` needs to be greater
    /// than or equal to the vector's length (which can be set via [`AllocVec::set_len`]).
    #[inline(always)]
    pub unsafe fn set_capacity(&mut self, capacity: SizeType) {
        self.capacity = capacity;
    }

    /// Consumes self and returns the inner pointer.
    #[inline(always)]
    pub fn into_inner(self) -> Pointer<T, SizeType> {
        let vec = ManuallyDrop::new(self);
        vec.data
    }
}

pub struct IterAllocVec<T, SizeType: IntoUsize + FromUsize, Alloc: LocalAlloc> {
    data: Pointer<T, SizeType>,
    capacity: SizeType,
    start: SizeType,
    end: SizeType,
    alloc: Alloc,
}

impl<T, SizeType, Alloc> Iterator for IterAllocVec<T, SizeType, Alloc>
    where 
        SizeType: IntoUsize + FromUsize,
        Alloc: LocalAlloc,
{

    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start != self.end {
            let t = unsafe {
                self.data.add(self.start.into_usize()).read()
            };
            self.start = self.start.step_forward();
            Some(t)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.start.into_usize(), Some(self.end.into_usize()))
    }
}

impl<T, SizeType, Alloc> DoubleEndedIterator for IterAllocVec<T, SizeType, Alloc>
    where
        SizeType: IntoUsize + FromUsize,
        Alloc: LocalAlloc,
{

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start != self.end {
            self.end = self.end.step_backward();
            let t = unsafe {
                self.data.add(self.end.into_usize()).read()
            };
            Some(t)
        } else {
            None
        }
    }
}

impl<T, SizeType, Alloc> ExactSizeIterator for IterAllocVec<T, SizeType, Alloc>
    where
        SizeType: IntoUsize + FromUsize,
        Alloc: LocalAlloc,
{}

impl<T, SizeType, Alloc> Drop for IterAllocVec<T, SizeType, Alloc>
    where
        SizeType: IntoUsize + FromUsize,
        Alloc: LocalAlloc,
{

    fn drop(&mut self) {
        unsafe {
            if self.capacity == SizeType::ZERO { return }
            self.data
                .add(self.start.into_usize())
                .drop_in_place(self.end - self.start);
            self.alloc
                .free_uninit(*self.data, self.capacity.into_usize());
        }
    }
}

impl<T, Alloc, ReservePol, IsStd, SizeType> Vector<T, SizeType> for
    AllocVec<T, Alloc, ReservePol, IsStd, SizeType> where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{

    type Iter<'a> = slice::Iter<'a, T>
        where
            T: 'a, Self: 'a;

    type IterMut<'a> = slice::IterMut<'a, T>
        where
            T: 'a, Self: 'a;

    type ReservePol = ReservePol;

    #[inline(always)]
    fn len(&self) -> SizeType {
        self.len
    }

    #[inline(always)]
    fn capacity(&self) -> SizeType {
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
    fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len.into_usize()) }
    }

    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.len.into_usize()) }
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, len: SizeType) {
        if len > self.capacity { panic!("len was larger than capacity") }
        self.len = len;
    }

    #[inline(always)]
    fn reserve(&mut self, capacity: SizeType)
    {
        let capacity = ReservePol::grow_infallible(self.capacity, capacity.into_usize());
        self.reserve_exact(capacity);
    }

    fn reserve_exact(&mut self, capacity: SizeType) {
        if capacity <= self.capacity { return }
        if !ReservePol::can_grow() {
            panic!(
                "maximum capacity of {} exceeded, requested capacity was {}",
                self.capacity, capacity,
            )
        }
        let tmp = unsafe {
            self.alloc.allocate_uninit(capacity.into_usize())
        }.unwrap().into();
        debug_assert!(self.len <= self.capacity);
        unsafe {
            self.data.move_elements(tmp, self.len);
        }
        if self.capacity != SizeType::ZERO {
            unsafe { self.alloc.free_uninit(*self.data, self.capacity.into_usize()); }
        }
        self.data = tmp;
        self.capacity = capacity
    }

    fn resize(&mut self, len: SizeType, value: T)
        where
            T: Clone
    {
        if len > self.capacity {
            self.reserve(len);
        }
        if len > self.len {
            for i in self.len.into_usize()..len.into_usize() {
                unsafe { self.data.add(i).write(value.clone()) }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
    }

    fn resize_with<F>(&mut self, len: SizeType, mut f: F)
        where
            F: FnMut() -> T
    {
        if len > self.capacity {
            self.reserve(len);
        }
        if len > self.len {
            for i in self.len.into_usize()..len.into_usize() {
                unsafe { self.data.add(i).write(f()) }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
    }

    fn try_resize_with<F, E>(
        &mut self,
        len: SizeType,
        mut f: F,
    ) -> Result<(), E>
        where
            F: FnMut() -> result::Result<T, E>,
    {
        if len > self.capacity {
            self.reserve(len)
        }
        if len > self.len {
            for i in self.len.iter(len) {
                unsafe { self.data
                    .add(i.into_usize())
                    .write(f().inspect_err(|_| {
                        self.len = i;
                    })?)
                }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, value: T) {
        if self.len >= self.capacity {
            if self.capacity == SizeType::ZERO {
                self.reserve(SizeType::from_usize_unchecked(2));
            }
            else {
                let capacity = ReservePol
                    ::grow(
                        self.capacity,
                        self.capacity.into_usize() * 2
                    ).unwrap();
                self.reserve_exact(capacity);
            }
        }
        unsafe { self.data.add(self.len.into_usize()).write(value) };
        self.len = self.len.step_forward();
    }

    fn append(&mut self, slice: &[T])
        where
            T: Clone
    {
        let len = self.len.into_usize() + slice.len();
        let capacity = ReservePol
            ::grow(self.capacity, len)
            .unwrap();
        self.reserve_exact(capacity);
        unsafe {
            Pointer
                ::new(slice.as_ptr().cast_mut())
                .unwrap()
                .clone_elements(
                    self.data.add(self.len.into_usize()),
                    FromUsize::from_usize_unchecked(slice.len()),
                );
        }
        self.len = SizeType::from_usize_unchecked(len);
    }

    fn append_map<U, F>(&mut self, slice: &[U], mut f: F)
        where
            F: FnMut(&U) -> T
    {
        let len = self.len.into_usize() + slice.len();
        let capacity = ReservePol
            ::grow(self.capacity, len)
            .unwrap();
        self.reserve_exact(capacity);
        let len = self.len.into_usize();
        for (i, u) in slice.iter().enumerate() {
            unsafe {
                self.data.add(len + i).write(f(u));
            }
        }
        self.len = SizeType::from_usize_unchecked(len);
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        if self.len == SizeType::ZERO { return None }
        self.len = self.len.step_backward();
        let ptr = unsafe { self.data.add(self.len.into_usize()) };
        Some(unsafe { ptr.read() })
    }

    #[inline(always)]
    fn last(&self) -> Option<&T> {
        if self.len == SizeType::ZERO {
            None
        }
        else {
            unsafe {
                Some(
                    self.data.add(self.len.into_usize() - 1).as_ref()
                )
            }
        }
    }

    #[inline(always)]
    fn last_mut(&mut self) -> Option<&mut T> {
        if self.len == SizeType::ZERO {
            None
        }
        else {
            unsafe {
                Some(
                    self.data.add(self.len.into_usize() - 1).as_mut()
                )
            }
        }
    }

    #[inline(always)]
    fn insert(&mut self, index: SizeType, value: T) {
        if index > self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len >= self.capacity {
            let capacity = ReservePol
                ::grow(
                    self.capacity,
                    self.capacity.into_usize() * 2
                ).unwrap();
            self.reserve_exact(capacity);
        }
        unsafe {
            self.data.insert_element(value, index, self.len);
            self.len = self.len.step_forward();
        }
    }

    fn remove(&mut self, index: SizeType) -> T {
        if index >= self.len {
            panic!("index {} was out of bounds with len {} when removing", index, self.len);
        }
        let removed = unsafe { self.data.add(index.into_usize()).read() };
        for i in index.into_usize() ..self.len.into_usize() - 1 {
            unsafe { self.data.add(i).write(
                self.data.add(i + 1).read()
            )}
        }
        self.len = self.len.step_backward();
        removed
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: SizeType) -> T {
        if index == self.len {
            panic!("index {} was out of bounds with len {} when removing", index, self.len)
        }
        let removed = unsafe { self.data.add(index.into_usize()).read() };
        self.len = self.len.step_backward();
        if index != self.len {
            unsafe { self.data.add(index.into_usize()).write(
                self.data.add(self.len.into_usize()).read()
            )}
        }
        removed
    }

    fn clear(&mut self) {
        debug_assert!(self.len <= self.capacity);
        if self.capacity == SizeType::ZERO { return }
        unsafe {
            self.data.drop_in_place(self.len);
        }
        self.len = SizeType::ZERO;
    }

    fn move_from_vec<V, S>(&mut self, from: &mut V)
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize,
    {
        self.clear();
        let capacity = ReservePol
            ::grow(self.capacity, from.len().into_usize())
            .unwrap();
        self.reserve_exact(capacity);
        let len = SizeType::from_usize_unchecked(from.len().into_usize());
        unsafe {
            Pointer
                ::new(from.as_mut_ptr())
                .unwrap()
                .move_elements(self.data, len);
        }
        self.len = len;
        unsafe { from.set_len(S::ZERO); }
    }

    fn move_from_vec_map<U, V, S, F>(&mut self, from: &mut V, mut f: F)
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T
    {
        self.clear();
        let capacity = ReservePol
            ::grow(self.capacity, from.len().into_usize())
            .unwrap();
        self.reserve_exact(capacity);
        let len = from.len().into_usize();
        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        unsafe {
            for i in 0..len {
                dst.add(i)
                .write(f(src.add(i).read()))
            }
        }
        self.len = SizeType::from_usize_unchecked(len);
        unsafe { from.set_len(S::ZERO); }
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

impl<T, Alloc, ReservePol, IsStd, SizeType>
    FallibleVec<T, SizeType> for AllocVec<T, Alloc, ReservePol, IsStd, SizeType>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{

    #[inline(always)]
    fn fallible_reserve(&mut self, capacity: SizeType) -> Result<(), TryReserveError<()>> {
        let capacity = ReservePol::grow(self.capacity, capacity.into_usize())?;
        self.fallible_reserve_exact(capacity)
    }

    fn fallible_reserve_exact(&mut self, capacity: SizeType) -> Result<(), TryReserveError<()>> {
        if capacity <= self.capacity { return Ok(()) }
        if !ReservePol::can_grow() {
            return Err(TryReserveError::max_capacity_exceeded(self.capacity, capacity.into_usize(), ()))
        }
        let tmp = unsafe {
            self.alloc.allocate_uninit(capacity.into_usize())
        }.map_err(|err| TryReserveError::alloc_error(err, ()))?.into();
        debug_assert!(self.len <= self.capacity);
        unsafe {
            self.data.move_elements(tmp, self.len);
        }
        if self.capacity != SizeType::ZERO {
            unsafe { self.alloc.free_uninit(*self.data, self.capacity.into_usize()); }
        }
        self.data = tmp;
        self.capacity = capacity;
        Ok(())
    }

    fn fallible_resize(&mut self, len: SizeType, value: T) -> Result<(), TryReserveError<()>>
        where
            T: Clone
    {
        if len > self.capacity {
            self.fallible_reserve_exact(len)?;
        }
        if len > self.len {
            for i in self.len.into_usize()..len.into_usize() {
                unsafe { self.data.add(i).write(value.clone()) }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn fallible_resize_with<F>(&mut self, len: SizeType, mut f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut() -> T
    {
        if len > self.capacity {
            self.fallible_reserve(len)?;
        }
        if len > self.len {
            for i in self.len.into_usize()..len.into_usize() {
                unsafe { self.data.add(i).write(f()) }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn fallible_try_resize_with<F, E, MapE>(
        &mut self,
        len: SizeType,
        mut f: F,
        map_reserve_err: MapE,
    ) -> Result<(), E>
        where
            F: FnMut() -> Result<T, E>,
            MapE: FnMut(TryReserveError<()>) -> E
    {
        if len > self.capacity {
            self.fallible_reserve(len)
                .map_err(map_reserve_err)?;
        }
        if len > self.len {
            for i in self.len.into_usize()..len.into_usize() {
                unsafe { self.data
                    .add(i)
                    .write(f().inspect_err(|_| {
                        self.len = SizeType::from_usize_unchecked(i);
                    })?)
                }
            }
        }
        else if len < self.len {
            unsafe {
                self.data.add(len.into_usize()).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn fallible_push(&mut self, value: T) -> Result<(), TryReserveError<T>> {
        if self.len >= self.capacity {
            if self.capacity == SizeType::ZERO {
                if let Err(err) = self.fallible_reserve_exact(SizeType::from_usize_unchecked(2)) {
                    return Err(err.with_value(value))
                }
            }
            else {
                let capacity = match ReservePol
                    ::grow(
                        self.capacity,
                        self.capacity.into_usize() * 2
                    )
                {
                    Ok(c) => c,
                    Err(err) => return Err(err.with_value(value))
                };
                if let Err(err) = self.fallible_reserve_exact(capacity) {
                    return Err(err.with_value(value))
                }
            }
        }
        let ptr = unsafe { self.data.add(self.len.into_usize()) };
        unsafe { ptr.write(value) };
        self.len = self.len.step_forward();
        Ok(())
    }

    fn fallible_append(&mut self, slice: &[T]) -> Result<(), TryReserveError<()>>
        where
            T: Clone
    {
        let len = self.len.into_usize() + slice.len();
        let capacity = ReservePol::grow(self.capacity, len)?;
        self.fallible_reserve_exact(capacity)?;
        unsafe {
            Pointer
                ::new(slice.as_ptr().cast_mut())
                .unwrap()
                .clone_elements(
                    self.data.add(self.len.into_usize()),
                    SizeType::from_usize_unchecked(slice.len()),
                );
        }
        self.len = SizeType::from_usize_unchecked(len);
        Ok(())
    }

    fn fallible_append_map<U, F>(&mut self, slice: &[U], mut f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut(&U) -> T
    {
        let len = self.len.into_usize() + slice.len();
        let capacity = ReservePol::grow(self.capacity, len)?;
        self.fallible_reserve_exact(capacity)?;
        let len = self.len.into_usize();
        for (i, u) in slice.iter().enumerate() {
            unsafe {
                self.data.add(len + i).write(f(u));
            }
        }
        self.len = SizeType::from_usize_unchecked(len);
        Ok(())
    }

    fn fallible_insert(&mut self, index: SizeType, value: T) -> Result<(), TryReserveError<T>> {
        if index > self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if index == self.len && index == self.capacity {
            let capacity = match ReservePol
                ::grow(
                    self.capacity,
                    self.capacity.into_usize() * 2
                ) {
                Ok(c) => c,
                Err(err) => return Err(err.with_value(value))
            };
            if let Err(err) = self.fallible_reserve_exact(capacity) {
                return Err(err.with_value(value))
            }
        }
        unsafe {
            self.data.insert_element(value, index, self.len);
            self.len = self.len.step_forward();
            Ok(())
        }
    }

    fn fallible_move_from_vec<V, S>(&mut self, from: &mut V) -> Result<(), TryReserveError<()>>
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize,
    {
        self.clear();
        let len = from.len().into_usize();
        let capacity = ReservePol::grow(self.capacity, len)?;
        self.fallible_reserve_exact(capacity)?;
        let len = SizeType::from_usize_unchecked(len);
        unsafe {
            Pointer
                ::new(from.as_mut_ptr())
                .unwrap()
                .move_elements(self.data, len);
        }
        self.len = len;
        unsafe { from.set_len(S::ZERO); }
        Ok(())
    }

    fn fallible_move_from_vec_map<U, V, S, F>(
        &mut self,
        from: &mut V,
        mut f: F,
    ) -> Result<(), TryReserveError<()>>
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T
    {
        self.clear();
        let len = from.len().into_usize();
        let capacity = ReservePol::grow(self.capacity, len)?;
        self.fallible_reserve_exact(capacity)?;
        let src = from.as_ptr();
        let dst = self.as_mut_ptr();
        unsafe {
            for i in 0..len {
                dst.add(i)
                .write(f(src.add(i).read()))
            }
        }
        self.len = SizeType::from_usize_unchecked(len);
        unsafe { from.set_len(S::ZERO); }
        Ok(())
    }
}

impl_traits!{
    for AllocVec<
        T: [Sized],
        Alloc: [LocalAlloc],
        ReservePol: [ReservePolicy<SizeType>],
        IsStd: [Conditional],
        SizeType: [IntoUsize + FromUsize]
    >
    Drop =>

        #[inline(always)]
        fn drop(&mut self) {
            self.clear();
            if self.capacity != SizeType::ZERO {
                unsafe { self.alloc.free_uninit(
                    *self.data,
                    self.capacity.into_usize()
                )}
            }
            self.capacity = SizeType::ZERO;
            self.data = Pointer::dangling();
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
    IntoIterator for &'vec =>

        type Item = &'vec T;
        type IntoIter = Iter<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator for mut &'vec =>

        type Item = &'vec mut T;
        type IntoIter = IterMut<'vec, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
    IntoIterator => 

        type Item = T;
        type IntoIter = IterAllocVec<T, SizeType, Alloc>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            unsafe {
                let s = ManuallyDrop::new(self);
                let alloc_ptr: *const Alloc = &s
                    .alloc;
                IterAllocVec {
                    data: s.data,
                    capacity: s.capacity,
                    start: SizeType::ZERO,
                    end: s.len,
                    alloc: alloc_ptr.read(),
                }
            }
        }
    ,
    Eq where T: Eq =>
    ,
    Hash where T: Hash =>

        #[inline(always)]
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.as_slice().hash(state)
        }
    ,
    Debug where T: Debug =>

        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            self.as_slice().fmt(f)
        }
    ,
    Display where T: Display =>

        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self.len == SizeType::ZERO {
                <str as Display>::fmt("[]", f)?;
                return Ok(())
            }
            <char as Display>::fmt(&'[', f)?;
            for value in &self[0..self.len.into_usize().step_backward()] {
                value.fmt(f)?;
                <str as Display>::fmt(", ", f)?;
            }
            self.last().unwrap().fmt(f)?;
            <char as Display>::fmt(&']', f)
        }
    ,
}

impl<T, A, Alloc, ReservePol, IsStd, SizeType> PartialEq<T> 
    for AllocVec<A, Alloc, ReservePol, IsStd, SizeType>
    where
        T: AsRef<[A]>,
        A: PartialEq,
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T, Alloc, ReservePol, IsStd, SizeType> PartialEq<
        AllocVec<T, Alloc, ReservePol, IsStd, SizeType>
    > for [T]
    where
        T: PartialEq,
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{
    fn eq(&self, other: &AllocVec<T, Alloc, ReservePol, IsStd, SizeType>) -> bool {
        self == other.as_ref()
    }
}

impl<T, Alloc, ReservePol, IsStd, SizeType> PartialEq<
        AllocVec<T, Alloc, ReservePol, IsStd, SizeType>
    > for &[T]
    where
        T: PartialEq,
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<SizeType>,
        IsStd: Conditional,
        SizeType: IntoUsize + FromUsize,
{
    fn eq(&self, other: &AllocVec<T, Alloc, ReservePol, IsStd, SizeType>) -> bool {
        *self == other.as_ref()
    }
}

unsafe impl<
    Alloc: LocalAlloc + Send,
    T: Send,
    ReservePol: ReservePolicy<SizeType>,
    IsStd: Conditional,
    SizeType: IntoUsize + FromUsize,
> Send for AllocVec<T, Alloc, ReservePol, IsStd, SizeType> {}

unsafe impl<
    Alloc: LocalAlloc + Sync,
    T: Sync,
    ReservePol: ReservePolicy<SizeType>,
    IsStd: Conditional,
    SizeType: IntoUsize + FromUsize,
> Sync for AllocVec<T, Alloc, ReservePol, IsStd, SizeType> {}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    use crate::{
        alloc::StdAlloc,
        conditional::True,
    };

    pub type StdVecBase<T, SizeType, ReservePol> = AllocVec<T, StdAlloc, ReservePol, True, SizeType>;

    /// A vec that uses [`GlobalAlloc`].
    pub type StdVec<T> = StdVecBase<T, usize, DynPolicy>;

    /// A vec type with a tiny footprint on 64-bit systems, backed by [`GlobalAlloc`].
    ///
    /// Stores capacity and length as [`u32`] instead of [`usize`] resulting in the struct taking
    /// only 16 bytes on the stack on 64-bit systems.
    ///
    ///
    /// On 64-bit systems the size of [`Vec32<T>`] is equal to the size of [`Box<\[T\]>`];
    ///
    /// This was mainly made for Vulkan usage, since Vulkan often uses u32 for counts.
    ///
    /// Maximum capacity is restricted to be equal to [`u32::MAX`].
    pub type Vec32<T> = StdVecBase<T, u32, DynPolicy32>;

    impl_traits!{
        for StdVecBase<T, SizeType: [IntoUsize + FromUsize], ReservePol: [ReservePolicy<SizeType>]>
        Default =>
            #[inline(always)]
            fn default() -> Self {
                StdVecBase::new()
            }
        ,
        Clone where T: Clone =>

            #[inline(always)]
            fn clone(&self) -> Self {
                let mut clone = StdVecBase::with_capacity(self.capacity);
                unsafe {
                    self.data.clone_elements(clone.data, self.len);
                }
                clone.len = self.len;
                clone
            }
        ,
        From<&[T]> where T: Clone =>
            
            #[inline(always)]
            fn from(value: &[T]) -> Self {
                let len =  SizeType::from_usize_unchecked(value.len());
                let mut vec = StdVecBase::with_capacity(len);
                unsafe {
                    Pointer
                        ::new(value.as_ptr().cast_mut())
                        .unwrap()
                        .clone_elements(vec.data, len);
                }
                vec.len = len;
                vec
            }
        ,
    }

    impl<const N: usize, T, SizeType, ReservePol> From<[T; N]>
        for StdVecBase<T, SizeType, ReservePol>
        where
            SizeType: IntoUsize + FromUsize,
            ReservePol: ReservePolicy<SizeType>,
    {
        
        fn from(value: [T; N]) -> Self {
            let mut vec = StdVecBase::with_capacity(SizeType::from_usize(N).unwrap());
            for v in value {
                vec.push(v);
            }
            vec
        }
    }

    impl<A, SizeType, ReservePol> FromIterator<A> for StdVecBase<A, SizeType, ReservePol>
        where 
            SizeType: IntoUsize + FromUsize,
            ReservePol: ReservePolicy<SizeType>,
    {

        fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
            let mut vec = Self::new();
            vec.extend(iter);
            vec
        }
    }

    impl<T, SizeType, ReservePol> StdVecBase<T, SizeType, ReservePol>
        where 
            SizeType: IntoUsize + FromUsize,
            ReservePol: ReservePolicy<SizeType>,
    {
        
        /// Creates an empty vector with zero capacity.
        #[inline(always)]
        pub fn new() -> Self {
            Self {
                data: Pointer::dangling(),
                capacity: SizeType::ZERO,
                len: SizeType::ZERO,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }

        /// Creates an empty vector with at least the given capacity.
        ///
        /// The capacity of the vector might be more than what is given. If you want a vector with an
        /// exacit capacity, use the `with_exact_capacity` instead.
        pub fn with_capacity(
            capacity: SizeType,
        ) -> Self {
            if capacity == SizeType::ZERO {
                return Default::default()
            }
            let capacity = ReservePol::grow_infallible(SizeType::ZERO, capacity.into_usize());
            let data = unsafe { StdAlloc
                .allocate_uninit(capacity.into_usize())
                .expect("global alloc failed").into()
            };
            Self {
                data,
                capacity,
                len: SizeType::ZERO,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }

        /// Creates an empty vector with at the exact capacity specified.
        pub fn with_exact_capacity(
            capacity: SizeType,
        ) -> Self {
            if capacity == SizeType::ZERO {
                return Default::default()
            }
            let data = unsafe { StdAlloc
                .allocate_uninit(capacity.into_usize())
                .expect("global alloc failed").into()
            };
            Self {
                data,
                capacity,
                len: SizeType::ZERO,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }

        pub fn with_len(
            len: SizeType,
            value: T,
        ) -> Self
            where
                T: Clone
        {
            if len == SizeType::ZERO {
                return Default::default()
            }
            let capacity = ReservePol::grow_infallible(SizeType::ZERO, len.into_usize());
            let data: Pointer<T, SizeType> = unsafe { StdAlloc 
                .allocate_uninit(capacity.into_usize())
                .expect("global alloc failed").into()
            };
            for i in 0..len.into_usize() {
                unsafe { data.add(i).write(value.clone()) };
            }
            Self {
                data,
                capacity,
                len,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }

        pub fn with_len_reserve_exact(
            len: SizeType,
            value: T,
        ) -> Self
            where
                T: Clone
        {
            if len == SizeType::ZERO {
                return Default::default()
            }
            let capacity = len;
            let data: Pointer<T, SizeType> = unsafe { StdAlloc 
                .allocate_uninit(capacity.into_usize())
                .expect("global alloc failed").into()
            };
            for i in 0..len.into_usize() {
                unsafe { data.add(i).write(value.clone()) };
            }
            Self {
                data,
                capacity,
                len,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }

        pub fn with_len_with<F>(
            len: SizeType,
            mut f: F,
        ) -> Self
            where
                F: FnMut(SizeType) -> T,
        {
            if len == SizeType::ZERO {
                return Default::default()
            }
            let capacity = ReservePol::grow_infallible(SizeType::ZERO, len.into_usize());
            let data: Pointer<T, SizeType> = unsafe { StdAlloc
                .allocate_uninit(capacity.into_usize())
                .expect("global alloc failed").into()
            };
            for i in 0..len.into_usize() {
                unsafe { data.add(i).write(f(SizeType::from_usize_unchecked(i))) };
            }
            Self {
                data,
                capacity,
                len,
                alloc: StdAlloc,
                _markers: PhantomData,
            }
        }
    }


    impl<T, Alloc, ReservePol, IsStd, SizeType> From<&AllocVec<T, Alloc, ReservePol, IsStd, SizeType>>
        for std::vec::Vec<T>
        where
            T: Clone,
            Alloc: LocalAlloc,
            ReservePol: ReservePolicy<SizeType>,
            IsStd: Conditional,
            SizeType: IntoUsize + FromUsize,
    {

        #[inline(always)]
        fn from(value: &AllocVec<T, Alloc, ReservePol, IsStd, SizeType>) -> Self {
            value.to_vec()
        }
    }
}

#[cfg(feature = "std")]
pub use std_features::*;
