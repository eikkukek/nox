use core::{
    ops::{Deref, DerefMut},
    hash::{Hash, Hasher},
    result,
};

use super::{VecError, Result};

pub trait Vector<T>:
    Sized +
    AsRef<[T]> +
    Deref<Target = [T]> +
    DerefMut
{

    type Iter<'a>: Iterator<Item = &'a T>
        where T: 'a, Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut T>
        where T: 'a, Self: 'a;

    fn len(&self) -> usize;

    fn capacity(&self) -> usize;

    fn as_ptr(&self) -> *const T;

    fn as_mut_ptr(&mut self) -> *mut T;

    fn as_slice(&self) -> &[T];

    fn as_mut_slice(&mut self) -> &mut [T];

    unsafe fn set_len(&mut self, new_len: usize);

    fn reserve(&mut self, size: usize) -> Result<()>;

    fn resize(&mut self, new_len: usize, value: T) -> Result<()>
        where
            T: Clone;

    fn resize_with<F>(&mut self, new_len: usize, f: F) -> Result<()>
        where
            F: FnMut() -> T;

    fn try_resize_with<F, E, MapE>(
        &mut self,
        new_len: usize,
        f: F,
        map_err: MapE,
    ) -> result::Result<(), E>
        where
            F: FnMut() -> result::Result<T, E>,
            MapE: FnMut(VecError) -> E;

    fn push(&mut self, value: T) -> Result<&mut T>;

    fn extend(&mut self, iter: impl Iterator<Item = T>) -> Result<&mut Self> {
        for item in iter {
            self.push(item)?;
        }
        Ok(self)
    }

    fn append(&mut self, slice: &[T]) -> Result<()>
        where
            T: Clone;

    fn append_map<U, F>(&mut self, slice: &[U], f: F) -> Result<()>
        where
            F: FnMut(&U) -> T;

    fn pop(&mut self) -> Option<T>;

    fn last(&self) -> Option<&T>;

    fn last_mut(&mut self) -> Option<&mut T>;

    fn insert(&mut self, index: usize, value: T) -> Result<&mut T>;

    fn remove(&mut self, index: usize) -> T;

    fn retain(&mut self, mut p: impl FnMut(&T) -> bool) {
        for i in (0..self.len()).rev() {
            if !p(&self[i]) {
                self.remove(i);
            }
        }
    }

    fn swap_remove(&mut self, index: usize) -> T;

    fn clear(&mut self);

    fn clone_from_slice(&mut self, from: &[T]) -> Result<()>
        where
            T: Clone;

    fn move_from_vec<V>(&mut self, from: &mut V) -> Result<()>
        where
            V: Vector<T>;

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> Self::Iter<'_>;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn map_hash<H, F>(&self, state: &mut H, mut f: F)
        where 
            H: Hasher,
            F: FnMut(&T, &mut H)
    {
        self.len().hash(state);
        for value in self.iter() {
            f(value, state);
        }
    }

    fn map_eq<F>(&self, other: &Self, mut f: F) -> bool
        where
            F: FnMut(&T, &T) -> bool
    {
        if self.len() != other.len() {
            return false
        }
        for (i, value) in self.iter().enumerate() {
            if !f(value, &other[i]) {
                return false
            }
        }
        true
    }
}
