mod face;
mod triangulate;
mod vertex_text_renderer;

pub use face::Face;
pub use triangulate::{triangulate, GlyphTriangles};
pub use vertex_text_renderer::*;

pub use nox::renderer::VertexInput;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, VertexInput)]
pub struct Vertex {
    pub pos: [f32; 2],
}

impl From<[f32; 2]> for Vertex {

    fn from(value: [f32; 2]) -> Self {
        Vertex { pos: value }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, VertexInput)]
pub struct VertexOffset {
    pub offset: [f32; 2],
}
