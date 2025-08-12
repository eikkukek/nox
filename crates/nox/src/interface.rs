#![allow(unused_variables)]

use crate::{
    renderer::{
        frame_graph::*,
        *,
    },
};

use super::{
    Nox,
    InitSettings,
    Error
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

    fn frame_buffer_size_callback(
        &mut self,
        renderer: &mut RendererContext
    ) -> Result<(), Error>;

    fn update(
        &mut self,
        nox: &mut Nox<Self>,
        renderer: &mut RendererContext,
        frame_buffer_size: (u32, u32),
    );

    fn compute(
        &mut self,
        commands: &mut ComputeCommands,
    ) -> Result<(), Error>;

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestID],
    ) -> Result<(), Error>;

    fn transfer_commands(
        &mut self,
        id: CommandRequestID,
        commands: &mut TransferCommands,
    ) -> Result<(), Error>;

    fn render_commands(
        &mut self,
        pass: PassID,
        commands: &mut RenderCommands,
    ) -> Result<(), Error>;

    fn clean_up(
        &mut self,
        renderer: &mut RendererContext,
    );
}
