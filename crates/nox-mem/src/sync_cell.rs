use std::sync::{Mutex, MutexGuard, PoisonError};

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

pub struct ARefCell<T> {
    mtx: Mutex<UnsafeCell<T>>,
    ref_count: u32,
}

impl<T: Sync> ARefCell<T> {

    pub const fn new(value: T) -> Self {
        Self {
            mtx: Mutex::new(UnsafeCell::new(value)),
            ref_count: 0,
        }
    }

    pub fn borrow_mut(&self) -> Result<RefMut<'_, T>, PoisonError<MutexGuard<UnsafeCell<T>>>> {
        let guard = self.mtx
            .lock()
            .map_err(|e| PoisonError::new(e.into_inner().into()))?;
    }
}

pub struct Ref<'a, T: Sync> {
    t: &'a T,
    cell: *mut UnsafeCell<ARefCell<T>>,
}

impl<'a, T: Sync> Deref for Ref<'a, T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.t
    }
}

impl<'a, T: Sync> Drop for Ref<'a, T> {

    fn drop(&mut self) {
        unsafe { (*self.cell).get_mut().ref_count -= 1; }
    }
}

pub struct RefMut<'a, T: Sync> {
    t: &'a mut T,
    cell: *mut UnsafeCell<ARefCell<T>>,
}

impl<'a, T: Sync> Deref for RefMut<'a, T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.t
    }
}

impl<'a, T: Sync> DerefMut for RefMut<'a, T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.t
    }
}

impl<'a, T: Sync> Drop for RefMut<'a, T> {

    fn drop(&mut self) {
        unsafe { (*self.cell).get_mut().ref_count = 0; }
    }
}
