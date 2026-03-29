use compact_str::CompactString;

use nox::{
    mem::vec::Vec32,
    error::*,
    gpu,
};
use nox_font::{CombinedRenderedText, text_segment};
use nox_geom::{
    *,
    shapes::*,
};

use crate::*;

#[derive(Default)]
pub struct HoverWindow {
    text: CompactString,
    rendered_text: CombinedRenderedText<BoundedTextInstance>,
    rect: Rect,
    vertices: Vec32<Vertex>,
    rect_vertex_range: Option<VertexRange>,
    rect_stroke_vertex_range: Option<VertexRange>,
    indices: Vec32<u32>,
    position: Vec2,
    stroke_thickness: f32,
}

impl HoverWindow {

    pub fn new() -> Self {
        Self {
            text: Default::default(),
            rendered_text: Default::default(),
            rect: Default::default(),
            rect_vertex_range: None,
            rect_stroke_vertex_range: None,
            vertices: Default::default(),
            indices: Default::default(),
            position: Default::default(),
            stroke_thickness: 0.0,
        }
    }

    pub fn update(
        &mut self,
        rec: &mut RecordCmd<'_>,
        style: &UiStyle,
        cached_data: &CachedUiData,
        text: &str,
    )
    {
        let mut rect = self.rect;
        rect.rounding = style.rounding;
        if text != self.text {
            self.rendered_text.clear();
            let rendered_text = rec.text_renderer().render(
                &[text_segment(text, &style.regular_font)], false, 0.0
            ).unwrap_or_default();
            self.rendered_text.add_text(
                &rendered_text,
                vec2(0.0, 0.0),
                BoundedTextInstance {
                    add_scale: vec2(1.0, 1.0),
                    min_bounds: vec2(f32::MIN, f32::MIN),
                    max_bounds: vec2(f32::MAX, f32::MAX),
                    color: style.focused_text_col,
                }
            );
            self.text = CompactString::new(text);
            rect.max = style.calc_text_box_size(&rendered_text);
        }
        let requires_triangulation =
            rect != self.rect ||
            self.stroke_thickness != style.window_stroke_thickness;
        self.stroke_thickness = style.window_stroke_thickness;
        self.rect = rect;
        self.position = cached_data.cursor_pos + vec2(-self.rect.max.x, style.item_pad_outer.y);
        if requires_triangulation {
            self.triangulate();
        }
    }

    pub fn triangulate(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        let mut points = Vec32::new();
        let mut outline_points = Vec32::new();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points, self.stroke_thickness * 0.5, false,
            &mut |p| { outline_points.push(p.into()); }
        );
        earcut::earcut(&outline_points, &[], false, &mut self.vertices, &mut self.indices);
        self.rect_stroke_vertex_range = VertexRange::new(0..self.vertices.len());
        let vertex_off = self.vertices.len();
        earcut::earcut(&points, &[], false, &mut self.vertices, &mut self.indices);
        self.rect_vertex_range = VertexRange::new(vertex_off..self.vertices.len());
    }

    pub fn set_vertex_params(
        &mut self,
        style: &UiStyle,
    ) {
        color_vertices(&mut self.vertices, self.rect_stroke_vertex_range, style.window_stroke_col);
        color_vertices(&mut self.vertices, self.rect_vertex_range, style.hover_window_bg_col);
    }

    pub fn draw(
        &self,
        cmd: &mut gpu::DrawCommands,
        rec: &mut RecordCmd<'_>,
        style: &UiStyle,
        cached_data: &CachedUiData,
    ) -> Result<()>
    {
        let vert_count = self.vertices.len();
        let vert_mem = rec.allocate_vertices(vert_count)?;
        let idx_count = self.indices.len();
        let idx_mem = rec.allocate_indices(idx_count)?;
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count as usize);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count as usize);
        }
        let (viewport, scissor) = cached_data.viewport_and_scissor();
        let mut pipeline_cmd = cmd.bind_pipeline(
            rec.base_pipeline(),
            &[viewport], &[scissor],
        )?;
        let pc_vertex = push_constants_vertex(
            self.position,
            vec2(1.0, 1.0),
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale,
        );
        let pc_fragment = base_push_constants_fragment(
            vec2(f32::MIN, f32::MIN),
            vec2(f32::MAX, f32::MAX),
        );
        pipeline_cmd.push_constants(pc_vertex.0, &[pc_vertex.1])?;
        pipeline_cmd.push_constants(pc_fragment.0, &[pc_fragment.1])?;
        pipeline_cmd.begin_drawing_indexed(
            gpu::IndexedDrawInfo::default()
                .index_count(idx_count),
            gpu::IndexBufferInfo::new(rec.index_buffer_id(), idx_mem.offset),
            &[gpu::DrawBufferRange::new(
                rec.vertex_buffer_id(),
                vert_mem.offset,
                vert_mem.size,
            )], None,
            |cmd| {
                cmd.draw_indexed()?;
                Ok(())
            }
        )?;
        let mut pipeline_cmd = cmd.bind_pipeline(rec.text_pipeline(), &[viewport], &[scissor])?;
        let pc_vertex = push_constants_vertex(
            self.position + style.item_pad_inner,
            vec2(style.font_scale, style.font_scale),
            cached_data.inv_aspect_ratio,
            cached_data.unit_scale,
        );
        render_text(
            &mut pipeline_cmd,
            rec, 
            self.rendered_text.iter(),
            pc_vertex.1,
        )?;
        Ok(())
    }
}
