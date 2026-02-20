use core::{
    ops::{Deref, DerefMut},
    fmt::{self, Display},
};

use super::*;

/// A smart pointer for niche cases where you either own the data or have a mutable reference to
/// the data.
///
/// Unlike [`Cow`] or [`CowMut`] this doesn't require the data to be cloneable/implement the
/// [`ToOwned`] trait, but this means that the borrowed type is just a mutable reference to the
/// data.
///
/// The implementation is dead simple, but has its use cases.
///
/// For unsized types, consider using [`Cow`], [`CowMut`] or [`BoxCow`].
///
/// # Examples
///
/// ``` rust
/// use nox_mem::borrow::SizedCowMut;
///
/// let cow1 = SizedCowMut::Owned(10);
/// let mut value = 10;
/// let cow2 = SizedCowMut::Borrowed(&mut value);
/// assert_eq!(cow1, cow2);
/// ```
#[derive(Debug)]
pub enum SizedCowMut<'a, T>
    where T: Sized
{
    Borrowed(&'a mut T),
    Owned(T),
}

impl<T> SizedCowMut<'_, T> {

    /// Returns whether the data is borrowed.
    #[inline(always)]
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns whether the data is owned.
    #[inline(always)]
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Acquires a mutable reference to the owned data.
    ///
    /// Equivalent to [`Cow::to_mut`], but this never clones the data as there is already a mutable
    /// reference of the data.
    #[inline(always)]
    pub fn to_mut(&mut self) -> &mut T
        where T: Clone
    {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o,
        }
    }

    /// Consumes self and returns the data.
    ///
    /// If the data is already owned, this returns the owned data. Otherwise this clones the
    /// data.
    ///
    /// Equivalent to [`Cow::into_owned`].
    #[inline(always)]
    pub fn into_owned(self) -> T
        where T: Clone
    {
        match self {
            Self::Borrowed(b) => b.clone(),
            Self::Owned(o) => o,
        }
    }
}

impl<T> Deref for SizedCowMut<'_, T>
{

    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o,
        }
    }
}

impl<T> DerefMut for SizedCowMut<'_, T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o,
        }
    }
}

impl<T> Borrow<T> for SizedCowMut<'_, T>
{

    #[inline(always)]
    fn borrow(&self) -> &T {
        self.deref()
    }
}

impl<T> Display for SizedCowMut<'_, T>
    where T: Display
{

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) => b.fmt(f),
            Self::Owned(o) => o.fmt(f),
        }
    }
}

impl<T, B> PartialEq<T> for SizedCowMut<'_, B>
    where
        T: Deref<Target = B>,
        B: PartialEq,
{

    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.deref() == other.deref()
    }
}

impl<T> Eq for SizedCowMut<'_, T>
    where T: Eq
{}
