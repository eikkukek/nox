//! A wrapper for types that aren't normally hashable.
//!
//! This module provides implementations for [`Hashable<f32>`] and [`Hashable<f64>`].

pub use core::{
    hash::{Hash, Hasher},
};

/// A wrapper for types that aren't normally hashable
///
/// Implemented for [`f32`] and [`f64`]
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Hashable<T>(pub T)
    where
        Self: Hash + PartialEq + Eq;

impl<T> Hashable<T>
    where
        Self: Hash + PartialEq + Eq
{

    pub fn to_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Hashable<T>
    where
        Self: Hash + PartialEq + Eq
{

    fn from(value: T) -> Self {
        Self(value)
    }
}

macro_rules! impl_float {
    ($($t:ty),+) => {
        $(
        impl Hash for Hashable<$t> {

            fn hash<H: Hasher>(&self, state: &mut H) {
                self.0.to_bits().hash(state);
            }
        }

        impl PartialEq for Hashable<$t> {

            fn eq(&self, other: &Self) -> bool {
                self.0.to_bits() == other.0.to_bits()
            }
        }

        impl Eq for Hashable<$t> {}
        )+
    };
}

impl_float!(f32, f64);
