use std::io;

use crate::vec_types::CapacityError;

pub enum LoadError {
    IoError(io::Error),
    CapacityError(CapacityError),
    InvalidUtf8,
    InvalidPath,
    InvalidType,
    TypeMismatch,
}

impl From<io::Error> for LoadError {

    fn from(value: io::Error) -> Self {
        LoadError::IoError(value)
    }
}

impl From<CapacityError> for LoadError {

    fn from(value: CapacityError) -> Self {
        LoadError::CapacityError(value)
    }
}
