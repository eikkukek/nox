use nox::*;

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
    pub inv_aspect_ratio: f32,
}

pub fn push_constants_vertex(
    vert_off: Vec2,
    inv_aspect_ratio: f32,
) ->PushConstantsVertex 
{
    PushConstantsVertex {
        vert_off,
        inv_aspect_ratio,
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

#[repr(C)]
pub(crate) struct TextPushConstantsVertex {
    pub vert_off: Vec2,
    pub inv_aspect_ratio: f32,
    pub font_scale: f32,
}

pub(crate) fn text_push_constants_vertex(
    vert_off: Vec2,
    inv_aspect_ratio: f32,
    font_scale: f32,
) -> TextPushConstantsVertex
{
    TextPushConstantsVertex {
        vert_off,
        inv_aspect_ratio,
        font_scale,
    }
}

pub const BASE_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        float inv_aspect_ratio;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos += pc.vert_off;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";

pub const TEXT_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 1) in vec2 in_off;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        float inv_aspect_ratio;
        float font_scale;
    } pc;

    void main() {
        vec2 pos = in_pos + in_off;
        pos *= pc.font_scale;
        pos += pc.vert_off;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";

pub const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 16) vec4 color;
    } pc;

    void main() {
        out_color = pc.color;
    }
";
