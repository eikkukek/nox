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
    pos: [f32; 2],
    bary: [f32; 3],
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, VertexInput)]
pub struct VertexOffset {
    offset: [f32; 2],
}
