use core::{
    ffi::{CStr, FromBytesWithNulError},
    slice,
    fmt::{self, Display},
    ops::Deref,
};

/// A trait for types, which can be trivially converted from and to [`u32`].
pub trait Word: Sized {
    
    fn from_word(word: u32) -> Self;
}

impl Word for u32 { 

    #[inline]
    fn from_word(word: u32) -> Self {
        word
    }
}

#[inline]
pub(crate) fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(slice.as_ptr() as *const u8, size_of_val(slice))
    }
}

/// Represents a string inside SPIR-V.
//
/// Doesn't do any extra allocations and is trivially copyable.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CompilerStr<'a> {
    bytes: &'a [u8],
}

impl<'a> Deref for CompilerStr<'a> {

    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.bytes
    }
}

impl<'a> CompilerStr<'a> {

    pub fn new(words: &'a [u32]) -> Self {
        let bytes = slice_as_bytes(words);
        let len = bytes
            .iter()
            .copied()
            .take_while(|&byte| byte != 0)
            .count() + 1;
        Self {
            bytes: &bytes[0..len],
        }
    }

    #[inline]
    pub fn to_cstr(&self) -> Result<&'a CStr, FromBytesWithNulError> {
        CStr::from_bytes_with_nul(
            self.bytes
        )
    }
}

impl Display for CompilerStr<'_> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", str
            ::from_utf8(&self.bytes[0..self.bytes.len()-1])
            .unwrap_or("<utf8-error>"
        ))
    }
}

/// Specifies the value of a literal constant.
#[derive(Clone, Copy, Debug)]
pub enum Literal {
    /// Not representable in Rust without nightly.
    F16(u16),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl Literal {

    #[inline]
    pub fn as_usize(self) -> Option<usize> {
        match self {
            Self::I8(x) => Some(x as usize),
            Self::I16(x) => Some(x as usize),
            Self::I32(x) => Some(x as usize),
            Self::I64(x) => Some(x as usize),
            Self::U8(x) => Some(x as usize),
            Self::U16(x) => Some(x as usize),
            Self::U32(x) => Some(x as usize),
            Self::U64(x) => Some(x as usize),
            _ => None,
        }
    }
}

impl Display for Literal {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F16(x) => write!(f, "{x:#x}"),
            Self::F32(x) => write!(f, "{x}"),
            Self::F64(x) => write!(f, "{x}"),
            Self::I8(x) => write!(f, "{x}"),
            Self::I16(x) => write!(f, "{x}"),
            Self::I32(x) => write!(f, "{x}"),
            Self::I64(x) => write!(f, "{x}"),
            Self::U8(x) => write!(f, "{x}"),
            Self::U16(x) => write!(f, "{x}"),
            Self::U32(x) => write!(f, "{x}"),
            Self::U64(x) => write!(f, "{x}"),
        }
    }
}
