use nox::*;

use nox_geom::*;

use nox_font::InstancedText;

use crate::*;

pub fn render_text<'a>(
    render_commands: &mut RenderCommands,
    text: impl IntoIterator<Item = (char, &'a InstancedText, &'a [BoundedTextInstance])>,
    pc_vertex: PushConstantsVertex,
    vertex_buffer: &mut RingBuf,
    index_buffer: &mut RingBuf,
) -> Result<(), Error>
{
    render_commands.push_constants(|_| unsafe {
        pc_vertex.as_bytes()
    })?;
    let vertex_buffer_id = vertex_buffer.id();
    let index_buffer_id = index_buffer.id();
    for (_ , text, bounded_instances) in text {
        let vert_count = text.trigs.vertices.len();
        let instance_count = text.offsets.len();
        let idx_count = text.trigs.indices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_count)?
        };
        let vert_off_mem = unsafe {
            vertex_buffer.allocate(render_commands, instance_count)?
        };
        let instance_mem = unsafe {
            vertex_buffer.allocate(render_commands, instance_count)?
        };
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_count)?
        };
        unsafe {
            text.trigs.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count);
            text.offsets
                .as_ptr()
                .copy_to_nonoverlapping(vert_off_mem.ptr.as_ptr(), instance_count);
            debug_assert!(bounded_instances.len() == instance_count);
            bounded_instances
                .as_ptr()
                .copy_to_nonoverlapping(instance_mem.ptr.as_ptr(), instance_count);
            text.trigs.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count);
        }
        render_commands.draw_indexed(
            DrawInfo {
                index_count: idx_count as u32,
                instance_count: instance_count as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer_id, vert_mem.offset),
                DrawBufferInfo::new(vertex_buffer_id, vert_off_mem.offset),
                DrawBufferInfo::new(vertex_buffer_id, instance_mem.offset),
            ],
            DrawBufferInfo::new(index_buffer_id, idx_mem.offset),
        )?;
    }
    Ok(())
}

#[inline(always)]
pub fn set_vertex_params(
    vertices: &mut [Vertex],
    range: VertexRange,
    offset: Vec2,
    target_color: ColorSRGBA,
) {
    let vertex_sample = vertices[range.start()];
    if vertex_sample.offset != offset || vertex_sample.color != target_color {
        for vertex in &mut vertices[range.range()] {
            vertex.offset = offset;
            vertex.color = target_color;
        }
    }
}

#[inline(always)]
pub fn hide_vertices(
    vertices: &mut [Vertex],
    range: VertexRange,
) {
    let vertex_sample = vertices[range.start()];
    if vertex_sample.color.alpha != 0.0 {
        for vertex in &mut vertices[range.range()] {
            vertex.color = ColorSRGBA::black(0.0);
        }
    }
}

#[inline(always)]
pub fn pos_to_norm_pos(pos: Vec2, unit_scale: f32, aspect_ratio: f32) -> Vec2 {
    // pos = (2.0 * orig_pos - 1.0) * aspect_ratio.x / unit_scale   | * unit scale
    // pos * unit_scale = (2.0 * orig_pos - 1.0) * aspect_ratio.x   | / aspect_ratio.x
    // pos * unit_scale / aspect_ratio.x = 2.0 * orig_pos - 1.0     | + 1.0 
    // pos * unit_scale / aspect_ratio.x + 1.0 = orig_pos * 2.0     | / 2.0     
    // orig_pos = (pos * unit_scale / aspect_ratio.x + 1.0) / 2.0
    let mut norm_pos = pos * unit_scale;
    norm_pos.x /= aspect_ratio;
    (norm_pos + vec2(1.0, 1.0)) * 0.5
}

#[inline(always)]
pub fn norm_pos_to_pos(norm_pos: Vec2, unit_scale: f32, aspect_ratio: f32) -> Vec2 {
    let mut pos = norm_pos;
    pos *= 2.0;
    pos -= vec2(1.0, 1.0);
    pos.x *= aspect_ratio;
    pos / unit_scale
}

#[inline(always)]
pub fn calc_bounds(window_pos: Vec2, widget_offset: Vec2, window_size: Vec2) -> (Vec2, Vec2) {
    let min_bounds = window_pos + widget_offset.min(window_size);
    let max_bounds = min_bounds + window_size - widget_offset;
    (min_bounds, max_bounds)
}
