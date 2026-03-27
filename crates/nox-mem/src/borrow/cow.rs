use super::*;

use core::{
    ops::{Deref, DerefMut},
    fmt::{self, Debug, Display},
};

/// A smart pointer for niche cases where you either own the data or have a mutable reference to
/// the data.
///
/// Works like [`Cow`], but holds a mutable reference to borrowed data.
///
/// # Examples
/// ``` rust
/// use nox_mem::borrow::CowMut;
///
/// let cow1 = CowMut::<[i32]>::Owned(vec![1, 2, 3]);
/// let mut value = [1, 2, 3];
/// let cow2 = CowMut::Borrowed(value.as_mut_slice());
/// assert_eq!(cow1, cow2);
/// ```
pub enum CowMut<'a, B>
    where
        B: ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{
    Borrowed(&'a mut B),
    Owned(<B as ToOwned>::Owned),
}

impl<B> CowMut<'_, B>
    where
        B: ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{

    /// Returns whether the data is borrowed.
    #[inline]
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns whether the data is owned.
    #[inline]
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }
    
    /// Acquires a mutable reference to the owned data, specifically the [`ToOwned::Owned`]
    /// type of `B`.
    ///
    /// Equivalent to [`Cow::to_mut`].
    ///
    /// If the data is not already owned, this clones the data.
    ///
    /// If you want a mutable reference to `B` without cloning the data, simply use
    /// [`DerefMut`].
    #[inline]
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned {
        match self {
            Self::Borrowed(b) => {
                *self = Self::Owned(b.to_owned());
                match self {
                    Self::Owned(o) => o,
                    Self::Borrowed(_) => unreachable!(),
                }
            },
            Self::Owned(o) => o,
        }
    }

    /// Consumes self and returns the [`ToOwned::Owned`] type of `B`.
    ///
    /// If the data is already owned, this returns the owned data. Otherwise this clones the
    /// data.
    ///
    /// Equivalent to [`Cow::into_owned`].
    #[inline]
    pub fn into_owned(self) -> <B as ToOwned>::Owned
    {
        match self {
            Self::Borrowed(b) => b.to_owned(),
            Self::Owned(o) => o,
        }
    }
}

impl<B> Deref for CowMut<'_, B>
    where
        B: ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{
    type Target = B;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o.borrow(),
        }
    }
}

impl<B> DerefMut for CowMut<'_, B>
    where
        B: ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o.borrow_mut(),
        }
    }
}

impl<B> Borrow<B> for CowMut<'_, B> 
    where
        B: ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{

    fn borrow(&self) -> &B {
        match self {
            Self::Borrowed(b) => b,
            Self::Owned(o) => o.borrow(),
        }
    }
}

impl<T, B> PartialEq<T> for CowMut<'_, B>
    where
        T: Borrow<B>,
        B: PartialEq + ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{

    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.deref() == other.borrow()
    }
}

impl<B> Eq for CowMut<'_, B>
    where
        B: Eq + ?Sized + ToOwned,
        <B as ToOwned>::Owned: BorrowMut<B>,
{}

impl<B> Debug for CowMut<'_, B>
    where
        B: Debug + ?Sized + ToOwned,
        <B as ToOwned>::Owned: Debug + BorrowMut<B>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) =>
                f.debug_tuple("Borrowed")
                .field(b)
                .finish(),
            Self::Owned(o) =>
                f.debug_tuple("Owned")
                .field(o)
                .finish()
            ,
        }
    }
}

impl<B> Display for CowMut<'_, B>
    where 
        B: Display + ?Sized + ToOwned,
        <B as ToOwned>::Owned: Display + BorrowMut<B>,
{

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) => b.fmt(f),
            Self::Owned(o) => o.fmt(f)
        }
    }
}
