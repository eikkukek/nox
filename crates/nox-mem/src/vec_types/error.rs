use core::ops::{Deref, DerefMut};

use crate::CapacityError;

#[derive(Clone, Copy, Debug)]
pub struct VecError {
    pub capacity_error: CapacityError,
}

impl From<CapacityError> for VecError {

    fn from(value: CapacityError) -> Self {
        Self { capacity_error: value }
    }
}

impl core::fmt::Display for VecError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.capacity_error)
    }
}

impl core::error::Error for VecError {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.capacity_error)
    }
}

impl Deref for VecError {

    type Target = CapacityError;

    fn deref(&self) -> &Self::Target {
        &self.capacity_error
    }
}

impl DerefMut for VecError {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capacity_error
    }
}
