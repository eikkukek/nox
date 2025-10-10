use std::hash::Hash;

use nox::{
    mem::vec_types::GlobalVec,
    *,
};

use nox_font::VertexTextRenderer;

use nox_geom::Vec2;

use crate::*;

pub struct UpdateResult {
    pub min_widget_width: f32,
    pub requires_triangulation: bool,
    pub cursor_in_widget: bool,
}

pub type VertexRange = core::ops::Range<usize>;

pub trait Widget<I, FontHash>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    fn hover_text(&self) -> Option<&str>;

    fn set_offset(
        &mut self,
        offset: Vec2,
    );

    fn calc_size(
        &mut self,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> Vec2;

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style<FontHash>,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        cursor_in_this_window: bool,
    ) -> UpdateResult;

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    );

    fn set_vertex_params(
        &mut self,
        style: &Style<FontHash>,
        vertices: &mut [Vertex],
    );

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style<FontHash>,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
    ) -> Result<(), Error>;

    fn hide(
        &self,
        vertices: &mut [Vertex],
    );
}
