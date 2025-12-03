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

use super::VecError;

pub struct PhantomVec<T> {
    _marker: PhantomData<T>
}

impl<T> Default for PhantomVec<T> {

    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Vector<T> for PhantomVec<T> {

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

    fn reserve(&mut self, _size: usize) -> Result<(), VecError> {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }

    fn resize(&mut self, _new_len: usize, _value: T) -> Result<(), VecError>
        where
            T: Clone { Err(CapacityError::FixedCapacity { capacity: 0 }.into()) }

    fn resize_with<F>(&mut self, _new_len: usize, _f: F) -> Result<(), VecError>
        where
            F: FnMut() -> T { Err(CapacityError::FixedCapacity { capacity: 0 }.into()) }

    fn push(&mut self, _value: T) -> Result<&mut T, VecError> {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }

    fn append(&mut self, _slice: &[T]) -> Result<(), VecError>
        where
        T: Clone
    {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }

    fn append_map<U, F>(&mut self, _slice: &[U], _f: F) -> Result<(), VecError>
        where
            F: FnMut(&U) -> T
    {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
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

    fn insert(&mut self, _index: usize, _value: T) -> Result<&mut T, VecError> {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }

    fn remove(&mut self, _index: usize) -> T {
        panic!("called remove on PhantomVec")
    }

    fn swap_remove(&mut self, _index: usize) -> T {
        panic!("called swap_remove on PhantomVec")
    }

    fn clear(&mut self) {}

    fn clone_from_slice(&mut self, _from: &[T]) -> Result<(), VecError>
        where
            T: Clone
    {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }

    fn move_from_vec<V>(&mut self, _from: &mut V) -> Result<(), VecError>
        where
            V: Vector<T>
    {
        Err(CapacityError::FixedCapacity { capacity: 0 }.into())
    }
    
    fn iter(&self) -> Self::Iter<'_> {
        Default::default()
    }
    
    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        Default::default()
    }
}

impl<T> AsRef<[T]> for PhantomVec<T> {
    
    fn as_ref(&self) -> &[T] {
        &[]
    }
}

impl<T> Deref for PhantomVec<T> {

    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &[]
    }
}

impl<T> DerefMut for PhantomVec<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut []
    }
}
