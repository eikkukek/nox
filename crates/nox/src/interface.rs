#![allow(unused_variables)]

use crate::{
    renderer::{
        self,
        RendererContext,
        CommandRequestID,
        TransferCommandbuffer,
        RenderCommands,
        frame_graph::PassID,
    }
};

use super::{
    Nox,
    InitSettings,
    Error,
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
        renderer: &mut RendererContext,
    ) -> Result<(), Error>;

    fn update(&mut self, nox: &mut Nox<Self>, renderer: &mut RendererContext);

    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: [u32; 2], image_count: u32) {}

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestID],
    ) -> Result<(), renderer::Error>;

    fn render_commands(
        &mut self,
        pass: PassID,
        commands: &mut RenderCommands,
    ) -> Result<(), renderer::Error>;

    fn transfer_commands(
        &mut self,
        id: CommandRequestID,
        command_buffer: &mut TransferCommandbuffer,
    );
}
