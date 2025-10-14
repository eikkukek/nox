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
    pub color: ColorSRGBA,
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
#[derive(Clone, Copy, VertexInput)]
pub struct ColorPickerVertex {
    pub pos: Vec2,
}

impl From<[f32; 2]> for ColorPickerVertex {

    fn from(value: [f32; 2]) -> Self {
        Self {
            pos: value.into(),
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
    pub color: ColorSRGBA,
}

pub fn text_push_constants_fragment(color: ColorSRGBA) -> TextPushConstantsFragment {
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

#[repr(C)]
pub struct HuePickerPushConstantsFragment {
    pub sat: f32,
    pub val: f32,
}

pub fn hue_picker_push_constants_fragment(sat: f32, val: f32) -> HuePickerPushConstantsFragment {
    HuePickerPushConstantsFragment {
        sat,
        val,
    }
}

impl HuePickerPushConstantsFragment {

    pub unsafe fn as_bytes(&self) -> &[u8] {
        unsafe {
            value_as_bytes(self).unwrap()
        }
    }
}

#[repr(C)]
pub struct ColorPickerPushConstantsFragment {
    pub hue: f32,
}

pub fn color_picker_push_constants_fragments(hue: f32) -> ColorPickerPushConstantsFragment {
    ColorPickerPushConstantsFragment {
        hue,
    }
}

impl ColorPickerPushConstantsFragment {

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

pub const COLOR_PICKER_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 0) out vec2 out_pos;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos.x *= pc.scale.x;
        pos.y *= pc.scale.y;
        pos += pc.vert_off;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
        out_pos = in_pos;
    }
";

pub const COLOR_PICKER_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) float hue;
    } pc;

    const float FRAC_PI_3 = 3.14159265358979323846 / 3.0;

    const float EXP = 1.0f / 2.4f;

    float map_rgb(float n, float hue, float sat, float val) {
        float k = mod(n + hue / FRAC_PI_3, 6.0);
        float ch = val - val * sat * max(min(min(k, 4.0f - k), 1.0), 0.0);
        return pow((ch + 0.055) / 1.055, 2.4);
    }

    vec3 hsv_to_srgb(vec3 c)  {
        return vec3(
            map_rgb(5.0, c.x, c.y, c.z),
            map_rgb(3.0, c.x, c.y, c.z),
            map_rgb(1.0, c.x, c.y, c.z)
        );
    }

    void main() {
        const vec3 white = vec3(0.0, 0.0, 1.0);
        const vec3 black = vec3(0.0);
        vec3 color_hsv = vec3(
            pc.hue,
            mix(0.0, 1.0, in_pos.x),
            mix(1.0, 0.0, in_pos.y)
        );
        out_color = vec4(hsv_to_srgb(color_hsv), 1.0);
    }
";

pub const COLOR_PICKER_FRAGMENT_SHADER_HUE: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) float sat;
        float val;
    } pc;

    const float TAU = 3.14159265358979323846 * 2.0;
    const float FRAC_PI_3 = 3.14159265358979323846 / 3.0;

    float map_rgb(float n, float hue, float sat, float val) {
        float k = mod(n + hue / FRAC_PI_3, 6.0f);
        return val - val * sat * max(min(min(k, 4.0f - k), 1.0), 0.0);
    }

    vec3 hsv_to_srgb(vec3 c)  {
        return vec3(
            map_rgb(5.0f, c.x, c.y, c.z),
            map_rgb(3.0f, c.x, c.y, c.z),
            map_rgb(1.0f, c.x, c.y, c.z)
        );
    }


    void main() {
        float hue = TAU * in_pos.x;
        vec3 color_hsv = vec3(hue, pc.sat, pc.val);
        out_color = vec4(hsv_to_srgb(color_hsv), 1.0);
    }
";
