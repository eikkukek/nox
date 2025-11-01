use compact_str::CompactString;

use nox::{mem::vec_types::{Vector, GlobalVec}, *};

use nox_font::{VertexTextRenderer, CombinedRenderedText, text_segment};

use nox_geom::{
    *,
    shapes::*,
};

use crate::*;


pub struct HoverWindow {
    text: CompactString,
    rendered_text: CombinedRenderedText<BoundedTextInstance, GlobalVec<BoundedTextInstance>>,
    rect: Rect,
    vertices: GlobalVec<Vertex>,
    rect_vertex_range: VertexRange,
    rect_outline_vertex_range: VertexRange,
    indices: GlobalVec<u32>,
    position: Vec2,
    outline_width: f32,
}

impl HoverWindow {

    pub fn new() -> Self {
        Self {
            text: Default::default(),
            rendered_text: Default::default(),
            rect: Default::default(),
            rect_vertex_range: Default::default(),
            rect_outline_vertex_range: Default::default(),
            vertices: Default::default(),
            indices: Default::default(),
            position: Default::default(),
            outline_width: 0.0,
        }
    }

    pub fn update<FontHash>(
        &mut self,
        style: &impl WindowStyle<FontHash>,
        text_renderer: &mut VertexTextRenderer<FontHash>,
        cursor_pos: Vec2,
        text: &str,
    )
        where
            FontHash: UiFontHash,
    {
        let mut rect = self.rect;
        rect.rounding = style.rounding();
        if text != self.text {
            self.rendered_text.clear();
            let rendered_text = text_renderer.render(
                &[text_segment(text, style.font_regular())], false, 0.0
            ).unwrap_or_default();
            self.rendered_text.add_text(
                &rendered_text,
                vec2(0.0, 0.0),
                BoundedTextInstance {
                    add_scale: vec2(1.0, 1.0),
                    min_bounds: vec2(f32::MIN, f32::MIN),
                    max_bounds: vec2(f32::MAX, f32::MAX),
                    color: style.focused_text_col(),
                }
            ).unwrap();
            self.text = CompactString::new(text);
            rect.max = style.calc_text_box_size(&rendered_text);
        }
        let requires_triangulation =
            rect != self.rect ||
            self.outline_width != style.window_outline_width();
        self.outline_width = style.window_outline_width();
        self.rect = rect;
        self.position = cursor_pos + vec2(-self.rect.max.x, style.item_pad_outer().y);
        if requires_triangulation {
            self.triangulate();
        }
    }

    pub fn triangulate(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        let mut points = GlobalVec::new();
        let mut outline_points = GlobalVec::new();
        let mut indices_usize = GlobalVec::new();
        self.rect.to_points(&mut |p| { points.push(p.into()); });
        nox_geom::shapes::outline_points(
            &points, self.outline_width * 0.5, false,
            &mut |p| { outline_points.push(p.into()); }
        );
        earcut::earcut(&outline_points, &[], false, &mut self.vertices, &mut indices_usize).unwrap();
        self.rect_outline_vertex_range = VertexRange::new(0..self.vertices.len());
        let vertex_off = self.vertices.len();
        earcut::earcut(&points, &[], false, &mut self.vertices, &mut indices_usize).unwrap();
        self.rect_vertex_range = VertexRange::new(vertex_off..self.vertices.len());
        self.indices.append_map(&indices_usize, |&v| v as u32);
    }

    pub fn set_vertex_params<FontHash>(
        &mut self,
        style: &impl WindowStyle<FontHash>,
    ) {
        if self.rect_outline_vertex_range.start() >= self.vertices.len() {
            return
        }
        let vertex_sample = self.vertices[self.rect_outline_vertex_range.start()];
        if vertex_sample.color != style.window_outline_col() {
            let target_color = style.window_outline_col();
            set_vertex_params(&mut self.vertices,
                self.rect_outline_vertex_range, vec2(0.0, 0.0), target_color
            );
        }
        let vertex_sample = self.vertices[self.rect_vertex_range.start()];
        if vertex_sample.color != style.hover_window_bg_col() {
            let target_color = style.hover_window_bg_col();
            for vertex in &mut self.vertices[self.rect_vertex_range.range()] {
                vertex.color = target_color;
            }
        }
    }

    pub fn render_commands<FontHash>(
        &self,
        render_commands: &mut RenderCommands,
        style: & impl WindowStyle<FontHash>,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
        unit_scale: f32,
    ) -> Result<(), Error>
    {
        let vert_count = self.vertices.len();
        let vert_mem = unsafe {
            vertex_buffer.allocate(render_commands, vert_count)?
        };
        let idx_count = self.indices.len();
        let idx_mem = unsafe {
            index_buffer.allocate(render_commands, idx_count)?
        };
        unsafe {
            self.vertices
                .as_ptr()
                .copy_to_nonoverlapping(vert_mem.ptr.as_ptr(), vert_count);
            self.indices
                .as_ptr()
                .copy_to_nonoverlapping(idx_mem.ptr.as_ptr(), idx_count);
        }
        render_commands.bind_pipeline(base_pipeline_id)?;
        let pc_vertex = push_constants_vertex(self.position, vec2(1.0, 1.0), inv_aspect_ratio, unit_scale);
        let pc_fragment = base_push_constants_fragment(vec2(f32::MIN, f32::MIN), vec2(f32::MAX, f32::MAX));
        render_commands.push_constants(|pc| unsafe {
            if pc.stage == ShaderStage::Vertex {
                pc_vertex.as_bytes()
            } else {
                pc_fragment.as_bytes()
            }
        })?;
        render_commands.draw_indexed(
            DrawInfo {
                index_count: self.indices.len() as u32,
                ..Default::default()
            },
            [
                DrawBufferInfo::new(vertex_buffer.id(), vert_mem.offset),
            ],
            DrawBufferInfo::new(index_buffer.id(), idx_mem.offset)
        )?;
        render_commands.bind_pipeline(text_pipeline_id)?;
        let pc_vertex = push_constants_vertex(
            self.position + style.item_pad_inner(),
            vec2(style.font_scale(), style.font_scale()),
            inv_aspect_ratio,
            unit_scale,
        );
        render_text(
            render_commands,
            self.rendered_text.iter().map(|(c, (t, b))| (*c, t, b.as_slice())),
            pc_vertex, vertex_buffer, index_buffer
        )?;
        Ok(())
    }
}
