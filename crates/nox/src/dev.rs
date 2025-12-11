mod error_context;

pub mod export {
    pub use super::super::export::*;
}

pub mod utility {

    pub use super::super::utility::*;
}

pub mod error {
    pub use nox_error::*;
    pub use super::error_context::*;
}

macro_rules! has_bits {
    ($a:expr, $b:expr) => ($a & $b == $b)
}
pub(crate) use has_bits;

macro_rules! has_not_bits {
    ($a:expr, $b:expr) => ($a & $b != $b)
}
pub(crate) use has_not_bits;

macro_rules! format_location {
    ($fmt:literal $(,$arg:expr)* $(,)?) => {
        { let loc = $crate::dev::error::location!(); compact_str::format_compact!($fmt $(,$arg)*) }
    };
}
pub(crate) use format_location;
