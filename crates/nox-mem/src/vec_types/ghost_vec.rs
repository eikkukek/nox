use core::{
    slice,
    ptr::{dangling, dangling_mut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    vec_types::*,
    *
};

pub struct GhostVec<T> {
    _marker: PhantomData<T>
}

impl<T> Default for GhostVec<T> {

    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Vector<T> for GhostVec<T> {

    type Iter<'a> = slice::Iter<'a, T>
            where T: 'a, Self: 'a;
    
    type IterMut<'a> = slice::IterMut<'a, T>
            where T: 'a, Self: 'a;

    fn len(&self) -> usize {
        0
    }
    
    fn capacity(&self) -> usize {
        0
    }

    fn as_ptr(&self) -> *const T {
        dangling()
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        dangling_mut()
    }

    fn as_slice(&self) -> &[T] {
        &[]
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        &mut []
    }

    unsafe fn set_len(&mut self, _new_len: usize) {}

    fn reserve(&mut self, _size: usize) -> Result<(), CapacityError> {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn resize(&mut self, _new_len: usize, _value: T) -> Result<(), CapacityError>
        where
            T: Clone { Err(CapacityError::FixedCapacity { capacity: 0 }) }

    fn resize_with<F>(&mut self, _new_len: usize, _f: F) -> Result<(), CapacityError>
        where
            F: FnMut() -> T { Err(CapacityError::FixedCapacity { capacity: 0 }) }

    fn push(&mut self, _value: T) -> Result<&mut T, CapacityError> {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn append(&mut self, _slice: &[T]) -> Result<(), CapacityError>
        where
        T: Clone
    {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn append_map<U, F>(&mut self, _slice: &[U], _f: F) -> Result<(), CapacityError>
        where
            F: FnMut(&U) -> T
    {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn pop(&mut self) -> Option<T> {
        None
    }

    fn last(&self) -> Option<&T> {
        None
    }

    fn last_mut(&mut self) -> Option<&mut T> {
        None
    }

    fn insert(&mut self, _index: usize, _value: T) -> Result<&mut T, CapacityError> {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn remove(&mut self, _index: usize) -> T {
        panic!("called remove on GhostVec")
    }

    fn swap_remove(&mut self, _index: usize) -> T {
        panic!("called swap_remove on GhostVec")
    }

    fn clear(&mut self) {}

    fn clone_from_slice(&mut self, _from: &[T]) -> Result<(), CapacityError>
        where
            T: Clone
    {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }

    fn move_from_vec<V>(&mut self, _from: &mut V) -> Result<(), CapacityError>
        where
            V: Vector<T>
    {
        Err(CapacityError::FixedCapacity { capacity: 0 })
    }
    
    fn iter(&self) -> Self::Iter<'_> {
        Default::default()
    }
    
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        Default::default()
    }
}

impl<T> AsRef<[T]> for GhostVec<T> {
    
    fn as_ref(&self) -> &[T] {
        &[]
    }
}

impl<T> Deref for GhostVec<T> {

    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &[]
    }
}

impl<T> DerefMut for GhostVec<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut []
    }
}
