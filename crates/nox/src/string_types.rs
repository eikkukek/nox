mod array_string;
//mod dyn_string;

pub use array_string::ArrayString;
//pub use dyn_string::DynString;

#[macro_export]
macro_rules! array_format {
    ($($arg:tt)*) => {
        $crate::string_types::ArrayString::format(format_args!($($arg)*))
    }
}
pub(crate) use array_format;

pub type SmallError = ArrayString<64>;
pub type LargeError = ArrayString<256>;
