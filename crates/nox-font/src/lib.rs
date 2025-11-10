mod face;
mod triangulate;
mod vertex_text_renderer;

pub use face::Face;
pub use triangulate::{triangulate, GlyphTriangles};
pub use vertex_text_renderer::*;

pub use nox::VertexInput;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, VertexInput)]
pub struct Vertex {
    pub pos: [f32; 2],
}

impl From<[f32; 2]> for Vertex {

    fn from(value: [f32; 2]) -> Self {
        Self { pos: value }
    }
}

impl From<nox_geom::Vec2> for Vertex {

    fn from(value: nox_geom::Vec2) -> Self {
        Self { pos: value.into() }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, VertexInput)]
pub struct VertexOffset {
    pub offset: [f32; 2],
}

impl From<[f32; 2]> for VertexOffset {

    fn from(value: [f32; 2]) -> Self {
        Self { offset: value }
    }
}

impl From<nox_geom::Vec2> for VertexOffset {

    fn from(value: nox_geom::Vec2) -> Self {
        Self { offset: value.into() }
    }
}
