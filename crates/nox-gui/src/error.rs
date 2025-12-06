use nox::{
    error::Error,
    ResourceError,
    FrameGraphError,
    CommandError,
    mem::vec_types::VecError,
};

#[derive(Error, Debug)] #[any("nox gui error")]
pub enum GuiError {

    #[display("undefined output samples")]
    UndefinedOutputSamples,

    #[display("begin not called")]
    BeginNotCalled,

    #[display("end not called")]
    EndNotCalled,

    #[display("graphics pipelines not created")]
    GraphicsPipelinesNotCreated,

    #[display("ring buffer out of memory")]
    RingBufferOutOfMemory,

    #[display("nox resource error")]
    ResourceError(#[source] #[from] ResourceError),

    #[display("nox frame graph error")]
    FrameGraphError(#[source] #[from] FrameGraphError),

    #[display("nox command error")]
    CommandError(#[source] #[from] CommandError),

    #[display("nox vec error")]
    VecError(#[source] #[from] VecError),
}

impl From<GuiError> for ResourceError {

    fn from(value: GuiError) -> Self {
        Self::Other(value.into())
    }
}

impl From<GuiError> for nox::Error {

    fn from(value: GuiError) -> Self {
        Self::Other(value.into())
    }
}

impl From<GuiError> for nox::CommandError {

    fn from(value: GuiError) -> Self {
        Self::Other(value.into())
    }
}
