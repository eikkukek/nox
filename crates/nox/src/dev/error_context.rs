use nox_error::Display;

use super::error::Location;

#[derive(Display)]
pub enum ErrorContext {

    #[display("internal vec error at {0}")]
    VecError(Location),

    #[display("internal string conversion error at {0}")]
    StringConversionError(Location),

    #[display("init error at {0}")]
    InitError(Location),

    #[display("event error at {0}")]
    EventError(Location),

    #[display("failed to begin command buffer at {0}")]
    CommandBufferBeginError(Location),

    #[display("failed to end command buffer at {0}")]
    CommandBufferEndError(Location),

    #[display("failed to submit to graphics queue at {0}")]
    GraphicsQueueSubmitError(Location),

    #[display("failed to submit to transfer queue at {0}")]
    TransferQueueSubmitError(Location),

    #[display("failed to submit to compute queue at {0}")]
    ComputeQueueSubmitError(Location),
}
