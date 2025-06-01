use crate::allocator_traits::AllocateExt;

use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

pub trait Vector<T>
    where
        Self: Sized + Index<usize, Output = T> + IndexMut<usize, Output = T>
{

    type Iter<'a>: Iterator<Item = &'a T>
        where T: 'a, Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut T>
        where T: 'a, Self: 'a;

    fn len(&self) -> usize;

    fn size(&self) -> usize;

    fn as_ptr(&self) -> *const MaybeUninit<T>;

    fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T>;

    fn as_slice(&self) -> &[T];

    fn as_mut_slice(&mut self) -> &mut [T];

    fn resize(&mut self, len: usize) -> bool
        where
            T: Default;

    fn push_back(&mut self, value: T) -> Option<&mut T>; 

    fn pop_back(&mut self) -> Option<T>;

    fn back(&self) -> Option<&T>;

    fn back_mut(&mut self) -> Option<&mut T>;

    fn insert(&mut self, value: T, index: usize) -> Option<&mut T>;

    fn remove(&mut self, index: usize) -> Option<T>;

    fn swap_remove(&mut self, index: usize) -> Option<T>;

    fn clear(&mut self);

    fn clone_from<V: Vector<T>>(&mut self, from: &V) -> bool
        where
            T: Clone + Default;

    fn copy_from<V: Vector<T>>(&mut self, from: &V) -> bool
        where
            T: Copy + Default;

    fn contains(&self,
        value: &T
    ) -> bool
        where
            T: Eq;

    fn push_back_if_unique(
        &mut self,
        value: T
    ) -> Option<&mut T>
        where
            T: Eq
    {
        if self.contains(&value) { None }
        else { self.push_back(value) }
    }

    fn iter(&self) -> Self::Iter<'_>;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;
}

pub trait DynamicVector<'mem, T>: Vector<T> {

    fn reserve(&mut self, size: usize) -> bool;
}
