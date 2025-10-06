use nox::{
    mem::value_as_bytes,
    *,
};

use crate::*;

use nox_geom::*;

#[repr(C)]
#[derive(VertexInput)]
pub struct Vertex {
    pub pos: Vec2,
}

impl From<[f32; 2]> for Vertex {

    fn from(value: [f32; 2]) -> Self {
        Self {
            pos: value.into(),
        }
    }
}

#[repr(C)]
#[derive(VertexInput)]
pub struct VertexUv {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

#[repr(C)]
pub struct PushConstantsVertex {
    pub vert_off: Vec2,
    pub scale: Vec2,
    pub inv_aspect_ratio: f32,
}

pub fn push_constants_vertex(
    vert_off: Vec2,
    scale: Vec2,
    inv_aspect_ratio: f32,
) -> PushConstantsVertex 
{
    PushConstantsVertex {
        vert_off,
        scale,
        inv_aspect_ratio,
    }
}

impl PushConstantsVertex {

    pub unsafe fn as_bytes(&self) -> &[u8] {
        unsafe {
            value_as_bytes(self).unwrap()
        }
    }
}

#[repr(C)]
pub struct PushConstantsFragment {
    pub color: ColorRGBA,
}

pub fn push_constants_fragment(
    color: ColorRGBA
) -> PushConstantsFragment
{
    PushConstantsFragment {
        color,
    }
}

impl PushConstantsFragment {

    pub unsafe fn as_bytes(&self) -> &[u8] {
        unsafe {
            value_as_bytes(self).unwrap()
        }
    }
}

pub const BASE_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 1) in vec2 in_offset;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
    } pc;

    void main() {
        vec2 pos = in_pos + in_offset;
        pos.x *= pc.scale.x;
        pos.y *= pc.scale.y;
        pos += pc.vert_off;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";

pub const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) vec4 color;
    } pc;

    void main() {
        out_color = pc.color;
    }
";
