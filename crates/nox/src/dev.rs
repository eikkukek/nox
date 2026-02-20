mod error_context;

pub mod prelude {
    pub use super::super::prelude::*;
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
        {
            let loc = $crate::dev::error::location!();
            compact_str::format_compact!($fmt, loc = loc $(,$arg)*)
        }
    };
}
pub(crate) use format_location;

macro_rules! or_flag {
    ($flags:expr, $flag:expr, $value:expr $(,)?) => {
        $flags |= $flag & ($value as u32) << $flag.trailing_zeros();
    };
}
pub(crate) use or_flag;
