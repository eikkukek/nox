use core::{
    ops::{Index, IndexMut},
    mem::MaybeUninit,
};

use super::{MemoryStrategy, DuplicateStrategy, CapacityPolicy, CapacityError};

pub trait Vector<T>
    where
        T: MemoryStrategy,
        Self:
            Sized +
            Index<usize, Output = T> +
            IndexMut<usize, Output = T>
{

    type CapacityPol: CapacityPolicy;

    type Iter<'a>: Iterator<Item = &'a T>
        where T: 'a, Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut T>
        where T: 'a, Self: 'a;

    fn len(&self) -> usize;

    fn capacity(&self) -> usize;

    fn as_ptr(&self) -> *const MaybeUninit<T>;

    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T>;

    fn as_slice(&self) -> &[T];

    fn as_mut_slice(&mut self) -> &mut [T];

    fn reserve(&mut self, size: usize) -> Result<(), CapacityError>;

    fn resize(&mut self, len: usize, value: T) -> Result<(), CapacityError>
        where
            T: Clone;

    fn resize_with<F>(&mut self, len: usize, f: F) -> Result<(), CapacityError>
        where
            F: FnMut() -> T;

    fn push(&mut self, value: T) -> Result<&mut T, CapacityError>;

    fn pop(&mut self) -> Option<T>;

    fn back(&self) -> Option<&T>;

    fn back_mut(&mut self) -> Option<&mut T>;

    fn insert(&mut self, value: T, index: usize) -> Result<&mut T, CapacityError>;

    fn remove(&mut self, index: usize) -> Option<T>;

    fn swap_remove(&mut self, index: usize) -> Option<T>;

    fn clear(&mut self);

    fn duplicate_from<V>(&mut self, from: &V) -> Result<(), CapacityError>
        where
            V: Vector<T>,
            T: DuplicateStrategy;

    fn contains(&self, value: &T) -> bool
        where
            T: PartialEq;

    fn push_if_unique(&mut self,value: T) -> Result<Option<&mut T>, CapacityError>
        where
            T: PartialEq
    {
        if self.contains(&value) {
            Ok(None)
        }
        else {
            let val = self.push(value)?;
            Ok(Some(
                val
            ))
        }
    }

    fn iter(&self) -> Self::Iter<'_>;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}
