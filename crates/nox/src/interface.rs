#![allow(unused_variables)]

use crate::*;

pub trait Interface
    where
        Self: Sized
{
    /// Provides the initialization settings for Nox.
    fn init_settings(&self) -> InitSettings;

    /// Gets called once right after start up.
    fn init_callback(
        &mut self,
        nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error> { Ok(()) }

    /// Gets called every frame before `compute`.
    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
    ) -> Result<(), Error> { Ok(()) }

    /// Gets called when the frame buffer is (re)created.
    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error> { Ok(()) }

    /// Gets called every frame before `render`.
    ///
    /// Commands dispatched in this function are synchronized to run before
    /// commands in `render_commands`.
    ///
    /// # Arguments
    /// `commands`: used to dispatch compute commands run on the GPU
    fn compute(
        &mut self,
        commands: &mut ComputeCommands,
    ) -> Result<(), Error> { Ok(()) }

    /// Gets called at the end of every frame.
    ///
    /// Used to construct the frame graph used in rendering.
    ///
    /// # Arguments
    /// `frame_graph`: the frame graph to be constructed
    /// `pending_transfers`: a slice of all pending transfer commands
    /// 
    /// # Safety
    /// The resources used by any pending transfers *must* not be used by the frame graph.
    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestId],
    ) -> Result<(), Error>;

    /// Gets called every frame before for each requested
    /// transfer command.
    ///
    /// # Arguments
    /// `id`: current transfer request Id
    /// `commands`: used to dispatch transfer commands on the GPU
    fn transfer_commands(
        &mut self,
        id: CommandRequestId,
        commands: &mut TransferCommands,
    ) -> Result<Option<std::thread::JoinHandle<()>>, Error>
    {
        Ok(None)
    }

    /// Gets called after frame graph construction in `render` 
    ///
    /// # Arguments
    /// `pass_id`: current pass Id
    /// `commands`: used to dispatch render commands on the GPU
    fn render_commands(
        &mut self,
        pass_id: PassId,
        commands: &mut RenderCommands,
    ) -> Result<(), Error>;

    /// Gets called once during app clean up.
    fn clean_up(
        &mut self,
        renderer: &mut RendererContext,
    ) {}
}
