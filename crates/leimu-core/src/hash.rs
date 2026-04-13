//! An extension of [`hash`][1].
//!
//! # New types
//! - [`Hashable`]: A [`Hash`] and [`Eq`] wrapper for types that aren't normally hashable.
//!
//! [1]: core::hash

pub use core::{
    hash::*,
};

use core::ops::{Deref, DerefMut};

use crate::slice::value_as_bytes;

/// A wrapper for types that aren't normally hashable.
///
/// [`Hash`] and [`Eq`] are implemented by hashing and comparing the bits of the types.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Hashable<T>(pub T);

impl<T> Hashable<T>
{
    #[inline]
    pub fn to_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Hashable<T>
{

    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Hash for Hashable<T> {

    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        value_as_bytes(self).hash(state);
    }
}

impl<T> PartialEq for Hashable<T> {

    #[inline]
    fn eq(&self, other: &Self) -> bool {
        value_as_bytes(self).eq(value_as_bytes(other))
    }
}

impl<T> Eq for Hashable<T>
    where [u8]: Eq {}

impl<T> Deref for Hashable<T> {

    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Hashable<T> {

    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
