#![allow(unused_variables)]

pub use winit::dpi::LogicalSize;

use crate::renderer::{
    self,
    GlobalResources,
    CommandRequests,
    CommandRequestID,
    TransferCommandbuffer,
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

    fn init_callback(&mut self, nox: &mut Nox<Self>) {}

    fn update(&mut self, nox: &mut Nox<Self>, renderer_resources: &mut GlobalResources);

    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: LogicalSize<f32>, image_count: u32) {}

    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        pending_transfers: &[CommandRequestID],
    ) -> Result<(), renderer::Error>;

    fn command_requests(&mut self, requests: &mut CommandRequests);

    fn transfer_commands(
        &mut self,
        id: CommandRequestID,
        command_buffer: &mut TransferCommandbuffer,
    );
}
