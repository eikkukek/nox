use nox::{gpu, error::*};
use nox_geom::*;
use nox_font::InstancedText;

use crate::*;

pub fn render_text<'a>(
    cmd: &mut gpu::DrawPipelineCommands,
    rec: &mut RecordCmd<'_>,
    text: impl IntoIterator<Item = (char, &'a InstancedText, &'a [BoundedTextInstance])>,
    pc_vertex: PushConstantsVertex,
) -> Result<()>
{
    cmd.push_constants(0, &[pc_vertex])?;
    let vertex_buffer_id = rec.vertex_buffer_id();
    let index_buffer_id = rec.index_buffer_id();
    for (_ , text, bounded_instances) in text {
        let vert_count = text.trigs.vertices.len();
        let instance_count = text.offsets.len() as u32;
        let idx_count = text.trigs.indices.len();
        let vert_mem = rec.allocate_vertices(vert_count)?;
        let vert_off_mem = rec.allocate_vertices(instance_count)?;
        let instance_mem = rec.allocate_vertices(instance_count)?;
        let idx_mem = rec.allocate_indices(idx_count)?;
        unsafe {
            text.trigs.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count as usize);
            text.offsets
                .as_ptr()
                .copy_to_nonoverlapping(vert_off_mem.ptr.as_ptr(), instance_count as usize);
            debug_assert!(bounded_instances.len() == instance_count as usize);
            bounded_instances
                .as_ptr()
                .copy_to_nonoverlapping(instance_mem.ptr.as_ptr(), instance_count as usize);
            text.trigs.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count as usize);
        }
        cmd.begin_drawing_indexed(
            gpu::IndexedDrawInfo
                ::default()
                .index_count(idx_count)
                .instance_count(instance_count),
            gpu::IndexBufferInfo::new(index_buffer_id, idx_mem.offset),
            &[
                gpu::DrawBufferRange::new(vertex_buffer_id, vert_mem.offset, vert_mem.size),
                gpu::DrawBufferRange::new(vertex_buffer_id, vert_off_mem.offset, vert_off_mem.size),
                gpu::DrawBufferRange::new(vertex_buffer_id, instance_mem.offset, instance_mem.size),
            ], None,
            |cmd| { cmd.draw_indexed()?; Ok(()) }
        ).context("failed to draw text")?;
    }
    Ok(())
}

#[inline(always)]
pub fn set_vertex_params(
    vertices: &mut [Vertex],
    range: Option<VertexRange>,
    offset: Vec2,
    target_color: ColorSRGBA,
) {
    if let Some(range) = range {
        let vertex_sample = vertices[range.start()];
        if vertex_sample.offset != offset || vertex_sample.color != target_color {
            for vertex in &mut vertices[range.range()] {
                vertex.offset = offset;
                vertex.color = target_color;
            }
        }
    }
}

#[inline(always)]
pub fn add_offset_vertices(
    vertices: &mut [Vertex],
    range: Option<VertexRange>,
    offset: Vec2,
) {
    if let Some(range) = range {
        for vertex in &mut vertices[range.range()] {
            vertex.offset += offset;
        }
    }
}

#[inline(always)]
pub fn color_vertices(
    vertices: &mut [Vertex],
    range: Option<VertexRange>,
    target_color: ColorSRGBA,
) {
    if let Some(range) = range {
        let vertex_sample = vertices[range.start()];
        if vertex_sample.color != target_color {
            for vertex in &mut vertices[range.range()] {
                vertex.color = target_color;
            }
        }
    }
}

#[inline(always)]
pub fn hide_vertices(
    vertices: &mut [Vertex],
    range: Option<VertexRange>,
) {
    if let Some(range) = range {
        let vertex_sample = vertices[range.start()];
        if vertex_sample.color.alpha != 0.0 {
            for vertex in &mut vertices[range.range()] {
                vertex.color = ColorSRGBA::black(0.0);
            }
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
pub fn off_to_norm_off(off: Vec2, unit_scale: f32, aspect_ratio: f32) -> Vec2 {
    let mut norm_off = off * unit_scale;
    norm_off.x /= aspect_ratio;
    norm_off * 0.5
}

#[inline(always)]
pub fn norm_off_to_off(norm_off: Vec2, unit_scale: f32, aspect_ratio: f32) -> Vec2 {
    let mut off = norm_off;
    off *= 2.0;
    off.x *= aspect_ratio;
    off / unit_scale
}

#[inline(always)]
pub fn calc_texture_push_constants_vertex(
    pos: Vec2,
    size: Vec2,
    inv_aspect_ratio: f32,
    unit_scale: f32,
) -> (u32, PushConstantsVertex) {
    push_constants_vertex(pos, size, inv_aspect_ratio, unit_scale)
}

#[inline(always)]
pub fn calc_bounds(
    window_pos: Vec2, content_off: Vec2,
    widget_offset: Vec2, window_size: Vec2
) -> (Vec2, Vec2)
{
    let offset = widget_offset.max(content_off);
    let min_bounds = window_pos + offset.min(window_size);
    let max_bounds = min_bounds + window_size - offset;
    (min_bounds, max_bounds)
}

#[inline(always)]
pub fn load_rgba_image(path: &str) -> ::image::ImageResult<::image::ImageBuffer<::image::Rgba<u8>, Vec<u8>>> {
    let image = ::image::ImageReader::open(path)?.decode()?;
    Ok(image.to_rgba8())
}
