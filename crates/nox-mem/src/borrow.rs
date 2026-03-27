#[cfg(feature = "std")]
mod cow;

#[cfg(feature = "std")]
mod boxed;

pub use core::borrow::*;


#[cfg(feature = "std")]
mod std_features {

    pub use std::borrow::*;

    pub use super::cow::*;
    pub use super::boxed::*;
}

#[cfg(feature = "std")]
pub use std_features::*;
