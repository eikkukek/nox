use super::*;

use std::boxed::Box;

use core::{
    ops::{Deref, DerefMut},
    fmt::{self, Debug, Display},
};

/// A trait for cloning trait objects.
///
/// Similar to [`ToOwned`] without the restriction of needing to implementing [`Borrow`].
pub trait BoxClone {

    type Cloned: ?Sized;
    
    /// Clones self to a [`Box`].
    fn box_clone(&self) -> Box<Self::Cloned>; 
}

pub trait DynBorrow: BoxClone {

    /// Borrows self.
    fn dyn_borrow(&self) -> &Self::Cloned;
}

pub trait DynBorrowMut: DynBorrow {

    /// Borrows self mutably.
    fn dyn_borrow_mut(&mut self) -> &mut Self::Cloned;
}

pub enum BoxCowBase<Owned, Borrowed>
    where
        Owned: ?Sized,
        Borrowed: BoxClone<Cloned = Owned>
{
    Borrowed(Borrowed),
    Owned(Box<Owned>),
}

/// A smart pointer that combines [`Cow`] and [`Box`].
///
/// For sized types that don't implement [`Clone`], consider using [`SizedCowMut`] that don't
/// allocate any extra space.
///
/// For slices and [`str`], consider using [`Cow`], [`CowMut`].
pub type BoxCow<'a, T> = BoxCowBase<T, &'a dyn DynBorrow<Cloned = T>>;

pub type BoxCowSync<'a, T>
    = BoxCowBase<T, &'a (dyn DynBorrow<Cloned = T> + Send + Sync)>;

pub type BoxCowMut<'a, T> = BoxCowBase<T, &'a mut dyn DynBorrowMut<Cloned = T>>;

impl<Owned, Borrowed> BoxCowBase<Owned, Borrowed>
    where
        Owned: ?Sized,
        Borrowed: BoxClone<Cloned = Owned>
{

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
    /// Equivalent to [`Cow::to_mut`].
    ///
    /// If the data is not already owned, this clones the data.
    #[inline(always)]
    pub fn to_mut(&mut self) -> &mut Owned {
        match self {
            Self::Borrowed(b) => {
                *self = Self::Owned(b.box_clone());
                match self {
                    Self::Owned(o) => o,
                    Self::Borrowed(_) => unreachable!(),
                }
            },
            Self::Owned(o) => o,
        }
    }

    /// Consumes self and returns the boxed data.
    ///
    /// If the data is already owned, this returns the owned data. Otherwise this clones the
    /// data.
    ///
    /// Equivalent to [`Cow::into_owned`].
    #[inline(always)]
    pub fn into_owned(self) -> Box<Owned>
    {
        match self {
            Self::Borrowed(b) => b.box_clone(),
            Self::Owned(o) => o,
        }
    }
}

impl<T, U> Deref for BoxCowBase<T, U>
    where
        T: ?Sized,
        U: DynBorrow<Cloned = T>
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(b) => b.dyn_borrow(),
            Self::Owned(o) => o,
        }
    }
}

impl<T, U> DerefMut for BoxCowBase<T, U>
    where
        T: ?Sized,
        U: DynBorrowMut<Cloned = T>
{

    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Borrowed(b) => b.dyn_borrow_mut(),
            Self::Owned(o) => o,
        }
    }
}

impl<T, U> Borrow<T> for BoxCowBase<T, U>
    where
        T: ?Sized,
        U: DynBorrow<Cloned = T>
{

    #[inline(always)]
    fn borrow(&self) -> &T {
        match self {
            Self::Borrowed(b) => b.dyn_borrow(),
            Self::Owned(o) => o,
        }
    }
}

impl<A, T, U> PartialEq<T> for BoxCowBase<A, U>
    where
        T: Deref<Target = A>,
        A: PartialEq + ?Sized,
        U: DynBorrow<Cloned = A>,
{

    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.deref() == other.deref()
    }
}

impl<T, U> Eq for BoxCowBase<T, U>
    where
        T: PartialEq + ?Sized,
        U: DynBorrow<Cloned = T>,
{}

impl<T, U> Display for BoxCowBase<T, U>
    where
        T: Display + ?Sized,
        U: DynBorrow<Cloned = T>,
{

    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) => b.dyn_borrow().fmt(f),
            Self::Owned(o) => o.fmt(f)
        }
    }
}

impl<T, U> Debug for BoxCowBase<T, U>
    where
        T: ?Sized + Debug,
        U: DynBorrow<Cloned = T>
{
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Borrowed(b) =>
                f.debug_tuple("Borrowed")
                .field(&b.dyn_borrow())
                .finish(),
            Self::Owned(o) =>
                f.debug_tuple("Owned")
                .field(o)
                .finish()
            ,
        }
    }
}

impl<T: ?Sized> BoxClone for &dyn BoxClone<Cloned = T> {

    type Cloned = T;

    #[inline(always)]
    fn box_clone(&self) -> Box<Self::Cloned> {
        (*self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &dyn DynBorrow<Cloned = T> {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (*self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &(dyn DynBorrow<Cloned = T> + Send + Sync) {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (*self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &dyn DynBorrowMut<Cloned = T> {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (*self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &(dyn DynBorrowMut<Cloned = T> + Send + Sync) {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (*self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &mut dyn DynBorrowMut<Cloned = T> {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (**self).box_clone()
    }
}

impl<T: ?Sized> BoxClone for &mut (dyn DynBorrowMut<Cloned = T> + Send + Sync) {

    type Cloned = T;

    fn box_clone(&self) -> Box<Self::Cloned> {
        (**self).box_clone()
    }
}

impl<T: ?Sized> DynBorrow for &dyn DynBorrow<Cloned = T>
    where Self: BoxClone<Cloned = T>
{

    #[inline(always)]
    fn dyn_borrow(&self) -> &Self::Cloned {
        (*self).dyn_borrow()
    }
}

impl<T: ?Sized> DynBorrow for &(dyn DynBorrow<Cloned = T> + Send + Sync)
    where Self: BoxClone<Cloned = T>
{

    #[inline(always)]
    fn dyn_borrow(&self) -> &Self::Cloned {
        (*self).dyn_borrow()
    }
}

impl<T: ?Sized> BoxClone for &mut dyn BoxClone<Cloned = T> {

    type Cloned = T;

    #[inline(always)]
    fn box_clone(&self) -> Box<Self::Cloned> {
        (**self).box_clone()
    }
}

impl<T: ?Sized> DynBorrowMut for &mut dyn DynBorrowMut<Cloned = T>
    where Self: DynBorrow<Cloned = T>
{
    
    #[inline(always)]
    fn dyn_borrow_mut(&mut self) -> &mut Self::Cloned {
        (*self).dyn_borrow_mut()
    }
}
