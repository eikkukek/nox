use core::{
    ops::Deref,
    ptr,
};

use crate::num::{NonZeroInteger, Integer};

pub trait OptionExt<T> {

    /// [`Option::get_or_insert_with`] with a closure that may fail.
    fn get_or_try_insert_with<E, F>(&mut self, f: F) -> Result<&mut T, E>
        where F: FnOnce() -> Result<T, E>;

    /// [`Option::unwrap_or_else`] with a closure that may fail.
    fn unwrap_or_try_else<F, E>(self, f: F) -> Result<T, E>
        where F: FnOnce() -> Result<T, E>;

    /// Edits the value contained if [`Some`].
    fn edit<F>(&mut self, f: F)
        where F: FnOnce(&mut T);

    /// Gets a pointer to the value contained if [`Some`] and a null pointer otherwise.
    /// # Example
    /// ``` rust
    /// use nox_mem::option::OptionExt;
    ///
    /// let value = 10;
    /// let opt = Some(&value);
    /// assert!(!opt.as_ptr().is_null());
    /// assert!(None::<&i32>.as_ptr().is_null());
    /// ```
    fn as_ptr(&self) -> *const <T as Deref>::Target
        where
            T: Deref,
            <T as Deref>::Target: Sized;

    fn unwrap_or_sentinel<U>(self, x: U) -> U
        where
            U: Integer,
            T: NonZeroInteger<U>;

    fn unwrap_or_sentinel_with<U, F>(self, f: F) -> U
        where
            U: Integer,
            F: FnOnce() -> U,
            T: NonZeroInteger<U>;
}

pub trait OptionSlice<T> {

    fn as_slice_ptr(&self) -> *const T;
}

impl<T> OptionExt<T> for Option<T> {

    #[inline]
    fn get_or_try_insert_with<E, F>(&mut self, f: F) -> Result<&mut T, E>
        where F: FnOnce() -> Result<T, E>
    {
        match self {
            Self::Some(t) => Ok(t),
            None => {
                let t = f()?;
                Ok(self.insert(t))
            }
        }
    }

    #[inline]
    fn unwrap_or_try_else<F, E>(self, f: F) -> Result<T, E>
        where F: FnOnce() -> Result<T, E>
    {
        match self {
            Self::Some(t) => Ok(t),
            None => {
                let t = f()?;
                Ok(t)
            }
        }
    }

    #[inline]
    fn edit<F>(&mut self, f: F)
        where F: FnOnce(&mut T) 
    {
        if let Some(t) = self {
            f(t)
        }
    }

    #[inline]
    fn as_ptr(&self) -> *const <T as Deref>::Target
        where
            T: Deref,
            <T as Deref>::Target: Sized
    {
        match self.as_ref() {
            Some(value) => value.deref(),
            None => ptr::null(),
        }
    }

    #[inline]
    fn unwrap_or_sentinel<U>(self, x: U) -> U
        where
            U: Integer,
            T: NonZeroInteger<U>
    {
        match self {
            Some(value) => value.get(),
            None => x,
        }
    }

    #[inline]
    fn unwrap_or_sentinel_with<U, F>(self, f: F) -> U
        where
            U: Integer,
            F: FnOnce() -> U,
            T: NonZeroInteger<U>
    {
        match self {
            Some(value) => value.get(),
            None => f(),
        }
        
    }
}

impl<T> OptionSlice<T> for Option<&[T]> {
    
    #[inline]
    fn as_slice_ptr(&self) -> *const T {
        match self {
            Some(value) => value.as_ptr(),
            None => ptr::null(),
        }
    }
}

impl<T> OptionSlice<T> for Option<&mut [T]> {
    
    #[inline]
    fn as_slice_ptr(&self) -> *const T {
        match self {
            Some(value) => value.as_ptr(),
            None => ptr::null(),
        }
    }
}
