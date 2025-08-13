use core::{
    marker::PhantomData,
    ptr::NonNull,
    slice::{self, Iter, IterMut},
    ops::{Deref, DerefMut},
    hash::{Hash, Hasher},
    fmt::{Debug, Display},
};

use crate::{
    allocator::Allocator,
    capacity_policy::{CapacityPolicy, Dyn, Fixed},
    conditional::{Conditional, False, True},
    errors::CapacityError,
    global_alloc::{GlobalAlloc, GLOBAL_ALLOC},
    impl_traits,
    option_alloc::OptionAlloc,
};

use super::{
    Vector,
    Pointer,
};

use CapacityError::{FixedCapacity, InvalidReservation, AllocFailed, ZeroSizedElement};

type Result<T> = core::result::Result<T, CapacityError>;

pub struct AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        T: Sized,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{
    data: Pointer<T>,
    capacity: usize,
    len: usize,
    alloc: OptionAlloc<'alloc, Alloc>,
    _markers: PhantomData<(CapacityPol, IsGlobal)>,
}

pub type AllocVecImpl<'alloc, T, Alloc, CapacityPol> = AllocVec<'alloc, T, Alloc, CapacityPol, False>;
pub type DynVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, Dyn, False>;
pub type FixedVec<'alloc, T, Alloc> = AllocVec<'alloc, T, Alloc, Fixed, False>;
pub type GlobalVec<T> = AllocVec<'static, T, GlobalAlloc, Dyn, True>;

impl<'alloc, T, Alloc, CapacityPol> AllocVec<'alloc, T, Alloc, CapacityPol, False>
    where
        T: Sized,
        CapacityPol: CapacityPolicy,
        Alloc: Allocator,
{

    #[inline(always)]
    pub fn new(alloc: &'alloc Alloc) -> Option<Self> {
        if !CapacityPol::can_grow() {
            return None
        }
        Some(Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::Some(alloc),
            _markers: PhantomData,
        })
    }

    pub fn with_no_alloc() -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::None,
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: usize,
        alloc: &'alloc Alloc,
    ) -> Result<Self> {
        if capacity == 0 {
            if CapacityPol::can_grow() {
                return Ok(Self::new(alloc).unwrap())
            }
            return Ok(Self::with_no_alloc())
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
            })?.into()
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
    ) -> Result<Self>
        where
            T: Clone
    {
        if len == 0 {
            if CapacityPol::can_grow() {
                return Ok(Self::new(alloc).unwrap())
            }
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
        let data: Pointer<T> = unsafe { alloc
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?.into()
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
    ) -> Result<Self>
    {
        if len == 0 {
            if CapacityPol::can_grow() {
                return Ok(Self::new(alloc).unwrap())
            }
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
        let data: Pointer<T> = unsafe { alloc
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            })?.into()
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

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    pub fn with_capacity(
        capacity: usize,
    ) -> Self {
        if capacity == 0 {
            return Default::default()
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
            }).unwrap().into()
        };
        Self {
            data,
            capacity: true_capacity,
            len: 0,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    pub fn with_len(
        len: usize,
        value: T,
    ) -> Self
        where
            T: Clone
    {
        if len == 0 {
            return Default::default()
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data: Pointer<T> = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            }).unwrap().into()
        };
        for i in 0..len {
            unsafe { data.add(i).write(value.clone()) };
        }
        Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    pub fn with_len_with<F>(
        len: usize,
        mut f: F,
    ) -> Self
        where
            F: FnMut() -> T,
    {
        if len == 0 {
            return Default::default()
        }
        let capacity =
            if <Self as Vector<T>>::CapacityPol::power_of_two() {
                len.next_power_of_two()
            }
            else {
                len
            };
        let data: Pointer<T> = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity)
            .ok_or_else(|| {
                if size_of::<T>() == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity }
                }
            }).unwrap().into()
        };
        for i in 0..len {
            unsafe { data.add(i).write(f()) };
        }
        Self {
            data,
            capacity,
            len,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _markers: PhantomData,
        }
    }

    #[inline(always)]
    pub fn reserve(&mut self, capacity: usize) {
        <Self as Vector<T>>::reserve(self, capacity).unwrap()
    }

    #[inline(always)]
    pub fn push(&mut self, value: T) -> &mut T {
        <Self as Vector<T>>::push(self, value).unwrap()
    }

    #[inline(always)]
    pub fn resize(&mut self, len: usize, value: T)
        where
            T: Clone
    {
        <Self as Vector<T>>::resize(self, len, value).unwrap()
    }

    #[inline(always)]
    pub fn append(&mut self, slice: &[T])
        where
            T: Clone
    {
        <Self as Vector<T>>::append(self, slice).unwrap()
    }

    #[inline(always)]
    pub fn append_map<U, F>(&mut self, slice: &[U], f: F)
        where
            F: FnMut(&U) -> T
    {
        <Self as Vector<T>>::append_map(self, slice, f).unwrap()
    }
}

impl<'alloc, T, Alloc, CapacityPol, IsGlobal> Vector<T> for AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{

    type Iter<'a> = slice::Iter<'a, T>
        where
            T: 'a, Self: 'a;

    type IterMut<'a> = slice::IterMut<'a, T>
        where
            T: 'a, Self: 'a;

    type CapacityPol = CapacityPol;

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
        *self.data
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

    fn reserve(&mut self, capacity: usize) -> Result<()>
    {
        if !CapacityPol::can_grow() {
            return Err(FixedCapacity { capacity: self.capacity })
        }
        let new_capacity = match CapacityPol::grow(self.capacity, capacity) {
            Some(r) => r,
            None => return Err(InvalidReservation { current: self.capacity, requested: capacity }),
        };
        let tmp = match unsafe { self.alloc.allocate_uninit(new_capacity) } {
            Some(r) => r.into(),
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
            self.data.move_elements(tmp, self.len);
        }
        if self.capacity != 0 {
            unsafe { self.alloc.free_uninit(*self.data, self.capacity); }
        }
        self.data = tmp;
        self.capacity = new_capacity;
        Ok(())
    }

    fn resize(&mut self, len: usize, value: T) -> Result<()> 
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
                self.data.add(len).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn resize_with<F>(&mut self, len: usize, mut f: F) -> Result<()>
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
                self.data.add(len).drop_in_place(self.len - len);
            }
        }
        self.len = len;
        Ok(())
    }

    fn append(&mut self, slice: &[T]) -> Result<()>
        where
            T: Clone
    {
        let new_len = self.len + slice.len();
        if new_len > self.capacity {
            self.reserve(new_len)?;
        }
        unsafe {
            Pointer
                ::new(slice.as_ptr() as _)
                .unwrap()
                .clone_elements(
                    self.data.add(self.len),
                    slice.len(),
                );
        }
        self.len = new_len;
        Ok(())
    }

    fn append_map<U, F>(&mut self, slice: &[U], mut f: F) -> Result<()>
        where
            F: FnMut(&U) -> T
    {
        let new_len = self.len + slice.len();
        if new_len > self.capacity {
            self.reserve(new_len)?;
        }
        let len = self.len;
        for (i, u) in slice.iter().enumerate() {
            unsafe {
                self.data.add(len + i).write(f(u));
            }
        }
        self.len = new_len;
        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, value: T) -> Result<&mut T> {
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

    fn insert(&mut self, value: T, index: usize) -> Result<&mut T> {
        if index > self.len {
            panic!("index {} was out of bounds with len {} when inserting", index, self.len)
        }
        if self.len >= self.capacity {
            self.reserve(self.capacity * 2)?;
        }
        unsafe {
            let mut ptr = self.data.insert_element(value, index, self.len);
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
            self.data.drop_in_place(self.len);
        }
        unsafe { self.alloc.free_uninit(
            *self.data,
            self.capacity)
        }
        self.len = 0;
        self.capacity = 0;
        self.data = Pointer::dangling();
    }

    fn clone_from(mut self, from: &[T]) -> Result<Self>
        where
            T: Clone
    {
        if self.capacity < from.len() {
            if !CapacityPol::can_grow() {
                return Err(FixedCapacity { capacity: self.capacity })
            }
            self.clear();
            self.reserve(from.len())?
        }
        else {
            unsafe { self.data.drop_in_place(self.len); }
            self.len = 0;
        }
        unsafe {
            Pointer
                ::new(from.as_ptr() as _)
                .unwrap()
                .clone_elements(self.data, from.len());
        }
        self.len = from.len();
        Ok(self)
    }

    fn move_from<V>(mut self, from: &mut V) -> Result<Self>
        where
            V: Vector<T>
    {
        if self.capacity < from.len() {
            self.clear();
            self.reserve(from.len())?
        }
        else {
            unsafe { self.data.drop_in_place(self.len); }
            self.len = 0;
        }
        let slice = from.as_mut_slice();
        unsafe {
            Pointer
                ::new(slice.as_mut_ptr())
                .unwrap()
                .move_elements(self.data, slice.len());
        }
        self.len = slice.len();
        unsafe { from.set_len(0); }
        Ok(self)
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
    fn iter(&self) -> Self::Iter<'_> {
        self.as_slice().iter()
    }

    #[inline(always)]
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        self.as_mut_slice().iter_mut()
    }
}

impl_traits!{
    for AllocVec<'alloc, T, Alloc: Allocator, CapacityPol: CapacityPolicy, IsGlobal: Conditional>
    Drop =>

        #[inline(always)]
        fn drop(&mut self) {
            self.clear()
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
    PartialEq where T: PartialEq =>

        fn eq(&self, other: &Self) -> bool {
            if self.len != other.len {
                return false
            }
            for (i, value) in self.iter().enumerate() {
                if value != &other[i] {
                    return false
                }
            }
            return true
        }
    ,
    Eq where T: Eq =>
    ,
    Hash where T: Hash =>

        fn hash<H: Hasher>(&self, state: &mut H) {
            self.len.hash(state);
            for value in self {
                value.hash(state);
            }
        }
    ,
    Debug where T: Debug =>

        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            self.as_slice().fmt(f)
        }
    ,
    Display where T: Display =>

        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <char as Display>::fmt(&'[', f)?;
            for value in &self[0..self.len() - 1] {
                value.fmt(f)?;
                <str as Display>::fmt(&", ", f)?;
            }
            self.back().unwrap().fmt(f)?;
            <char as Display>::fmt(&']', f)
        }
    ,
}

impl<'alloc, T, Alloc, CapacityPol, IsGlobal> From<&AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>> for Vec<T>
    where
        T: Clone,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{

    #[inline(always)]
    fn from(value: &AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal>) -> Self {
        value.to_vec()
    }
}

impl_traits!{
    for AllocVecImpl<'alloc, T, Alloc: Allocator, CapacityPol: CapacityPolicy>
    Default =>
        fn default() -> Self {
            Self::with_no_alloc()
        }
    ,
}

impl_traits!{
    for GlobalVec<T>
    Default =>
        #[inline(always)]
        fn default() -> Self {
            GlobalVec::new()
        }
    ,
    Clone where T: Clone =>

        #[inline(always)]
        fn clone(&self) -> Self {
            let mut clone = GlobalVec::with_capacity(self.capacity);
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
            let len = value.len();
            let mut vec = GlobalVec::with_capacity(len);
            unsafe {
                Pointer
                    ::new(value.as_ptr() as _)
                    .unwrap()
                    .clone_elements(vec.data, len);
            }
            vec.len = len;
            vec
        }
    ,
}

unsafe impl<
    'alloc,
    Alloc: Allocator + Send,
    T: Send,
    CapacityPol: CapacityPolicy,
    IsGlobal: Conditional
> Send for AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal> {}

unsafe impl<
    'alloc,
    Alloc: Allocator + Sync,
    T: Sync,
    CapacityPol: CapacityPolicy,
    IsGlobal: Conditional
> Sync for AllocVec<'alloc, T, Alloc, CapacityPol, IsGlobal> {}
