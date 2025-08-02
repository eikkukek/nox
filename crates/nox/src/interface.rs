#![allow(unused_variables)]

pub use winit::dpi::LogicalSize;

use crate::{
    Error,
    renderer::{
        self,
        RendererContext,
        CommandRequestID,
        TransferCommandbuffer,
    }
};

use super::{
    Nox,
    InitSettings,
    renderer::frame_graph::FrameGraphInit,
};

pub trait Interface
    where
        Self: Sized + Send + Sync + 'static
{
    fn init_settings(&self) -> InitSettings;

    fn init_callback(
        &mut self,
        nox: &mut Nox<Self>,
        renderer_context: &mut RendererContext
    ) -> Result<(), Error>;

    fn update(&mut self, nox: &mut Nox<Self>, renderer_contexts: &mut RendererContext);

    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: LogicalSize<f32>, image_count: u32) {}

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestID],
    ) -> Result<(), renderer::Error>;

    fn transfer_commands(
        &mut self,
        id: CommandRequestID,
        command_buffer: &mut TransferCommandbuffer,
    );
}
