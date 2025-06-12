use core::{
    ops::{Index, IndexMut},
    mem::{MaybeUninit, needs_drop},
    ptr,
};

#[derive(Debug)]
pub enum CapacityError {
    Fixed {
        capacity: usize,
    },
    InvalidReservation {
        current: usize,
        requested: usize,
    },
    AllocFailed {
        new_capacity: usize,
    },
}

pub trait CapacityPolicy {
    fn power_of_two() -> bool;
    fn can_grow() -> bool;
    fn grow(current: usize, required: usize) -> Option<usize>;
}

pub trait MemoryStrategy {
    unsafe fn move_elements(src: *const Self, dst: *mut Self, len: usize);
    unsafe fn insert(ptr: *mut Self, value: Self, index: usize, len: usize) -> *mut Self;
    unsafe fn drop_in_place(ptr: *mut Self, len: usize);
}

pub trait DuplicateStrategy: Clone {
    unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize);
}

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
            T: Eq;

    fn push_if_unique(&mut self,value: T) -> Result<Option<&mut T>, CapacityError>
        where
            T: Eq
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

pub struct Dyn {}

impl CapacityPolicy for Dyn {

    fn power_of_two() -> bool {
        true
    }

    fn can_grow() -> bool {
        true
    }

    fn grow(current: usize, required: usize) -> Option<usize> {
        if required <= current { None }
        else { Some(required.max(2).next_power_of_two()) }
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

impl<T> MemoryStrategy for T {

    #[inline(always)] 
    unsafe fn move_elements(src: *const Self, dst: *mut Self, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    dst.add(i).write(src.add(i).read())
                }
            }
        }
        else {
            unsafe { ptr::copy_nonoverlapping(src, dst, len); }
        }
    }

    #[inline(always)]
    unsafe fn insert(ptr: *mut Self, value: Self, index: usize, len: usize) -> *mut Self {
        assert!(len >= index);
        if needs_drop::<T>() {
            unsafe {
                for i in (index + 1..=len).rev() {
                    ptr.add(i).write(ptr.add(i - 1).read());
                }
                let res = ptr.add(index);
                res.write(value);
                res
            }
        }
        else {
            unsafe {
                ptr::copy(ptr, ptr.add(1), len - index);
                let res = ptr.add(index);
                res.write(value);
                res
            }
        }
    }

    #[inline(always)]
    unsafe fn drop_in_place(ptr: *mut Self, len: usize) {
        if needs_drop::<T>() {
            unsafe {
                for i in 0..len {
                    ptr::drop_in_place(ptr.add(i));
                }
            }
        }
    }
}

impl<T: Clone> DuplicateStrategy for T {

    #[inline(always)]
    default unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize) {
        unsafe {
            for i in 0..len {
                dst.add(i).write(src.add(i).read().clone());
            }
        }
    }
}

impl<T: Copy> DuplicateStrategy for T {

    #[inline(always)]
    unsafe fn duplicate(src: *const Self, dst: *mut Self, len: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, len); }
    }
}
