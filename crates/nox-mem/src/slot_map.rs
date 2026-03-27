//! A compact slot map implementation with support for custom allocators.
//!
//! Slot map is a data structure that associates values with *opaque, stable handles* (indices).
//! Unlike `Vec`, removal doesn't shift elements, and re-insertion may reuse free slots.
//!
//! This module provides:
//!
//! [`DynSlotMap`]: a slot map using a local allocator
//! [`FixedSlotMap`]: [`DynSlotMap`] with a fixed-capacity
//! and if the `std` feature is enabled
//! [`SlotMap`]: a slot map using [`StdAlloc`]
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
//! use nox_mem::slot_map::SlotMap;
//!
//! let mut map = SlotMap::new();
//! let key1 = map.insert("hello");
//! let key2 = map.insert("world");
//! assert_eq!(map.get(key1).ok(), Some(&"hello"));
//! assert_eq!(map.remove(key1).ok(), Some("hello"));
//! assert_eq!(map.get(key1).ok(), None);
//! assert_eq!(map.get(key2).ok(), Some(&"world"));
//! ```

use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut, Deref},
    fmt::{self, Formatter},
};

use nox_proc::{Error, Display};

use crate::{
    conditional::{Conditional, False, True},
    alloc::{LocalAlloc, LocalAllocExt, LocalAllocWrap},
    vec::Pointer,
    collections::{TryReserveError, ReservePolicy},
    impl_traits,
    num::{UInteger, NonZeroInteger},
};

#[derive(Debug, Error)]
pub enum IndexError<IndexType: UInteger = u32> {
    #[display("stale slot map index {index}, slot version was {slot_version}")]
    StaleIndex { index: SlotIndex<(), IndexType>, slot_version: IndexType, },
    #[display("index {index} was out of bounds with capacity {capacity}")]
    IndexOutOfBounds { index: SlotIndex<(), IndexType>, capacity: u32, },
}

use IndexError::{StaleIndex, IndexOutOfBounds};

pub struct DynPolicy;

unsafe impl ReservePolicy<u32> for DynPolicy {

    fn can_grow() -> bool {
        true
    }

    fn grow(current: u32, required: usize) -> core::result::Result<u32, TryReserveError<()>> {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 > u32::MAX as usize {
            Err(TryReserveError::max_capacity_exceeded(u32::MAX as usize, power_of_2, ()))
        } else if power_of_2 <= current as usize {
            Ok(current)
        } else { Ok(power_of_2.max(2) as u32) }
    }

    fn grow_infallible(current: u32, required: usize) -> u32 {
        let power_of_2 = required.next_power_of_two().max(2);
        if power_of_2 <= current as usize {
            current
        } else if power_of_2 > u32::MAX as usize {
            panic!("maximum capacity {} reached", u32::MAX)
        } else {
            power_of_2.max(2) as u32
        }
    }
}

pub type FixedPolicy = crate::vec::FixedPolicy32;

struct Slot<T, IndexType: UInteger> {
    value: MaybeUninit<T>,
    version: IndexType::NonZero,
    next_free_index: Option<Option<u32>>,
}

impl<T: Clone, IndexType: UInteger> Clone for Slot<T, IndexType> {

    fn clone(&self) -> Self {
        Self {
            value: if self.next_free_index.is_none() {
                unsafe {
                    MaybeUninit::new(self.value.assume_init_ref().clone())
                }
            } else {
                MaybeUninit::uninit()
            },
            version: self.version,
            next_free_index: self.next_free_index,
        }
    }
}

impl<T, IndexType: UInteger> Slot<T, IndexType> {

    fn empty(next_free_index: Option<u32>) -> Self {
        Self {
            value: MaybeUninit::uninit(),
            version: unsafe {
                IndexType::NonZero::new_unchecked(IndexType::ONE)
            },
            next_free_index: Some(next_free_index),
        }
    }
}

#[must_use]
#[derive(Display)] #[display("(version {version}, index: {index})")]
pub struct SlotIndex<T, IndexType = u32>
    where
        IndexType: UInteger,
{
    version: IndexType::NonZero,
    index: u32,
    _marker: PhantomData<T>,
}

impl<T, IndexType> SlotIndex<T, IndexType>
    where
        IndexType: UInteger
{

    /// Gets the index part of [`SlotIndex`].
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }
    
    /// Gets the version part of [`SlotIndex`].
    #[inline]
    pub fn version(&self) -> IndexType {
        self.version.get()
    }
   
    #[inline]
    fn unit_index(self) -> SlotIndex<(), IndexType> {
        SlotIndex {
            version: self.version,
            index: self.index,
            _marker: PhantomData,
        }
    }
}

unsafe impl<T, IndexType: UInteger> Send for SlotIndex<T, IndexType> {}
unsafe impl<T, IndexType: UInteger> Sync for SlotIndex<T, IndexType> {}

impl<T, IndexType: UInteger> fmt::Debug for SlotIndex<T, IndexType> {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        "SlotIndex { version: ".fmt(f)?;
        self.version.fmt(f)?;
        ", index: ".fmt(f)?;
        self.index.fmt(f)?;
        "}".fmt(f)
    }
}

impl<T, IndexType: UInteger> Default for SlotIndex<T, IndexType>
{

    #[inline]
    fn default() -> Self {
        Self {
            version: unsafe {
                IndexType::NonZero::new_unchecked(IndexType::ONE)
            },
            index: u32::MAX,
            _marker: PhantomData,
        }
    }
}

impl<T, IndexType: UInteger> Clone for SlotIndex<T, IndexType> {

    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, IndexType: UInteger> Copy for SlotIndex<T, IndexType> {}

impl<T, IndexType: UInteger> PartialEq for SlotIndex<T, IndexType> {

    fn eq(&self, other: &Self) -> bool {
        self.version == other.version &&
        self.index == other.index
    }
}

impl<T, IndexType: UInteger> Eq for SlotIndex<T, IndexType> {}

impl<T, IndexType: UInteger> core::hash::Hash for SlotIndex<T, IndexType> {

    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
        self.index.hash(state);
    }
}

pub struct AllocSlotMap<T, Alloc, ReservePol, IsStd, IndexType = u32>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<u32>,
        IsStd: Conditional,
        IndexType: UInteger,
{
    data: Pointer<Slot<T, IndexType>, u32>,
    capacity: u32,
    len: u32,
    free_head: Option<u32>,
    alloc: Alloc,
    _marker: PhantomData<(IsStd, ReservePol)>,
}

/// A dynamic slot map storing values of type `T`, backed by allocator 'Alloc'.
///
/// Provides stable, opaque handles for accessing values. Removal leaves versioned
/// empty slots and insertions reuse free slots.
///
/// See also [`SlotMap`] for a version using [`GlobalAlloc`].
///
/// # Type parameters
///
/// - `T`: value type
/// - `Alloc`: allocator implementing [`LocalAlloc`]
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
/// let allocator = MyLocalAlloc::default();
/// let mut map = DynSlotMap::new(&allocator);
/// let key = map.insert("value").unwrap();
/// map.remove(key);
pub type DynSlotMap<T, Alloc, Wrap> = AllocSlotMap<T, LocalAllocWrap<Alloc, Wrap>, DynPolicy, False>;

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
/// - `Alloc`: allocator implementing [`LocalAlloc`]
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
/// let allocator = MyLocalAlloc::default();
/// let mut map = FixedSlotMap::with_capacity(8, &allocator).unwrap();
/// let key = map.insert("value").unwrap();
/// map.remove(key);
/// ```
pub type FixedSlotMap<T, Alloc, Wrap> = AllocSlotMap<T, LocalAllocWrap<Alloc, Wrap>, FixedPolicy, False>;

impl<T, Alloc, Wrap, ReservePol, IndexType>
    AllocSlotMap<T, LocalAllocWrap<Alloc, Wrap>, ReservePol, False, IndexType>
    where
        Alloc: LocalAlloc,
        Wrap: Deref<Target = Alloc>,
        IndexType: UInteger,
        ReservePol: ReservePolicy<u32>,
{

    pub fn new(alloc: Wrap) -> Self {
        Self {
            data: Pointer::dangling(),
            capacity: 0,
            len: 0,
            free_head: None,
            alloc: LocalAllocWrap::new(alloc),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: u32, alloc: Wrap) -> Result<Self, TryReserveError<()>> {
        if capacity == 0 {
            return Ok(Self::new(alloc))
        }
        let capacity = ReservePol::grow(capacity, capacity as usize)?;
        let data: Pointer<Slot<T, IndexType>, u32> = unsafe { alloc
            .alloc_uninit(capacity as usize)
            .map_err(|err| TryReserveError::alloc_error(err, ()))?
            .into()
        };
        for i in 0..capacity  - 1 {
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
            alloc: LocalAllocWrap::new(alloc),
            _marker: PhantomData,
        })
    }

    #[inline]
    pub fn insert(&mut self, value: T) -> Result<SlotIndex<T, IndexType>, TryReserveError<T>> {
        self.insert_internal(value)
    }
}

impl<T, Alloc, ReservePol, IsStd, IndexType> AllocSlotMap<T, Alloc, ReservePol, IsStd, IndexType>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<u32>,
        IsStd: Conditional,
        IndexType: UInteger,
{

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn reserve(&mut self, capacity: u32) {
        let capacity = ReservePol
            ::grow(self.capacity, capacity as usize)
            .unwrap();
        self.reserve_internal(capacity).unwrap();
    }

    pub fn reserve_exact(&mut self, capacity: u32) {
        self.reserve_internal(capacity).unwrap();
    }

    pub fn try_reserve(&mut self, capacity: u32) -> Result<(), TryReserveError<()>> {
        let capacity = ReservePol
            ::grow(self.capacity, capacity as usize)?;
        self.reserve_internal(capacity)
    }

    pub fn try_reserve_exact(&mut self, capacity: u32) -> Result<(), TryReserveError<()>> {
        self.reserve_internal(capacity)
    }

    fn reserve_internal(&mut self, capacity: u32) -> Result<(), TryReserveError<()>> {
        if capacity == self.capacity { return Ok(()) }
        let tmp: Pointer<Slot<T, IndexType>, u32> = unsafe { self.alloc
            .alloc_uninit(capacity as usize)
            .map_err(|err| TryReserveError::alloc_error(err, ()))?
            .into()
        };
        unsafe {
            self.data.move_elements(tmp, self.capacity);
        };
        for i in self.capacity..capacity - 1 {
            unsafe {
                tmp.add(i as usize).write(Slot::empty(Some(i + 1)));
            }
        }
        unsafe {
            tmp.add(capacity as usize - 1).write(
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
        self.capacity = capacity;
        Ok(())
    }

    fn insert_internal(&mut self, value: T) -> Result<SlotIndex<T, IndexType>, TryReserveError<T>>
    {
        if self.free_head.is_none() {
            let capacity = self.capacity as usize * 2;
            if capacity > u32::MAX as usize {
                return Err(TryReserveError::max_capacity_exceeded(u32::MAX, capacity, value))
            }
            if let Err(err) = self.try_reserve(capacity as u32) {
                return Err(err.with_value(value))
            }
        }
        let index = self.free_head.unwrap();
        let free_slot = unsafe { self.data.add(index as usize).as_mut() };
        self.free_head = free_slot.next_free_index.unwrap();
        free_slot.value.write(value);
        free_slot.next_free_index = None;
        self.len += 1;
        Ok(SlotIndex {
            version: free_slot.version,
            index,
            _marker: PhantomData,
        })
    }

    pub fn remove(&mut self, index: SlotIndex<T, IndexType>) -> Result<T, IndexError<IndexType>>
    {
        if index.index >= self.capacity {
            return Err(
                IndexOutOfBounds {
                    index: index.unit_index(),
                    capacity: self.capacity,
                }
            )
        }
        let ptr = unsafe { self.data.add(index.index as usize) };
        let mut slot = unsafe { ptr.read() };
        let index_version = index.version;
        if slot.version != index_version {
            return Err(StaleIndex {
                index: index.unit_index(),
                slot_version: slot.version.get(),
            })
        }
        let value = unsafe { slot.value.assume_init() };
        slot.version = IndexType::NonZero
            ::new(slot.version.get().wrapping_add(IndexType::ONE))
            .unwrap_or_else(|| unsafe { IndexType::NonZero::new_unchecked(IndexType::ONE) });
        slot.next_free_index = Some(self.free_head);
        slot.value = MaybeUninit::uninit();
        unsafe {
            ptr.write(slot);
        }
        self.free_head = Some(index.index);
        self.len -= 1;
        Ok(value)
    }

    pub fn contains(&self, index: SlotIndex<T, IndexType>) -> bool {
        if index.index >= self.capacity {
            return false
        }
        let index_version = index.version;
        let slot = unsafe { self.data.add(index.index as usize).as_ref() };
        slot.version == index_version
    }

    pub fn get(&self, index: SlotIndex<T, IndexType>) -> Result<&T, IndexError<IndexType>> {
        if index.index >= self.capacity {
            return Err(IndexOutOfBounds {
                index: index.unit_index(),
                capacity: self.capacity,
            })
        }
        let index_version = index.version;
        let slot = unsafe { self.data.add(index.index as usize).as_ref() };
        if slot.version != index_version {
            return Err(StaleIndex {
                index: index.unit_index(),
                slot_version: slot.version.get(),
            })
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        unsafe {
            Ok(slot.value.assume_init_ref())
        }
    }

    pub fn get_mut(&mut self, index: SlotIndex<T, IndexType>) -> Result<&mut T, IndexError<IndexType>> {
        if index.index >= self.capacity {
            return Err(IndexOutOfBounds {
                index: index.unit_index(),
                capacity: self.capacity,
            })
        }
        let index_version = index.version;
        let slot = unsafe { self.data.add(index.index as usize).as_mut() };
        if slot.version != index_version {
            return Err(StaleIndex {
                index: index.unit_index(),
                slot_version: slot.version.get(),
            })
        }
        assert!(slot.next_free_index.is_none(), "invalid index");
        unsafe {
            Ok(slot.value.assume_init_mut())
        }
    }
    
    /// Gets a reference to value at `index` bypassing all validity checks including bounds checks.
    ///
    /// # Safety
    /// The index *must* be a valid index, otherwise the value might be uninitialized or out of
    /// bounds.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: SlotIndex<T, IndexType>) -> &T {
        unsafe {
            self.data.add(index.index as usize).as_ref()
            .value.assume_init_ref()
        }
    }

    /// Gets a mutable reference to value at `index` bypassing all validity checks including bounds
    /// checks.
    ///
    /// # Safety
    /// The index *must* be a valid index, otherwise the value might be uninitialized or out of
    /// bounds.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: SlotIndex<T, IndexType>) -> &mut T {
        unsafe {
            self.data.add(index.index as usize).as_mut()
            .value.assume_init_mut()
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.capacity() {
            unsafe {
                let slot = self.data.add(i as usize).read();
                if slot.next_free_index.is_none() {
                    self.remove(SlotIndex {
                        version: slot.version,
                        index: i,
                        _marker: PhantomData,
                    }).unwrap();
                }
            }
        }
        self.len = 0;
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T, IndexType> {
        unsafe {
            Iter::new(self.data, self.data.add(self.capacity as usize))
        }
    }

    #[inline]
    pub fn keys(&self) -> IterKeys<'_, T, IndexType> {
        unsafe {
            IterKeys::new(self.data, self.data.add(self.capacity() as usize))
        }
    }

    #[inline]
    pub fn values(&self) -> IterValues<'_, T, IndexType> {
        unsafe {
            IterValues::new(self.data, self.data.add(self.capacity() as usize))
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T, IndexType> {
        unsafe {
            IterMut::new(self.data, self.data.add(self.capacity as usize))
        }
    }

    #[inline]
    pub fn values_mut(&self) -> IterValuesMut<'_, T, IndexType> {
        unsafe {
            IterValuesMut::new(self.data, self.data.add(self.capacity() as usize))
        }
    }
}

mod iter {

    use super::*;

    pub struct KeyValue;
    pub struct Keys;
    pub struct Values;

    pub struct Base<'a, T, IndexType, IsMut, Type>
        where
            IndexType: UInteger,
            IsMut: Conditional,
    {
        pub(super) ptr: Pointer<Slot<T, IndexType>, u32>,
        pub(super)end: Pointer<Slot<T, IndexType>, u32>,
        pub(super) index: u32,
        pub(super) _marker: PhantomData<(&'a T, IsMut, Type)>
    }

    impl<'a, T, IndexType, IsMut, Type> Base<'a, T, IndexType, IsMut, Type>
        where
            IndexType: UInteger, 
            IsMut: Conditional,
    {

        pub(super) unsafe fn new(
            mut ptr: Pointer<Slot<T, IndexType>, u32>,
            end: Pointer<Slot<T, IndexType>, u32>
        ) ->  Self
        {
            let mut index = 0;
            unsafe {
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
            }
            Self {
                ptr,
                end,
                index,
                _marker: PhantomData,
            }
        }
    }
}

pub type Iter<'a, T, IndexType> = iter::Base<'a, T, IndexType, False, iter::KeyValue>;
pub type IterKeys<'a, T, IndexType> = iter::Base<'a, T, IndexType, False, iter::Keys>;
pub type IterValues<'a, T, IndexType> = iter::Base<'a, T, IndexType, False, iter::Values>;

pub type IterMut<'a, T, IndexType> = iter::Base<'a, T, IndexType, True, iter::KeyValue>;
pub type IterValuesMut<'a, T, IndexType> = iter::Base<'a, T, IndexType, True, iter::Values>;

impl<'a, T, IndexType> Iterator for Iter<'a, T, IndexType>
    where IndexType: UInteger,
{

    type Item = (SlotIndex<T, IndexType>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_ref();
                let current_idx = self.index;
                let mut ptr = self.ptr.add(1);
                let end = self.end;
                let mut index = self.index + 1;
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
                self.ptr = ptr;
                self.index = index;
                Some((
                    SlotIndex {
                        version: item.version,
                        index: current_idx,
                        _marker: PhantomData,
                    },
                    item.value.assume_init_ref(),
                ))
            }
        }
    }
}

impl<'a, T, IndexType> Iterator for IterKeys<'a, T, IndexType>
    where IndexType: UInteger
{

    type Item = SlotIndex<T, IndexType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_ref();
                let current_idx = self.index;
                let mut ptr = self.ptr.add(1);
                let end = self.end;
                let mut index = self.index + 1;
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
                self.ptr = ptr;
                self.index = index;
                Some(
                    SlotIndex {
                        version: item.version,
                        index: current_idx,
                        _marker: PhantomData,
                    },
                )
            }
        }
    }
}

impl<'a, T, IndexType> Iterator for IterValues<'a, T, IndexType>
    where IndexType: UInteger
{

    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_ref();
                let mut ptr = self.ptr.add(1);
                let end = self.end;
                let mut index = self.index + 1;
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
                self.ptr = ptr;
                self.index = index;
                Some(
                    item.value.assume_init_ref(),
                )
            }
        }
    }
}

impl<'a, T, IndexType> Iterator for IterMut<'a, T, IndexType>
    where IndexType: UInteger
{

    type Item = (SlotIndex<T, IndexType>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_mut();
                let current_idx = self.index;
                let mut ptr = self.ptr.add(1);
                let end = self.end;
                let mut index = self.index + 1;
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
                self.ptr = ptr;
                self.index = index;
                Some((
                    SlotIndex {
                        version: item.version,
                        index: current_idx,
                        _marker: PhantomData,
                    },
                    item.value.assume_init_mut(),
                ))
            }
        }
    }
}

impl<'a, T, IndexType> Iterator for IterValuesMut<'a, T, IndexType>
    where IndexType: UInteger
{

    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        }
        else {
            unsafe {
                let item = self.ptr.as_mut();
                let mut ptr = self.ptr.add(1);
                let end = self.end;
                let mut index = self.index + 1;
                while ptr != end {
                    if ptr.as_ref().next_free_index.is_none() {
                        break
                    }
                    ptr = ptr.add(1);
                    index += 1;
                }
                self.ptr = ptr;
                self.index = index;
                Some(
                    item.value.assume_init_mut(),
                )
            }
        }
    }
}

pub struct IterMove<T, Alloc, ReservePol, IsStd, IndexType>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<u32>,
        IsStd: Conditional,
        IndexType: UInteger,
{
    slot_map: AllocSlotMap<T, Alloc, ReservePol, IsStd, IndexType>,
    off: u32,
}

impl<T, Alloc, ReservePol, IsStd, IndexType> Iterator for IterMove<T, Alloc, ReservePol, IsStd, IndexType>
    where
        Alloc: LocalAlloc,
        ReservePol: ReservePolicy<u32>,
        IsStd: Conditional,
        IndexType: UInteger,
{

    type Item = (SlotIndex<T, IndexType>, T);

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = unsafe { IterMut::new(
            self.slot_map.data.add(self.off as usize),
            self.slot_map.data.add(self.slot_map.capacity as usize)
        )};
        iter.index += self.off;
        self.off = iter.index;
        let index = iter.next()?.0;
        Some((index, self.slot_map.remove(index).unwrap()))
    }
}

impl_traits!(
    for AllocSlotMap<T, Alloc: [LocalAlloc], ReservePol: [ReservePolicy<u32>], IsStd: [Conditional], IndexType: [UInteger]>
    Index<SlotIndex<T, IndexType>> =>

        type Output = T;

        fn index(&self, index: SlotIndex<T, IndexType>) -> &Self::Output {
            if index.index >= self.capacity {
                panic!("index {} out of bounds with capacity {}",
                    index.index, self.capacity
                )
            }
            let index_version = index.version;
            let slot = unsafe { self.data.add(index.index as usize).as_ref() };
            if slot.version != index_version {
                panic!("stale index: slot version {}, index version {}",
                    slot.version, index_version,
                );
            }
            assert!(slot.next_free_index.is_none(), "invalid index");
            unsafe {
                slot.value.assume_init_ref()
            }
        }
    ,
    IndexMut<SlotIndex<T, IndexType>> =>

        fn index_mut(&mut self, index: SlotIndex<T, IndexType>) -> &mut Self::Output {
            if index.index >= self.capacity {
                panic!("index {} out of bounds with capacity {}", index.index, self.capacity)
            }
            let index_version = index.version;
            let slot = unsafe { self.data.add(index.index as usize).as_mut() };
            if slot.version != index_version {
                panic!("stale index: slot version {}, index version {}", 
                    slot.version, index_version,
                );
            }
            assert!(slot.next_free_index.is_none(), "invalid index");
            unsafe {
                slot.value.assume_init_mut()
            }
        }
    ,
    IntoIterator for &'map =>

        type Item = (SlotIndex<T, IndexType>, &'map T);
        type IntoIter = Iter<'map, T, IndexType>;

        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    ,
    IntoIterator for mut &'map =>

        type Item = (SlotIndex<T, IndexType>, &'map mut T);
        type IntoIter = IterMut<'map, T, IndexType>;

        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    ,
    IntoIterator =>

        type Item = (SlotIndex<T, IndexType>, T);
        type IntoIter = IterMove<T, Alloc, ReservePol, IsStd, IndexType>;

        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            IterMove {
                slot_map: self,
                off: 0,
            }
        }
    ,
    Drop =>

        fn drop(&mut self) {
            self.clear();
            if self.capacity() != 0 {
                unsafe {
                    self.alloc.free_uninit(*self.data, self.len() as usize);
                }
            }
        }
    ,
);

unsafe impl<
    T: Send,
    Alloc: LocalAlloc + Send,
    ReservePol: ReservePolicy<u32>,
    IsStd: Conditional,
> Send for AllocSlotMap<T, Alloc, ReservePol, IsStd> {}

unsafe impl<
    T: Sync,
    Alloc: LocalAlloc + Sync,
    ReservePol: ReservePolicy<u32>,
    IsStd: Conditional,
> Sync for AllocSlotMap<T, Alloc, ReservePol, IsStd> {}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    use crate::alloc::StdAlloc;

    /// A dynamic slot map storing values of type `T`, backed by [`GlobalAlloc`].
    ///
    /// # Type parameters
    ///
    /// - `T`: value type
    ///
    /// # Example
    ///
    /// ```rust
    /// use nox_mem::slot_map::SlotMap;
    ///
    /// let mut map = SlotMap::new();
    /// let key1 = map.insert("hello");
    /// let key2 = map.insert("world");
    /// assert_eq!(map.get(key1).ok(), Some(&"hello"));
    /// assert_eq!(map.remove(key1).ok(), Some("hello"));
    /// assert_eq!(map.get(key1).ok(), None);
    /// assert_eq!(map.get(key2).ok(), Some(&"world"));
    /// ```
    pub type SlotMap<T, IndexType = u32> = AllocSlotMap<T, StdAlloc, DynPolicy, True, IndexType>;

    impl<T, IndexType> SlotMap<T, IndexType>
        where IndexType: UInteger
    {

        pub fn new() -> Self {
            Self {
                data: Pointer::dangling(),
                capacity: 0,
                len: 0,
                free_head: None,
                alloc: StdAlloc,
                _marker: PhantomData,
            }
        }

        pub fn with_capacity(capacity: u32) -> Self {
            if capacity == 0 {
                return Self::new()
            }
            let capacity = DynPolicy::grow(0, capacity as usize).unwrap();
            let data: Pointer<Slot<T, IndexType>, u32> = unsafe { StdAlloc
                .alloc_uninit(capacity as usize)
                .expect("global alloc failed")
                .into()
            };
            for i in 0..capacity - 1 {
                unsafe {
                    data.add(i as usize).write(Slot::empty(Some(i + 1)));
                }
            }
            unsafe { data.add(capacity as usize - 1).write(Slot::empty(None)) };
            Self {
                data,
                capacity,
                len: 0,
                free_head: Some(0),
                alloc: StdAlloc,
                _marker: PhantomData,
            }
        }

        pub fn insert(&mut self, value: T) -> SlotIndex<T, IndexType> {
            self.insert_internal(value).unwrap()
        }
    }

    impl<T, IndexType> Clone for SlotMap<T, IndexType>
        where
            T: Clone,
            IndexType: UInteger,
    {

        fn clone(&self) -> Self {
            let data: Pointer<Slot<T, IndexType>, u32> = unsafe { StdAlloc
                .alloc_uninit(self.capacity as usize)
                .expect("global alloc failed")
                .into()
            };
            for i in 0..self.capacity as usize {
                unsafe {
                    data.add(i).write(
                        self.data.add(i).as_ref().clone()
                    );
                }
            }
            Self {
                data,
                capacity: self.capacity,
                len: self.len,
                free_head: self.free_head,
                alloc: StdAlloc,
                _marker: PhantomData,
            }
        }
    }

    impl<T, IndexType: UInteger> Default for SlotMap<T, IndexType> {

        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "std")]
pub use std_features::*;
