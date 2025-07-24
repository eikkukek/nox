#![allow(unused_variables)]

pub use winit::dpi::LogicalSize;

use crate::renderer;

use super::{
    Nox,
    InitSettings,
    renderer::{ash::vk, QueueFamilyIndices, frame_graph::FrameGraphInit},
};

pub trait Interface
    where
        Self: Sized
{
    fn init_settings(&mut self) -> &InitSettings;
    fn init_callback(&mut self, nox: &mut Nox<Self>) {}
    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: LogicalSize<f32>, image_count: u32) {}
    fn render<'a>(
        &mut self,
        frame_graph: &'a mut dyn FrameGraphInit,
        render_image_format: vk::Format,
        queue_family_indices: QueueFamilyIndices,
    ) -> Result<(), renderer::Error>;
}
