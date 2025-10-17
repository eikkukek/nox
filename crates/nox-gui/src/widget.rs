use core::hash::Hash;

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

pub trait HoverContents<I, FontHash, HoverStyle: WindowStyle<FontHash>>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
{

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &HoverStyle,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error>;
}

#[derive(Default, Clone, Copy, Debug)]
pub struct VertexRange {
    start: u32,
    end: u32,
}

pub trait Widget<I, FontHash, Style, HoverStyle>
    where
        I: Interface,
        FontHash: Clone + Eq + Hash,
        Style: WindowStyle<FontHash>,
        HoverStyle: WindowStyle<FontHash>
{

    fn hover_text(&self) -> Option<&str>;

    fn set_offset(
        &mut self,
        offset: Vec2,
    );

    fn calc_height(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
    ) -> f32;

    fn is_active(
        &self,
        nox: &Nox<I>,
        style: &Style,
        hover_style: &HoverStyle,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> bool;

    fn update(
        &mut self,
        nox: &Nox<I>,
        style: &Style,
        hover_style: &HoverStyle,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_width: f32,
        window_pos: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        window_moving: bool,
    ) -> UpdateResult;

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    );

    fn set_vertex_params(
        &mut self,
        style: &Style,
        hover_style: &HoverStyle,
        vertices: &mut [Vertex],
    );

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        base_pipeline_id: GraphicsPipelineId,
        text_pipeline_id: GraphicsPipelineId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, HoverStyle>>, Error>;

    fn hide(
        &self,
        vertices: &mut [Vertex],
    );
}

impl VertexRange {

    #[inline(always)]
    pub fn new(range: core::ops::Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }

    #[inline(always)]
    pub fn start(self) -> usize {
        self.start as usize
    }

    #[inline(always)]
    pub fn range(self) -> core::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}
