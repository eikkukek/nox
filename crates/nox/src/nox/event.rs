use crate::{
    event_loop,
    win,
    gpu,
};

pub(super) enum RunEvent {
    Tick,
}

pub enum Event<'a, 'b, 'c, 'd> {
    /// Nox has initialized.
    ///
    /// Gets called once at the beginning before any other events.
    Initialized {
        event_loop: &'a event_loop::ActiveEventLoop<'b>,
        gpu: &'a mut gpu::GpuContext<'b>,
    },
    /// Nox is updating.
    ///
    /// Happens once per frame before any GPU work.
    Update {
        event_loop: &'a event_loop::ActiveEventLoop<'b>,
        gpu: &'a mut gpu::GpuContext<'b>,
    },
    /// Nox is recording compute commands.
    ///
    /// Commands dispatched with `commands` are synchronized to
    /// run before any render work.
    ///
    /// Happens once per frame:
    ComputeWork {
        commands: &'a mut gpu::ComputeCommands<'b, 'c>,
    },
    /// Nox is recording transfer commands.
    ///
    /// Commands dispatched with `commands` are *not*
    /// synchronized by default.
    ///
    /// Synchronization can be achieved using a custom [`TimelineSemaphore`].
    TransferWork {
        request_id: gpu::CommandRequestId,
        commands: &'a mut gpu::TransferCommands<'b, 'c, 'd>,
    },
    /// The frame buffer for window with `window_id` has been (re)created.
    FrameBufferCreated {
        window_id: win::WindowId,
        event_loop: &'a event_loop::ActiveEventLoop<'b>,
        gpu: &'a mut gpu::GpuContext<'b>,
        new_size: gpu::Dimensions,
        new_format: gpu::RawFormat,
    },
    /// Nox is constructing a [`FrameGraph`].
    ///
    /// # Safety
    /// The resources used by any pending transfers *must* not be used while rendering.
    ///
    /// Happens once per frame.
    Render {
        frame_graph: &'a mut gpu::FrameGraph<'b, 'c>,
        pending_transfers: &'a [gpu::CommandRequestId],
    },
    /// Nox is recording render commands.
    ///
    /// Happens for each pass added to the current [`FrameGraph`]
    RenderWork {
        pass_id: gpu::PassId,
        commands: &'a mut gpu::RenderCommands<'b, 'c, 'd>,
    },
    CleanUp {
        gpu: &'a mut gpu::GpuContext<'b>,
    },
}
