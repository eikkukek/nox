mod array_string;

use core::{
    error::Error,
    fmt::Display,
};

pub use array_string::ArrayString;

#[derive(Debug)]
pub enum StringError {
    InvalidAscii
}

impl Display for StringError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidAscii => write!(f, "invalid ascii"),
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
