use core::{
    fmt::{self, Display, Formatter},
    error,
};

use nox::error::{
    AnyError,
    ResourceError,
};

#[derive(Debug)]
pub enum GuiError {
    ResourceError(ResourceError),
    UndefinedOutputSamples,
}

impl Display for GuiError {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceError(_) => write!(f, "nox resource error"),
            Self::UndefinedOutputSamples => write!(f, "")
        }
    }
}

impl Into<AnyError> for GuiError {

    fn into(self) -> AnyError {
        AnyError::new("nox gui error", self)
    }
}
