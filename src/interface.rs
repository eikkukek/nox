#![allow(unused_variables)]

use super::{
    Nox,
    InitSettings,
    Extent,
    renderer::Frame,
    string_types::LargeError,
};

pub trait Interface
    where
        Self: Sized
{
    fn init_settings(&mut self) -> &InitSettings;
    fn init_callback(&mut self, nox: &mut Nox<Self>) {}
    fn surface_update(&mut self, nox: &mut Nox<Self>, surface_size: Extent::<u32>, image_count: u32) {}
    fn render<'f, 'mem, 'r>(&mut self, frame: &'f Frame<'mem, 'r>) -> Result<(), LargeError>
        where
            'mem: 'r,
            'f: 'r;
}
