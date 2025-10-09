use nox::{
    mem::value_as_bytes,
    *,
};

use crate::*;

use nox_geom::*;

#[repr(C)]
#[derive(Clone, Copy, VertexInput)]
pub struct Vertex {
    pub pos: Vec2,
    pub offset: Vec2,
    pub color: ColorRGBA,
}

impl From<[f32; 2]> for Vertex {

    fn from(value: [f32; 2]) -> Self {
        Self {
            pos: value.into(),
            offset: Default::default(),
            color: Default::default(),
        }
    }
}

#[repr(C)]
pub struct PushConstantsVertex{
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
pub struct TextPushConstantsFragment {
    pub color: ColorRGBA,
}

pub fn text_push_constants_fragment(color: ColorRGBA) -> TextPushConstantsFragment {
    TextPushConstantsFragment {
        color,
    }
}

impl TextPushConstantsFragment {

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
    layout(location = 2) in vec4 in_color;

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos.x *= pc.scale.x;
        pos.y *= pc.scale.y;
        pos += pc.vert_off + in_offset;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
        out_color = in_color;
    }
";

pub const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec4 in_color;

    layout(location = 0) out vec4 out_color;

    void main() {
        out_color = in_color;
    }
";

pub const TEXT_VERTEX_SHADER: &'static str = "
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

pub const TEXT_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) vec4 color;
    } pc;

    void main() {
        out_color = pc.color;
    }
";
