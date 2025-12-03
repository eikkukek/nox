use nox_mem::vec_types::VecError;

use crate::{ResourceError, Error};

#[derive(Debug)]
pub enum FrameGraphError {
    VecError(VecError),
    ResourceError(ResourceError),
    RenderCommandError(Box<Error>),
}

impl FrameGraphError {

    pub fn new(err: impl Into<FrameGraphError>) -> Self {
        err.into()
    }
}

impl core::fmt::Display for FrameGraphError {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VecError(_) => write!(f, "vec error"),
            Self::ResourceError(_) => write!(f, "resource error"),
            Self::RenderCommandError(_) => write!(f, "render commands error"),
        }
    }
}

impl core::error::Error for FrameGraphError {

    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::VecError(err) => Some(err),
            Self::ResourceError(err) => Some(err),
            Self::RenderCommandError(err) => Some(err),
        }
    }
}

impl From<VecError> for FrameGraphError {

    fn from(value: VecError) -> Self {
        Self::VecError(value)
    }
}

impl From<ResourceError> for FrameGraphError {

    fn from(value: ResourceError) -> Self {
        Self::ResourceError(value)
    }
}
