use core::{
    ffi::FromBytesWithNulError,
    fmt::{Display, self},
    error::Error,
};

use crate::{op, ParseError, Literal};
use super::*;

/// The main error type for when reflection fails.
#[derive(Debug)]
pub enum ReflectError {
    /// A [`CStr`][1] error.
    ///
    /// [1]: core::ffi::CStr
    FromBytesWithNulError(FromBytesWithNulError),
    /// A parsing error.
    Parse(ParseError),
    /// An error indicating that a type with [`Id`] was not found.
    InvalidTypeId(Id),
    /// An error indicating that a constant with [`Id`] was not found.
    InvalidConstantId(Id),
    /// An error indicating that an integer literal was expected, but a non-integer literal was
    /// found.
    NonIntegerLiteral(Literal),
    /// An error indicating that a constant with a [`literal`][1] was expected, but another
    /// type of constant was found.
    ///
    /// [1]: Literal
    ExpectedConstantLiteral {
        /// The constant found.
        found: String
    },
    /// An error indicating that a scalar type instruction was expected, but another instruction
    /// was found.
    ExpectedScalarType {
        /// The [`Code`][1] of the instruction found.
        ///
        /// [1]: op::Code
        found: op::Code,
    },
    /// An error indicating that a vector type instruction was expected, but another instruction
    /// was found.
    ExpectedVectorType {
        /// The [`Code`][1] of the instruction found.
        ///
        /// [1]: op::Code
        found: op::Code,
    },
    /// An error indicating that a required decoration was expected but not found.
    MissingRequiredDecoration(&'static str),
    /// An error indicating that an invalid placement of a runtime array.
    InvalidRuntimeArray,
}

impl Display for ReflectError {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromBytesWithNulError(_) => write!(f, "ffi string conversion error"),
            Self::Parse(_) => write!(f, "invalid spirv"),
            Self::InvalidTypeId(id) => write!(f, "invalid type id {id}"),
            Self::InvalidConstantId(id) => write!(f, "invalid constant id {id}"),
            Self::NonIntegerLiteral(literal) => write!(f, "non-integer litral {literal:?}"),
            Self::ExpectedConstantLiteral { found } => write!(f, "expected literal constant, found {found}"),
            Self::ExpectedScalarType { found } => write!(f, "expected scalar type, found {found}"),
            Self::ExpectedVectorType { found } => write!(f, "expected vector type, found {found}"),
            Self::MissingRequiredDecoration(dec) => write!(f, "missing required decoration {dec}"),
            Self::InvalidRuntimeArray
                => write!(f, "invalid runtime array, runtime arrays must be the last member of a struct")
        }
    }
}

impl From<FromBytesWithNulError> for ReflectError {

    #[inline]
    fn from(value: FromBytesWithNulError) -> Self {
        Self::FromBytesWithNulError(value)
    }
}

impl From<ParseError> for ReflectError {
    
    #[inline]
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl Error for ReflectError {

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FromBytesWithNulError(err) => Some(err),
            Self::Parse(err) => Some(err),
            _ => None,
        }
    }
}

/// The [`Result`] of a reflection operation.
pub type ReflectResult<T> = Result<T, ReflectError>;
