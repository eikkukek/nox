#![allow(unused_variables)]

pub use winit::dpi::LogicalSize;

use nox_mem::Allocator;

use super::{
    Nox,
    InitSettings,
    renderer::{FrameGraphInit, RenderError},
};

pub trait Interface
    where
        Self: Sized
{
    fn init_settings(&mut self) -> &InitSettings;
    fn init_callback(&mut self, nox: &mut Nox<Self>) {}
    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: LogicalSize<f32>, image_count: u32) {}
    fn render<'alloc, Alloc: Allocator>(&mut self, frame_graph: FrameGraphInit<'alloc, Alloc>) -> Result<(), RenderError>;
}
