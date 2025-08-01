use core::{
    cell::UnsafeCell,
    ops::{Deref},
    ops::{DerefMut},
};

pub struct MadCell<T> {
    cell: UnsafeCell<T>,
}

impl<T> MadCell<T> {

    pub unsafe fn get_mut(&self) -> &mut T {
        unsafe {
            &mut *self.cell.get()
        }
    }
}

impl<T> Deref for MadCell<T> {

    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.cell
    }
}

impl<T> DerefMut for MadCell<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cell
    }
}
