//! Traits and types for use in custom collections.
//!
//! Contains:
//! - [`ReservePolicy`]: A trait that controls how collections grow their capacity.
//! - [`ReserveError`]: An error caused by reservation failure.

use core::{
    error::Error,
    fmt::{self, Display, Debug},
};

#[cfg(not(feature = "std"))]
mod no_std {

    use super::*;

    /// Indicates an allocation error.
    #[derive(Debug)]
    pub struct AllocError;

    impl AllocError {
        /// Creates a new allocation error.
        #[inline]
        pub fn new<E>(_error: E) -> Self
            where E: Error + Send + Sync + 'static
        {
            Self
        }
    }

    impl Display for AllocError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "allocation error")
        }
    }

    impl Error for AllocError {}
}

#[cfg(feature = "std")]
mod std_features {

    use super::*;

    use std::boxed::Box;

    /// Indicates an allocation error.
    #[derive(Debug)]
    pub struct AllocError(Box<dyn Error + Send + Sync>);

    impl AllocError {
        /// Creates a new allocation error.
        #[inline]
        pub fn new<E>(error: E) -> Self
            where E: Error + Send + Sync + 'static
        {
            Self(Box::new(error))
        }
    }

    impl Display for AllocError {

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for AllocError {}
}

#[cfg(feature = "std")]
pub use std_features::*;

#[cfg(not(feature = "std"))]
pub use no_std::*;

use crate::int::IntoUsize;

/// The type [`ReserveError`].
#[derive(Debug)]
pub enum ReserveErrorKind {
    /// Indicates an allocation error
    AllocError(AllocError),
    /// Indicates that the maximum capacity of a collection has been reached.
    MaxCapacityExceeded {
        /// The maximum capacity.
        max_capacity: usize,
        /// The requested capacity.
        requested_capacity: usize,
    },
}

impl Display for ReserveErrorKind {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocError(_) => write!(f, "allocation failed"),
            Self::MaxCapacityExceeded { max_capacity, requested_capacity } =>
                write!(f, "maximum capacity of {max_capacity} exceeded, requested capacity is {requested_capacity}"),
        }
    }
}

impl Error for ReserveErrorKind {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::AllocError(error) = self {
            error.source()
        } else { None }
    }
}

/// An error indicating reservation failure.
pub struct ReserveError<T> {
    error: ReserveErrorKind,
    value: T,
}

impl<T> Display for ReserveError<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl<T> Debug for ReserveError<T> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

impl<T> Error for ReserveError<T> {}

impl<T> ReserveError<T> {

    /// Creates a new error.
    #[inline]
    pub fn new(error: ReserveErrorKind, value: T) -> Self {
        Self {
            error,
            value,
        }
    }

    /// Creates a new allocation error.
    #[inline]
    pub fn alloc_error<E>(error: E, value: T) -> Self
        where E: Error + Send + Sync + 'static
    {
        Self {
            error: ReserveErrorKind::AllocError(AllocError::new(error)),
            value,
        }
    }

    /// Creates a new error indicating that the maximum capacity of a collection has been
    /// exceeded.
    #[inline]
    pub fn max_capacity_exceeded<SizeType: IntoUsize>(
        max_capacity: SizeType,
        requested_capacity: usize,
        value: T,
    ) -> Self {
        Self {
            error: ReserveErrorKind::MaxCapacityExceeded {
                max_capacity: max_capacity.into_usize(),
                requested_capacity,
            },
            value,
        }
    }

    /// Sets the [`recoverable`][1] value of the error.
    ///
    /// [1]: Self::recover_value
    #[inline]
    pub fn with_value<U>(self, value: U) -> ReserveError<U> {
        ReserveError {
            error: self.error,
            value,
        }
    }

    /// Recovers the value of the error.
    #[inline]
    pub fn recover_value(self) -> (T, ReserveErrorKind) {
        (self.value, self.error)
    }
} 

/// A trait for determining how a given collection grows.
///
/// # Safety
/// Implementing this trait is unsafe because the implementation must adhere to certain rules. For
/// example, [`ReservePolicy::grow`] must never return a value that's less than `required`.
pub unsafe trait ReservePolicy<SizeType = usize> {

    /// Returns whether the collection can grow.
    fn can_grow() -> bool;

    /// Tries to grow the capacity.
    fn grow(current: SizeType, required: usize) -> Result<SizeType, ReserveError<()>>;

    /// Grows the capacity, possibly panicking if there's an error.
    fn grow_infallible(current: SizeType, required: usize) -> SizeType;
}
