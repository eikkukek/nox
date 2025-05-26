#![allow(unused_variables)]

use super::{Nox, InitSettings, Memory, Extent};

pub trait Interface
    where
        Self: Sized
{

    fn create_memory<'a>() -> Memory<'a>;
    fn init_settings(&mut self) -> &InitSettings;
    fn init_callback(&mut self, nox: &mut Nox<Self>) {}
    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: Extent::<u32>, image_count: u32) {}
}
