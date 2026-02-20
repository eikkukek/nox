#[cfg(not(feature = "std"))]
mod no_std {

    use core::{
    error::Error,
        fmt::{self, Debug, Formatter},
    };

    use nox_proc::Error;

    use crate::num::IntoUsize;

    #[derive(Debug, Error)]
    pub enum TryReserveErrorKind {
        #[display("allocation failed")]
        AllocError,
        #[display(
            "maximum capacity of {max_capacity} exceeded, requested capacity was {requested_capacity}"
        )]
        MaxCapacityExceeded {
            max_capacity: usize,
            requested_capacity: usize,
        },
    }

    #[derive(Error)] #[display("{error}")]
    pub struct TryReserveError<T> {
        error: TryReserveErrorKind,
        value: T,
    }

    impl<T> TryReserveError<T> {

        #[inline(always)]
        pub fn new(error: TryReserveErrorKind, value: T) -> Self {
            Self {
                error,
                value,
            }
        }

        #[inline(always)]
        pub fn alloc_error<E>(_error: E, value: T) -> Self
            where E: Error + Send + Sync + 'static
        {
            Self {
                error: TryReserveErrorKind::AllocError,
                value,
            }
        }

        #[inline(always)]
        pub fn max_capacity_exceeded<SizeType: IntoUsize>(
            max_capacity: SizeType,
            requested_capacity: usize,
            value: T,
        ) -> Self {
            Self {
                error: TryReserveErrorKind::MaxCapacityExceeded {
                    max_capacity: max_capacity.into_usize(),
                    requested_capacity,
                },
                value,
            }
        }

        #[inline(always)]
        pub fn with_value<U>(self, value: U) -> TryReserveError<U> {
            TryReserveError {
                error: self.error,
                value,
            }
        }

        #[inline(always)]
        pub fn recover_value(self) -> (T, TryReserveErrorKind) {
            (self.value, self.error)
        }
    }

    impl<T> Debug for TryReserveError<T> {

        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.error.fmt(f)
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std::*;

#[cfg(feature = "std")]
mod std_features {

    use std::{
        collections::hash_map::{Entry, VacantEntry},
        boxed::Box,
    }; 

    use core::{
        error::Error,
        fmt::{self, Debug, Formatter},
    };

    use nox_proc::Error;

    use crate::num::IntoUsize;

    #[derive(Debug, Error)]
    pub enum TryReserveErrorKind {
        #[display("allocation failed")]
        AllocError {
            #[source(Some(alloc_error.as_ref()))] alloc_error: Box<dyn Error + Send + Sync>,
        },
        #[display(
            "maximum capacity of {max_capacity} exceeded, requested capacity was {requested_capacity}"
        )]
        MaxCapacityExceeded {
            max_capacity: usize,
            requested_capacity: usize,
        },
    }

    #[derive(Error)] #[display("{error}")]
    pub struct TryReserveError<T> {
        error: TryReserveErrorKind,
        value: T,
    }

    impl<T> TryReserveError<T> {

        #[inline(always)]
        pub fn new(error: TryReserveErrorKind, value: T) -> Self {
            Self {
                error,
                value,
            }
        }

        #[inline(always)]
        pub fn alloc_error<E>(error: E, value: T) -> Self
            where E: Error + Send + Sync + 'static
        {
            Self {
                error: TryReserveErrorKind::AllocError { alloc_error: Box::new(error) },
                value,
            }
        }

        #[inline(always)]
        pub fn max_capacity_exceeded<SizeType: IntoUsize>(
            max_capacity: SizeType,
            requested_capacity: usize,
            value: T,
        ) -> Self {
            Self {
                error: TryReserveErrorKind::MaxCapacityExceeded {
                    max_capacity: max_capacity.into_usize(),
                    requested_capacity,
                },
                value,
            }
        }

        #[inline(always)]
        pub fn with_value<U>(self, value: U) -> TryReserveError<U> {
            TryReserveError {
                error: self.error,
                value,
            }
        }

        #[inline(always)]
        pub fn recover_value(self) -> (T, TryReserveErrorKind) {
            (self.value, self.error)
        }
    }

    impl<T> Debug for TryReserveError<T> {

        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.error.fmt(f)
        }
    }

    pub trait EntryExt<'a, K, V> {

        fn or_try_insert_with<F, E>(self, f: F) -> Result<&'a mut V, E>
            where
                F: FnOnce() -> Result<V, E>;

        fn or_try_insert_with_key<F, E>(self, f: F) -> Result<&'a mut V, E>
            where
                F: FnOnce(&K) -> Result<V, E>;

        fn vacant(self) -> Option<VacantEntry<'a, K, V>>;
    }

    impl<'a, K, V> EntryExt<'a, K, V> for Entry<'a, K, V> {

        #[inline(always)]
        fn or_try_insert_with<F, E>(self, f: F) -> Result<&'a mut V, E>
            where
                F: FnOnce() -> Result<V, E>
        {
            match self {
                Self::Occupied(occ) => {
                    Ok(occ.into_mut())
                },
                Self::Vacant(vac) => {
                    let value = f()?;
                    Ok(vac.insert(value))
                }
            }
        }

        #[inline(always)]
        fn or_try_insert_with_key<F, E>(self, f: F) -> Result<&'a mut V, E>
            where
                F: FnOnce(&K) -> Result<V, E>
        {
            match self {
                Self::Occupied(occ) => {
                    Ok(occ.into_mut())
                },
                Self::Vacant(vac) => {
                    let value = f(vac.key())?;
                    Ok(vac.insert(value))
                }
            }
        }

        #[inline(always)]
        fn vacant(self) -> Option<VacantEntry<'a, K, V>> {
            match self {
                Self::Occupied(_) => {
                    None
                },
                Self::Vacant(vac) => Some(vac),
            }
        }
    }
}

#[cfg(feature = "std")]
pub use std_features::*;

/// A trait for determining how a given collection grows.
///
/// # Safety
/// Implementing this trait is unsafe because the implementation must adhere to certain rules. For
/// example, [`ReservePolicy::grow`] must never return a value that's less than `required`.
pub unsafe trait ReservePolicy<SizeType = usize> {

    fn can_grow() -> bool;

    fn grow(current: SizeType, required: usize) -> Result<SizeType, TryReserveError<()>>;

    fn grow_infallible(current: SizeType, required: usize) -> SizeType;
}
