mod array_string;

pub use array_string::ArrayString;

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
