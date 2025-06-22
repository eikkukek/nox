use core::{
    cell::UnsafeCell,
};

#[repr(transparent)]
pub struct Triv<T>(UnsafeCell<T>);
