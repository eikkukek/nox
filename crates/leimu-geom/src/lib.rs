mod vec2;
mod bounding_box;

pub mod fn_2d;
pub mod earcut;
pub mod bezier;
pub mod shapes;

pub use vec2::*;
pub use bounding_box::*;

pub use nox_mem as mem;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}
