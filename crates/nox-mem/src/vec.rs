mod pointer;
mod alloc_vec;
mod array_vec;
mod non_null;

pub use pointer::Pointer;
pub use alloc_vec::{
    AllocVecBase,
    DynVec, DynVec32,
    FixedVec, FixedVec32,
    DynPolicy, FixedPolicy,
    DynPolicy32, FixedPolicy32,
};
pub use non_null::{NonNullVecBase, NonNullVec, NonNullVec32};
pub use array_vec::ArrayVec;

#[cfg(feature = "std")]
mod std_features {

    /// Macro for creating a new [`Vec32`].
    ///
    /// # Examples
    /// ``` rust
    /// use nox_mem::vec32;
    ///
    /// let vec1 = vec32!["foo"; 10];
    /// let vec2 = vec32![0, 1, 2, 3, 4];
    /// ```
    #[macro_export]
    macro_rules! vec32 {
        [$v:expr; $n:expr] => {
            $crate::vec::Vec32::with_len_reserve_exact($n, $v)
        };
        [$($elem:expr),* $(,)?] => {
            $crate::vec::Vec32::from([$($elem),*])
        };
    }

    pub use super::alloc_vec::{StdVecBase, StdVec, Vec32};
}

#[cfg(feature = "std")]
pub use std_features::*;
