#![allow(unused_variables)]

use crate::*;

pub trait Interface
    where
        Self: Sized
{
    /// Provides the initialization settings for Nox.
    fn init_settings(&self) -> InitSettings;

    /// Gets called once right after start up.
    fn init(
        &mut self,
        win: &mut win::WindowContext,
        gpu: &mut gpu::GpuContext,
    ) -> Result<()> { Ok(()) }
   
    /// Gets called when an [`Event`] is being processed.
    fn event(&mut self, event: Event) -> Result<()>;

    /// Gets called once during app clean up.
    fn clean_up(
        &mut self,
        gpu: &mut gpu::GpuContext,
    ) {}
}
