use nox::{
    mem::value_as_bytes,
    *,
};

use crate::*;

use nox_geom::{
    *
};

#[repr(C)]
#[derive(Default, Clone, Copy, VertexInput)]
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
#[derive(Default, Clone, Copy, VertexInput, PartialEq)]
pub struct BoundedTextInstance {
    pub add_scale: Vec2,
    pub min_bounds: Vec2,
    pub max_bounds: Vec2,
    pub color: ColorSRGBA,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PushConstantsVertex{
    pub vert_off: Vec2,
    pub scale: Vec2,
    pub inv_aspect_ratio: f32,
    pub unit_scale: f32,
}

pub fn push_constants_vertex(
    vert_off: Vec2,
    scale: Vec2,
    inv_aspect_ratio: f32,
    unit_scale: f32,
) -> PushConstantsVertex 
{
    PushConstantsVertex {
        vert_off,
        scale,
        inv_aspect_ratio,
        unit_scale,
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
#[derive(Clone, Copy)]
pub struct BasePushConstantsFragment{
    pub min_bounds: Vec2,
    pub max_bounds: Vec2,
}

pub fn base_push_constants_fragment(
    min_bounds: Vec2,
    max_bounds: Vec2,
) -> BasePushConstantsFragment
{
    BasePushConstantsFragment {
        min_bounds,
        max_bounds,
    }
}

impl BasePushConstantsFragment {

    pub unsafe fn as_bytes(&self) -> &[u8] {
        unsafe {
            value_as_bytes(self).unwrap()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AlphaPickerPushConstantsFragment {
    pub hue: f32,
    pub sat: f32,
    pub val: f32,
    pub picker_width: f32,
    pub tile_width: f32,
}

pub fn aplha_picker_push_constants_fragment(
    color: ColorHSVA,
    picker_width: f32,
    tile_width: f32,
) -> AlphaPickerPushConstantsFragment
{
    AlphaPickerPushConstantsFragment {
        hue: color.hue,
        sat: color.sat,
        val: color.val,
        picker_width,
        tile_width,
    }
}

impl AlphaPickerPushConstantsFragment {

    pub unsafe fn as_bytes(&self) -> &[u8] {
        unsafe {
            value_as_bytes(self).unwrap()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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

    layout(location = 0) out vec2 out_pos;
    layout(location = 1) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
        float unit_scale;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos.x *= pc.scale.x;
        pos.y *= pc.scale.y;
        out_pos = pos + pc.vert_off + in_offset;
        pos *= pc.unit_scale;
        pos += (pc.vert_off + in_offset) * pc.unit_scale;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
        out_color = in_color;
    }
";

pub const BASE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;
    layout(location = 1) in vec4 in_color;

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) vec2 min_bounds;
        vec2 max_bounds;
    } pc;

    bool in_rect() {
        return pc.min_bounds.x < in_pos.x && pc.max_bounds.x > in_pos.x &&
            pc.min_bounds.y < in_pos.y && pc.max_bounds.y > in_pos.y;
    }

    void main() {
        if (in_rect()) {
            out_color = in_color;
        } else {
            out_color = vec4(0.0);
        }
    }
";

pub const TEXTURE_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) out vec2 out_uv;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
        float unit_scale;
    } pc;

    vec2 positions[6] = vec2[](
        vec2(1.0, 1.0),
        vec2(-1.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(1.0, 1.0),
        vec2(-1.0, -1.0)

    );

    vec2 uvs[6] = vec2[](
        vec2(1.0, 1.0),
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0)
    );

    void main() {
        int vertex_index = gl_VertexIndex;
        vec2 pos = positions[vertex_index];
        //pos.x *= pc.scale.x;
        //pos.y *= pc.scale.y;
        //pos *= pc.unit_scale;
        //pos += pc.vert_off;
        //pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
        out_uv = uvs[vertex_index];
    }
";

pub const TEXTURE_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_uv;

    layout(location = 0) out vec4 out_color;

    layout(set = 0, binding = 0) uniform sampler2D render_image;

    void main() {
        out_color = texture(render_image, in_uv);
    }
";

pub const TEXT_VERTEX_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 1) in vec2 in_offset;

    layout(location = 2) in vec2 in_add_scale;
    layout(location = 3) in vec2 in_min_bounds;
    layout(location = 4) in vec2 in_max_bounds;
    layout(location = 5) in vec4 in_color;

    layout(location = 0) out vec2 out_pos;
    layout(location = 1) out flat vec2 out_min_bounds;
    layout(location = 2) out flat vec2 out_max_bounds;
    layout(location = 3) out flat vec4 out_color;

    layout(push_constant) uniform PushConstant {
        vec2 vert_off;
        vec2 scale;
        float inv_aspect_ratio;
        float unit_scale;
    } pc;

    void main() {
        vec2 pos = in_pos + in_offset;
        pos.x *= pc.scale.x * in_add_scale.x;
        pos.y *= pc.scale.y * in_add_scale.y;
        out_pos = pos + pc.vert_off;
        out_min_bounds = in_min_bounds;
        out_max_bounds = in_max_bounds;
        out_color = in_color;
        pos *= pc.unit_scale;
        pos += pc.vert_off * pc.unit_scale;
        pos.x *= pc.inv_aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";

pub const TEXT_FRAGMENT_SHADER: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;
    layout(location = 1) in vec2 in_min_bounds;
    layout(location = 2) in vec2 in_max_bounds;
    layout(location = 3) in vec4 in_color;

    layout(location = 0) out vec4 out_color;

    bool in_rect() {
        return in_min_bounds.x < in_pos.x && in_max_bounds.x > in_pos.x &&
            in_min_bounds.y < in_pos.y && in_max_bounds.y > in_pos.y;
    }

    void main() {
        if (in_rect()) {
            out_color = in_color;
        } else {
            out_color = vec4(0.0);
        }
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
        float unit_scale;
    } pc;

    void main() {
        vec2 pos = in_pos;
        pos.x *= pc.scale.x;
        pos.y *= pc.scale.y;
        pos *= pc.unit_scale;
        pos += pc.vert_off * pc.unit_scale;
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

    float map_rgb(float n, float hue, float sat, float val) {
        float k = mod(n + hue / FRAC_PI_3, 6.0f);
        float ch = val - val * sat * max(min(min(k, 4.0f - k), 1.0f), 0.0f);
        return pow((ch + 0.055f) / 1.055f, 2.4f);
    }

    vec3 hsv_to_srgb(vec3 c)  {
        return vec3(
            map_rgb(5.0f, c.x, c.y, c.z),
            map_rgb(3.0f, c.x, c.y, c.z),
            map_rgb(1.0f, c.x, c.y, c.z)
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

    const float TAU = 3.14159265358979323846 * 2.0;
    const float FRAC_PI_3 = 3.14159265358979323846 / 3.0;

    float map_rgb(float n, float hue, float sat, float val) {
        float k = mod(n + hue / FRAC_PI_3, 6.0f);
        float ch = val - val * sat * max(min(min(k, 4.0f - k), 1.0f), 0.0f);
        return pow((ch + 0.055f) / 1.055f, 2.4f);
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
        vec3 color_hsv = vec3(hue, 1.0, 1.0);
        out_color = vec4(hsv_to_srgb(color_hsv), 1.0);
    }
";

pub const COLOR_PICKER_FRAGMENT_SHADER_ALPHA: &'static str = "
    #version 450

    layout(location = 0) in vec2 in_pos;

    layout(location = 0) out vec4 out_color;

    layout(push_constant) uniform PushConstant {
        layout(offset = 32) float hue;
        float sat;
        float val;
        float picker_width;
        float tile_width;
    } pc;

    const float FRAC_PI_3 = 3.14159265358979323846 / 3.0;

    float map_rgb(float n, float hue, float sat, float val) {
        float k = mod(n + hue / FRAC_PI_3, 6.0f);
        float ch = val - val * sat * max(min(min(k, 4.0f - k), 1.0f), 0.0f);
        return pow((ch + 0.055f) / 1.055f, 2.4f);
    }

    vec3 hsv_to_srgb(vec3 c)  {
        return vec3(
            map_rgb(5.0f, c.x, c.y, c.z),
            map_rgb(3.0f, c.x, c.y, c.z),
            map_rgb(1.0f, c.x, c.y, c.z)
        );
    }

    vec3 bg_col() {
        const vec3 bg1 = vec3(0.039f);
        const vec3 bg2 = vec3(0.31f);
        float m = mod(pc.picker_width * (in_pos.x + 0.01), pc.tile_width * 2.0);
        float frac = pc.tile_width;
        if (in_pos.y >= 0.5f) {
            if (m >= frac) {
                return bg1;
            }
            return bg2;
        }
        if (m >= frac) {
            return bg2;
        }
        return bg1;
    }

    void main() {
        vec3 color_hsv = vec3(
            pc.hue,
            pc.sat,
            pc.val
        );
        float alpha = in_pos.x;
        vec3 bg = bg_col();
        vec3 color = hsv_to_srgb(color_hsv);
        color = color * alpha + bg * (1.0 - alpha);
        out_color = vec4(color, 1.0);
    }
";
