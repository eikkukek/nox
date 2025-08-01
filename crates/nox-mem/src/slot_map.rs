//! A compact slot map implementation with support for custom allocators.
//!
//! Slot map is a data structure that associates values with *opaque, stable handles* (indices).
//! Unlike `Vec`, removal doesn't shift elements, and re-insertion may reuse free slots.
//!
//! This module provides:
//!
//! [`DynSlotMap<'alloc, T, Alloc: Allocator>`]: generic, allocator-aware base
//! [`FixedSlotMap<'alloc, T, Alloc: Allocator>`]: [`DynSlotMap`] with a fixed-capacity
//! [`GlobalSlotMap<'alloc, T, Alloc: Allocator>`]: [`DynSlotMap`] using [`GlobalAlloc`]
//!
//! # Features
//!
//! - Constant-time insertion, removal and lookup
//! - Stable handles
//! - Custom allocators
//! - No 'unsafe' in public API
//!
//! # Examples
//!
//! ```rust
//! use nox_mem::slot_map::GlobalSlotMap;
//!
//! let mut map = GlobalSlotMap::new();
//! let key1 = map.insert("hello");
//! let key2 = map.insert("world");
//! assert_eq!(map.try_get(key1), Some(&"hello"));
//! map.remove(key1);
//! assert_eq!(map.try_get(key1), None);
//! assert_eq!(map.try_get(key2), Some(&"world"));
//! ```

use core::{
    marker::PhantomData,
    mem::{needs_drop, MaybeUninit},
    num::NonZeroU32,
    ops::{Index, IndexMut},
};

use crate::{
    capacity_policy::CapacityPolicy,
    conditional::{Conditional, False, True},
    global_alloc::{GlobalAlloc, GLOBAL_ALLOC},
    vec_types::Pointer,
    Allocator,
    CapacityError,
    OptionAlloc,
    size_of,
    impl_traits,
};

type Result<T> = core::result::Result<T, CapacityError>;

use CapacityError::{ FixedCapacity, InvalidReservation, AllocFailed, ZeroSizedElement };

pub struct Dyn {}

impl CapacityPolicy for Dyn {

    fn power_of_two() -> bool {
        true
    }

    fn can_grow() -> bool {
        true
    }

    fn grow(current: usize, required: usize) -> Option<usize> {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 <= current || power_of_2 > u32::MAX as usize { None }
        else  { Some(power_of_2.max(2)) }
    }
}

pub struct Fixed {}

impl CapacityPolicy for Fixed {

    fn power_of_two() -> bool {
        false
    }

    fn can_grow() -> bool {
        false
    }

    fn grow(_: usize, _: usize) -> Option<usize> {
        None
    }
}

struct Slot<T> {
    value: MaybeUninit<T>,
    version: u32,
    next_free_index: Option<Option<u32>>,
}

impl<T> Slot<T> {

    fn empty(next_free_index: Option<u32>) -> Self {
        Self {
            value: MaybeUninit::uninit(),
            version: 1,
            next_free_index: Some(next_free_index),
        }
    }
}

#[derive(Debug)]
pub struct SlotIndex<T> {
    version: NonZeroU32,
    index: u32,
    _marker: PhantomData<T>,
}

impl<T> Clone for SlotIndex<T> {

    fn clone(&self) -> Self {
        Self {
            version: self.version,
            index: self.index,
            _marker: self._marker,
        }
    }
}

impl<T> Copy for SlotIndex<T> {}

impl<T> PartialEq for SlotIndex<T> {

    fn eq(&self, other: &Self) -> bool {
        self.version == other.version &&
        self.index == other.index
    }
}

impl<T> Eq for SlotIndex<T> {}

pub struct AllocSlotMap<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        T: Sized,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional,
{
    data: Pointer<Slot<T>>,
    capacity: u32,
    len: u32,
    free_head: Option<u32>,
    alloc: OptionAlloc<'alloc, Alloc>,
    _marker: PhantomData<(IsGlobal, CapacityPol)>,
}

/// A dynamic slot map storing values of type `T`, backed by allocator 'Alloc'.
///
/// Provides stable, opaque handles for accessing values. Removal leaves versioned
/// empty slots and insertions reuse free slots.
///
/// See also [`GlobalSlotMap`] for a version using [`GlobalAlloc`].
///
/// # Type parameters
///
/// - `T`: value type
/// - `Alloc`: allocator implementing [`Allocator`]
///
/// # Errors
///
/// Insertion and reservations return `Result` and may fail due to:
///
/// - Allocation failure
/// - Capacity reaching over u32::MAX
///
/// # Safety
///
/// The allocator must return valid pointers that are aligned to the requested alignment
/// and that are not freed or overwritten as long as the slot map uses them.
///
/// # Example
///
/// ```rust
/// let allocator = MyAllocator::default();
/// let mut map = DynSlotMap::new(&allocator);
/// let key = map.insert("value").unwrap();
/// map.remove(key);
pub type DynSlotMap<'alloc, T, Alloc> = AllocSlotMap<'alloc, T, Alloc, Dyn, False>;

/// A fixed-capacity slot map storing values of type `T`, backed by allocator `Alloc`.
///
/// Provides stable, opaque handles for accessing values. Removal leaves versioned
/// empty slots and insertions reuse free slots.
///
/// See also [`DynSlotMap`] for a version with dynamic capacity.
///
/// # Type parameters
///
/// - `T`: value type
/// - `Alloc`: allocator implementing [`Allocator`]
///
/// # Errors
///
/// Insertion and reservations return `Result` and may fail due to:
///
/// - Allocation failure
/// - Capacity reaching over u32::MAX
/// - Reaching fixed capacity
///
/// # Safety
///
/// The allocator must return valid pointers that are aligned to the requested alignment
/// and that are not freed or overwritten as long as the slot map uses them.
///
/// # Example
///
/// ```rust
/// let allocator = MyAllocator::default();
/// let mut map = FixedSlotMap::new(8, &allocator).unwrap();
/// let key = map.insert("value").unwrap();
/// map.remove(key);
/// ```
pub type FixedSlotMap<'alloc, T, Alloc> = AllocSlotMap<'alloc, T, Alloc, Fixed, False>;

/// A dynamic slot map storing values of type `T`, backed by [`GlobalAlloc`].
///
/// # Type parameters
///
/// - `T`: value type
///
/// # Example
///
/// ```rust
/// use nox_mem::slot_map::GlobalSlotMap;
///
/// let mut map = GlobalSlotMap::new();
/// let key1 = map.insert("hello");
/// let key2 = map.insert("world");
/// assert_eq!(map.try_get(key1), Some(&"hello"));
/// map.remove(key1);
/// assert_eq!(map.try_get(key1), None);
/// assert_eq!(map.try_get(key2), Some(&"world"));
/// ```
pub type GlobalSlotMap<T> = AllocSlotMap<'static, T, GlobalAlloc, Dyn, True>;

impl<'alloc, T, Alloc> AllocSlotMap<'alloc, T, Alloc, Dyn, False> 
    where
        T: Sized,
        Alloc: Allocator,
{

    #[inline(always)]
    pub fn new(alloc: &'alloc Alloc) -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            free_head: None,
            alloc: OptionAlloc::Some(alloc),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: u32, alloc: &'alloc Alloc) -> Result<Self> {
        if capacity == 0 {
            return Ok(Self::new(alloc))
        }
        let data: Pointer<Slot<T>> = unsafe { alloc
            .allocate_uninit(capacity as usize)
            .ok_or(
                if size_of!(T) == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity as usize }
                }
            )?
            .into()
        };
        for i in 0..capacity - 1 {
            unsafe {
                data.add(i as usize).write(Slot::empty(Some(i + 1)));
            }
        }
        unsafe { data.add(capacity as usize - 1).write(Slot::empty(None)) };
        Ok(Self {
            data,
            capacity,
            len: 0,
            free_head: Some(0),
            alloc: OptionAlloc::Some(alloc),
            _marker: PhantomData,
        })
    }
}

impl<'alloc, T, Alloc> AllocSlotMap<'alloc, T, Alloc, Fixed, False> 
    where
        T: Sized,
        Alloc: Allocator,
{

    #[inline(always)]
    pub fn new(capacity: u32, alloc: &'alloc Alloc) -> Result<Self> {
        if capacity == 0 {
            return Err(FixedCapacity { capacity: capacity as usize })
        }
        let data: Pointer<Slot<T>> = unsafe { alloc
            .allocate_uninit(capacity as usize)
            .ok_or(
                if size_of!(T) == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity as usize }
                }
            )?
            .into()
        };
        for i in 0..capacity - 1 {
            unsafe {
                data.add(i as usize).write(Slot::empty(Some(i + 1)));
            }
        }
        Ok(Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            free_head: None,
            alloc: OptionAlloc::Some(alloc),
            _marker: PhantomData,
        })
    }
}


impl<'alloc, T, Alloc, CapacityPol> AllocSlotMap<'alloc, T, Alloc, CapacityPol, False>
    where
        T: Sized,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
{

    #[inline(always)]
    pub fn with_no_alloc() -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            free_head: None,
            alloc: OptionAlloc::None,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, value: T) -> Result<SlotIndex<T>> {
        self.insert_internal(value)
    }
}

impl<T> GlobalSlotMap<T>
    where
        T: Sized,
{

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            free_head: None,
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: u32) -> Result<Self> {
        if capacity == 0 {
            return Ok(Self::new())
        }
        let data: Pointer<Slot<T>> = unsafe { GLOBAL_ALLOC
            .allocate_uninit(capacity as usize)
            .ok_or(
                if size_of!(T) == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: capacity as usize }
                }
            )?
            .into()
        };
        for i in 0..capacity - 1 {
            unsafe {
                data.add(i as usize).write(Slot::empty(Some(i + 1)));
            }
        }
        unsafe { data.add(capacity as usize - 1).write(Slot::empty(None)) };
        Ok(Self {
            data,
            capacity,
            len: 0,
            free_head: Some(0),
            alloc: OptionAlloc::Some(&GLOBAL_ALLOC),
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn insert(&mut self, value: T) -> SlotIndex<T> {
        self.insert_internal(value).unwrap()
    }
}

impl<'alloc, T, Alloc, CapacityPol, IsGlobal> AllocSlotMap<'alloc, T, Alloc, CapacityPol, IsGlobal>
    where
        T: Sized,
        Alloc: Allocator,
        CapacityPol: CapacityPolicy,
        IsGlobal: Conditional
{

    #[inline(always)]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn reserve(&mut self, capacity: u32) -> Result<()> {
        if !CapacityPol::can_grow() {
            return Err(FixedCapacity { capacity: self.capacity as usize })
        }
        let new_capacity = match CapacityPol::grow(self.capacity as usize, capacity as usize) {
            Some(c) => c as u32,
            None => return Err(InvalidReservation {
                current: self.capacity as usize,
                requested: capacity as usize,
            })
        };
        let tmp: Pointer<Slot<T>> = unsafe { self.alloc 
            .allocate_uninit(new_capacity as usize)
            .ok_or(
                if size_of!(T) == 0 {
                    ZeroSizedElement
                }
                else {
                    AllocFailed { new_capacity: new_capacity as usize }
                }
            )?
            .into()
        };
        unsafe {
            self.data.move_elements(tmp, self.capacity as usize);
        };
        for i in self.capacity..new_capacity - 1 {
            unsafe {
                tmp.add(i as usize).write(Slot::empty(Some(i + 1)));
            }
        }
        unsafe {
            tmp.add(new_capacity as usize - 1).write(
                Slot::empty(self.free_head)
            );
        }
        self.free_head = Some(self.capacity);
        if self.capacity != 0 {
            unsafe { self.alloc
                .free_uninit(*self.data, self.capacity as usize);
        }
        }
        self.data = tmp;
        self.capacity = new_capacity;
        Ok(())
    }

    #[inline(always)]
    fn insert_internal(&mut self, value: T) -> Result<SlotIndex<T>>
    {
        if self.free_head.is_none() {
            self.reserve(self.capacity * 2)?;
        }
        let index = self.free_head.unwrap();
        let free_slot = unsafe { self.data.add(index as usize).as_mut() };
        self.free_head = free_slot.next_free_index.unwrap();
        free_slot.value.write(value);
        free_slot.next_free_index = None;
        self.len += 1;
        Ok(SlotIndex {
            version: NonZeroU32::new(free_slot.version).unwrap(),
            index,
            _marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn remove(&mut self, index: SlotIndex<T>) -> T
    {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let ptr = unsafe { self.data.add(index.index as usize) };
        let mut slot = unsafe { ptr.read() };
        let index_version = index.version.get();
        if slot.version != index_version {
            panic!("stale index: slot version {}, index version {}", slot.version, index_version)
        }
        let value = unsafe { slot.value.assume_init() };
        slot.version += 1;
        slot.next_free_index = Some(self.free_head);
        slot.value = MaybeUninit::uninit();
        unsafe {
            ptr.write(slot);
        }
        self.free_head = Some(index.index);
        self.len -= 1;
        value
    }

    #[inline(always)]
    pub fn contains(&self, index: SlotIndex<T>) -> bool {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let index_version = index.version.get();
        let slot = unsafe { self.data.add(index.index as usize).as_ref() };
        slot.version == index_version
    }

    #[inline(always)]
    pub fn get(&self, index: SlotIndex<T>) -> &T {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let index_version = index.version.get();
        let slot = unsafe { self.data.add(index.index as usize).as_ref() };
        if slot.version != index_version {
            panic!("stale index: slot version {}, index version {}", slot.version, index_version)
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        unsafe {
            slot.value.assume_init_ref()
        }
    }

    pub fn try_get(&self, index: SlotIndex<T>) -> Option<&T> {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let index_version = index.version.get();
        let slot = unsafe { self.data.add(index.index as usize).as_ref() };
        if slot.version != index_version {
            return None
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        Some(unsafe {
            slot.value.assume_init_ref()
        })
    }

    #[inline(always)]
    pub fn get_mut(&mut self, index: SlotIndex<T>) -> &mut T {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let index_version = index.version.get();
        let slot = unsafe { self.data.add(index.index as usize).as_mut() };
        if slot.version != index_version {
            panic!("stale index: slot version {}, index version {}", slot.version, index_version)
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        unsafe {
            slot.value.assume_init_mut()
        }
    }

    pub fn try_get_mut(&mut self, index: SlotIndex<T>) -> Option<&mut T> {
        if index.index > self.capacity {
            panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
        }
        let index_version = index.version.get();
        let slot = unsafe { self.data.add(index.index as usize).as_mut() };
        if slot.version != index_version {
            return None
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        Some(unsafe {
            slot.value.assume_init_mut()
        })
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        if needs_drop::<T>() {
            for i in 0..self.capacity {
                unsafe {
                    let mut slot = self.data.add(i as usize).read();
                    if slot.next_free_index.is_none() {
                        slot.value.assume_init_drop();
                    }
                }
            }
        }
        if self.capacity != 0 {
            unsafe {
                self.alloc.free_uninit(*self.data, self.capacity as usize);
            }
        }
        self.data = Pointer::dangling();
        self.free_head = None;
        self.len = 0;
        self.capacity = 0;
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter::new(self.data, self.data.add(self.capacity as usize))
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut::new(self.data, self.data.add(self.capacity as usize))
        }
    }
}

pub struct IterBase<'a, T, IsMut: Conditional> {
    ptr: Pointer<Slot<T>>,
    end: Pointer<Slot<T>>,
    _marker: PhantomData<(&'a T, IsMut)>
}

impl<'a, T, IsMut: Conditional> IterBase<'a, T, IsMut> {

    #[inline(always)]
    unsafe fn new(mut ptr: Pointer<Slot<T>>, end: Pointer<Slot<T>>) ->  Self {
        unsafe {
            while ptr != end {
                if ptr.as_ref().next_free_index.is_none() {
                    break
                }
                ptr = ptr.add(1);
            }
        }
        Self {
            ptr,
            end,
            _marker: PhantomData,
        }
    }
}

pub type Iter<'a, T> = IterBase<'a, T, False>;
pub type IterMut<'a, T> = IterBase<'a, T, True>;

impl<'a, T> Iterator for Iter<'a, T> {

    type Item = &'a T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_ref();
                self.ptr = self.ptr.add(1);
                while self.ptr != self.end {
                    if self.ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    self.ptr = self.ptr.add(1);
                }
                Some(item.value.assume_init_ref())
            }
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {

    type Item = &'a mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_mut();
                self.ptr = self.ptr.add(1);
                while self.ptr != self.end {
                    if self.ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    self.ptr = self.ptr.add(1);
                }
                Some(item.value.assume_init_mut())
            }
        }
    }
}

impl_traits!(
    for AllocSlotMap<'alloc, T: Sized, Alloc: Allocator, CapacityPol: CapacityPolicy, IsGlobal: Conditional>
    Index<SlotIndex<T>> =>

        type Output = T;

        #[inline(always)]
        fn index(&self, index: SlotIndex<T>) -> &Self::Output {
            self.get(index)
        }
    ,
    IndexMut<SlotIndex<T>> =>

        #[inline(always)]
        fn index_mut(&mut self, index: SlotIndex<T>) -> &mut Self::Output {
            self.get_mut(index)
        }
    ,
    IntoIterator for &'map =>

        type Item = &'map T;
        type IntoIter = Iter<'map, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator for mut &'map =>

        type Item = &'map mut T;
        type IntoIter = IterMut<'map, T>;

        #[inline(always)]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
    Drop =>

        #[inline(always)]
        fn drop(&mut self) {
            self.clear();
        }
    ,
);

impl_traits!(
    for GlobalSlotMap<T: Sized>
    Default =>

        fn default() -> Self {
            Self::new()
        }
    ,
);

impl<'alloc, T: Sized, Alloc: Allocator, CapacityPol: CapacityPolicy> Default
    for AllocSlotMap<'alloc, T, Alloc, CapacityPol, False>
{

    fn default() -> Self {
        Self::with_no_alloc()
    }
}
