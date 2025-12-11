use crate::{
    win,
    gpu,
};

pub enum Event<'a, 'b, 'c> {
    /// Nox is updating.
    ///
    /// Happens once per frame.
    Update {
        win: &'a mut win::WindowContext,
        gpu: gpu::GpuContext<'a>,
    },
    /// Nox is recording compute commands.
    ///
    /// Commands dispatched with `commands` are synchronized to
    /// run before any render work.
    ///
    /// Happens once per frame:
    ComputeWork {
        commands: &'a mut gpu::ComputeCommands<'b>,
    },
    /// Nox is recording transfer commands.
    ///
    /// Commands dispatched with `commands` are *not*
    /// synchronized by default.
    ///
    /// Synchronization can be achieved using a custom [`TimelineSemaphore`].
    TransferWork {
        request_id: gpu::CommandRequestId,
        commands: &'a mut gpu::TransferCommands<'b, 'c>,
    },
    /// The frame buffer has been resized or (re)created.
    FrameBufferResized {
        gpu: &'a mut gpu::GpuContext<'b>,
        new_size: gpu::Dimensions,
    },
    /// Nox is constructing a [`FrameGraph`].
    ///
    /// # Safety
    /// The resources used by any pending transfers *must* not be used while rendering.
    ///
    /// Happens once per frame.
    Render {
        frame_graph: &'a mut gpu::FrameGraph<'b>,
        pending_transfers: &'a [gpu::CommandRequestId],
    },
    /// Nox is recording render commands.
    ///
    /// Happens for each pass added to the current [`FrameGraph`]
    RenderWork {
        pass_id: gpu::PassId,
        commands: &'a mut gpu::RenderCommands<'b, 'c>,
    },
}
