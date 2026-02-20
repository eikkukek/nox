mod no_std;

#[cfg(feature = "std")]
mod cow;

#[cfg(feature = "std")]
mod boxed;

#[cfg(feature = "std")]
mod map;

pub use core::borrow::*;

pub use no_std::*;

#[cfg(feature = "std")]
mod std_features {

    pub use std::borrow::*;

    pub use super::cow::*;
    pub use super::boxed::*;
    pub use super::map::*;
}

#[cfg(feature = "std")]
pub use std_features::*;
