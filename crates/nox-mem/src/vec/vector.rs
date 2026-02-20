use core::{
    ops::{Deref, DerefMut},
    borrow::{Borrow, BorrowMut},
};

use crate::{
    collections::{ReservePolicy, TryReserveError},
    num::{IntoUsize, FromUsize},
};

pub trait Vector<T, SizeType = usize>:
    Sized +
    AsRef<[T]> +
    Deref<Target = [T]> +
    DerefMut + 
    Borrow<[T]> +
    BorrowMut<[T]>
    where 
        SizeType: IntoUsize + FromUsize,
{

    type Iter<'a>: Iterator<Item = &'a T>
        where T: 'a, Self: 'a;

    type IterMut<'a>: Iterator<Item = &'a mut T>
        where T: 'a, Self: 'a;

    type ReservePol: ReservePolicy<SizeType>;
    
    /// Gets the length of the vector.
    fn len(&self) -> SizeType;

    /// Returns whether the vector is empty, i.e. whether the length is equal to zero.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == SizeType::ZERO
    }

    /// Gets the allocated capacity of the vector.
    fn capacity(&self) -> SizeType;

    /// Gets the vector's internal pointer.
    fn as_ptr(&self) -> *const T;

    /// Gets the vector's internal pointer as mutable.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Returns a slice over the vector.
    fn as_slice(&self) -> &[T];

    /// Returns a mutable slice over the vector.
    fn as_mut_slice(&mut self) -> &mut [T];

    /// Sets the length of the vector to `len`. Panics if `len` exceeds `capacity`.
    /// # Safety
    /// If `len` > current length, elements with with indices greater or equal to `len` are
    /// left uninitialized and if `len` < current length, indices greater or equal to `len`
    /// are not dropped.
    unsafe fn set_len(&mut self, len: SizeType);

    /// Reserves space for the vector.
    ///
    /// This may speculatively reserve more space than `capacity` to avoid frequent reallocations.
    /// Use [`Vector::reserve_exact`] to allocate exact size.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn reserve(&mut self, capacity: SizeType);

    /// Reserves space for the vector exactly up to `capacity`.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn reserve_exact(&mut self, capacity: SizeType);

    /// Resizes the vector with a clonable value.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn resize(&mut self, len: SizeType, value: T)
        where
            T: Clone;

    /// Resizes the vector with a a closure.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn resize_with<F>(&mut self, len: SizeType, f: F)
        where
            F: FnMut() -> T;

    /// Tries to resize the vector with a closure that may fail.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn try_resize_with<F, E>(
        &mut self,
        len: SizeType,
        f: F,
    ) -> Result<(), E>
        where 
            F: FnMut() -> Result<T, E>;

    /// Pushes an element to the vector.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn push(&mut self, value: T);

    /// Extends the vector with an iterator.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn extend<I>(&mut self, iter: I) -> &mut Self 
        where
            I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        if let (start, Some(end)) = iter.size_hint() {
            self.reserve_exact(
                Self::ReservePol::grow_infallible(self.capacity(),
                end.saturating_sub(start),
            ));
        }
        for item in iter {
            self.push(item);
        }
        self
    }

    /// Tries to extend the vector with an iterator over [`Result`].
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn try_extend<I, E>(
        &mut self,
        iter: I,
    ) -> Result<&mut Self, E>
        where 
            I: IntoIterator<Item = Result<T, E>>
    {
        let iter = iter.into_iter();
        if let (start, Some(end)) = iter.size_hint() {
            self.reserve_exact(
                Self::ReservePol::grow_infallible(self.capacity(),
                end.saturating_sub(start),
            ));
        }
        for item in iter {
            self.push(item?);
        }
        Ok(self)
    }

    /// Appends the contents of a slice over [`T`] to the vector.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn append(&mut self, slice: &[T])
        where
            T: Clone;

    /// Maps and appends the contents of a slice over [`U`] to the vector.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn append_map<U, F>(&mut self, slice: &[U], f: F)
        where
            F: FnMut(&U) -> T;

    /// Removes and returns an element from the vector at the given `index`.
    ///
    /// Panics if the index is out of bounds.
    fn remove(&mut self, index: SizeType) -> T;

    /// Removes and returns the last element of the vector.
    ///
    /// Returns [`None`] if the vector is empty.
    fn pop(&mut self) -> Option<T>;

    /// Gets a reference of the last element, if any, from the vector.
    ///
    /// Returns [`None`] if the vector is empty.
    fn last(&self) -> Option<&T>;

    /// Gets a mutable reference of the last element from the vector.
    ///
    /// Returns [`None`] if the vector is empty.
    fn last_mut(&mut self) -> Option<&mut T>;

    /// Inserts an element to the vector at the given `index`.
    ///
    /// Panics if the index is out of bounds.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn insert(&mut self, index: SizeType, value: T);

    /// Removes and returns an element from the vector at the given `index`.
    ///
    /// The element at `index` is replaced by the last element of the vector.
    fn swap_remove(&mut self, index: SizeType) -> T;

    /// Retains only the elements specified by the given predicate.
    ///
    /// This preserves the order of the elements.
    fn retain<F>(&mut self, mut p: F)
        where 
            F: FnMut(&T) -> bool,
    {
        for i in SizeType::ZERO.iter(self.len()).rev() {
            if !p(&self[i.into_usize()]) {
                self.remove(i);
            }
        }
    }

    /// Retains only the elements specified by the given predicate.
    ///
    /// This may not preserve the order of elements.
    fn retain_unordered<F>(&mut self, mut p: F)
        where 
            F: FnMut(&T) -> bool,
    {
        for i in SizeType::ZERO.iter(self.len()).rev() {
            if !p(&self[i.into_usize()]) {
                self.swap_remove(i);
            }
        }
    }

    /// Removes consecutive repeated elements from the vector.
    fn dedup(&mut self)
        where T: PartialEq
    {
        for i in (0..self.len().into_usize().saturating_sub(1)).rev() {
            if self[i] == self[i + 1] {
                self.remove(SizeType::from_usize_unchecked(i + 1));
            }
        }
    }

    fn dedup_by<F>(&mut self, mut p: F)
        where F: FnMut(&T, &T) -> bool
    {
        for i in (0..self.len().into_usize().saturating_sub(1)).rev() {
            if p(&self[i], &self[i + 1]) {
                self.remove(SizeType::from_usize_unchecked(i + 1));
            }
        }
    }

    /// Removes all elements from the vector preserving its capacity.
    fn clear(&mut self);

    /// Moves the contents of another vector to this vector and sets the length of the other vector
    /// to zero.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn move_from_vec<V, S>(&mut self, from: &mut V)
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize;

    /// Maps and moves the contents of another vector to this vector and sets the length of the
    /// other vector to zero.
    ///
    /// May panic if the vector has a fixed capacity or if an allocation fails. See the
    /// [`FallibleVec`] trait for vectors that may fail to reserve more capacity.
    fn move_from_vec_map<U, V, S, F>(&mut self, from: &mut V, f: F)
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T;

    /// Returns an iterator over the vector.
    fn iter(&self) -> Self::Iter<'_>;

    /// Returns a mutable iterator over the vector.
    fn iter_mut(&mut self) -> Self::IterMut<'_>; 
}

pub trait FallibleVec<T, SizeType = usize>: Vector<T, SizeType>
    where
        SizeType: IntoUsize + FromUsize,
{

    /// Tries to reserve space for the vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_reserve(&mut self, capacity: SizeType) -> Result<(), TryReserveError<()>>;

    /// Reserves space for the vector exactly up to `capacity`.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_reserve_exact(&mut self, capacity: SizeType) -> Result<(), TryReserveError<()>>;

    /// Tries to resize the vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_resize(&mut self, len: SizeType, value: T) -> Result<(), TryReserveError<()>>
        where
            T: Clone;

    /// Tries to resize the vector with a closure.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_resize_with<F>(&mut self, len: SizeType, f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut() -> T;
    
    /// Tries to resize the vector with a closure that may fail.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_try_resize_with<F, E, MapE>(
        &mut self,
        len: SizeType,
        f: F,
        map_reserve_err: MapE,
    ) -> Result<(), E>
        where
            F: FnMut() -> Result<T, E>,
            MapE: FnMut(TryReserveError<()>) -> E;

    /// Tries to push an element to the vector
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_push(&mut self, value: T) -> Result<(), TryReserveError<T>>;

    /// Tries to extend the vector with an iterator.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    #[allow(clippy::type_complexity)]
    fn fallible_extend<I>(
        &mut self,
        iter: I,
    ) -> Result<&mut Self, TryReserveError<(Option<T>, I::IntoIter)>>
        where
            I: IntoIterator<Item = T>
    {
        let mut iter = iter.into_iter();
        if let (start, Some(end)) = iter.size_hint() {
            let capacity = match Self::ReservePol::grow(self.capacity(), end.saturating_sub(start)) {
                Ok(c) => c,
                Err(err) => return Err(err.with_value((None, iter)))
            };
            if let Err(err) = self.fallible_reserve_exact(capacity) {
                return Err(err.with_value((None, iter)))
            }
        }
        while let Some(item) = iter.next() {
            if let Err(err) = self.fallible_push(item) {
                let (value, err) = err.recover_value();
                return Err(TryReserveError::new(err, (Some(value), iter)))
            }
        }
        Ok(self)
    }

    /// Tries to extend the vector with an iterator over [`Result`].
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_try_extend<I, E, F>(
        &mut self,
        iter: I,
        mut map_reserve_err: F,
    ) -> Result<&mut Self, E>
        where
            I: IntoIterator<Item = Result<T, E>>,
            F: FnMut(TryReserveError<(Option<T>, I::IntoIter)>) -> E,
    {
        let mut iter = iter.into_iter();
        if let (start, Some(end)) = iter.size_hint() {
            let capacity = match Self::ReservePol::grow(self.capacity(), end.saturating_sub(start)) {
                Ok(c) => c,
                Err(err) => return Err(map_reserve_err(err.with_value((None, iter))))
            };
            if let Err(err) = self.fallible_reserve_exact(capacity) {
                return Err(map_reserve_err(err.with_value((None, iter))))
            }
        }
        while let Some(item) = iter.next() {
            if let Err(err) = self.fallible_push(item?) {
                let (value, err) = err.recover_value();
                return Err(map_reserve_err(TryReserveError::new(err, (Some(value), iter))))
            }
        }
        Ok(self)
    }

    /// Tries to append the contents of a slice over [`T`] to the vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_append(&mut self, slice: &[T]) -> Result<(), TryReserveError<()>>
        where
            T: Clone;

    /// Tries to map and append the contents of a slice over [`U`] to the vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_append_map<U, F>(&mut self, slice: &[U], f: F) -> Result<(), TryReserveError<()>>
        where
            F: FnMut(&U) -> T;

    /// Tries to map and append the contents of a slice over [`U`] to the vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_insert(&mut self, index: SizeType, value: T) -> Result<(), TryReserveError<T>>;

    /// Tries moves the contents of another vector to this vector and sets the length of the other
    /// vector.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_move_from_vec<V, S>(&mut self, from: &mut V) -> Result<(), TryReserveError<()>>
        where
            V: Vector<T, S>,
            S: IntoUsize + FromUsize;

    /// Tries to map and move the contents of another vector to this vector and sets the length of the
    /// other vector to zero.
    ///
    /// This is a part of operations that may fail because the vector fails to reserve more space.
    fn fallible_move_from_vec_map<U, V, S, F>(
        &mut self,
        from: &mut V, 
        f: F,
    ) -> Result<(), TryReserveError<()>>
        where 
            V: Vector<U, S>,
            S: IntoUsize + FromUsize,
            F: FnMut(U) -> T;
}
