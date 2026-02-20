mod array_string;

use core::{
    error::Error,
    fmt::Display,
};

pub use array_string::ArrayString;

#[derive(Debug)]
pub enum StringError {
    Utf8Error(usize)
}

impl Display for StringError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Utf8Error(valid_up_to) => write!(f, "utf8 error (valid up to {valid_up_to})"),
        }
    }
}

impl Error for StringError {}

#[macro_export]
macro_rules! array_format {
    ($($arg:tt)*) => {
        $crate::string_types::ArrayString::format(format_args!($($arg)*))
    }
}

#[macro_export]
macro_rules! array_string {
    ($arg:tt) => {
        $crate::string_types::ArrayString::from_str($arg)
    };
}

#[cfg(feature = "std")]
mod std_features {

    pub use std::string::*;
}

#[cfg(feature = "std")]
pub use std_features::*;
