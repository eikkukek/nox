use core::hash::Hash;

use nox::{
    mem::vec_types::GlobalVec,
    *,
};

use nox_font::{RenderedText, VertexTextRenderer};

use nox_geom::*;

use crate::*;

#[derive(Default, Clone, Copy)]
pub struct UpdateResult {
    pub requires_triangulation: bool,
    pub cursor_in_widget: bool,
}

#[derive(Clone, Copy)]
pub enum WidgetStatus<'a> {
    Inactive,
    Hovered(Option<&'a str>),
    Active,
}

pub trait UiFontHash: Default + Clone + Eq + Hash {}

impl<T: Default + Clone + Eq + PartialEq + Hash> UiFontHash for T {}

pub trait HoverContents<I, FontHash, Style: WindowStyle<FontHash>>
    where
        I: Interface,
        FontHash: UiFontHash,
{

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
        unit_scale: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error>;
}

#[derive(Default, Clone, Copy, Debug)]
pub struct VertexRange {
    pub start: u32,
    pub end: u32,
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
    pub fn end(self) -> usize {
        self.end as usize
    }

    #[inline(always)]
    pub fn range(self) -> core::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

pub trait Widget<I, FontHash, Style>
    where
        I: Interface,
        FontHash: UiFontHash,
        Style: WindowStyle<FontHash>
{

    fn get_offset(&self) -> Vec2;

    fn set_offset(&mut self, offset: Vec2);

    fn set_scroll_offset(&mut self, offset: Vec2);

    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<FontHash>,
    ) -> Vec2;

    fn status<'a>(
        &'a self,
        nox: &Nox<I>,
        style: &Style,
        window_pos: Vec2,
        cursor_pos: Vec2,
    ) -> WidgetStatus<'a>;

    fn update(
        &mut self,
        nox: &mut Nox<I>,
        style: &Style,
        text_renderer: &mut VertexTextRenderer<'_, FontHash>,
        window_size: Vec2,
        window_pos: Vec2,
        content_offset: Vec2,
        cursor_pos: Vec2,
        delta_cursor_pos: Vec2,
        cursor_in_this_window: bool,
        other_widget_active: bool,
        cursor_in_other_widget: bool,
        window_moving: bool,
        hover_blocked: bool,
        collect_text: &mut dyn FnMut(&RenderedText, Vec2, BoundedTextInstance),
    ) -> UpdateResult;

    fn triangulate(
        &mut self,
        points: &mut GlobalVec<[f32; 2]>,
        helper_points: &mut GlobalVec<[f32; 2]>,
        tri: &mut dyn FnMut(&[[f32; 2]]) -> VertexRange,
    );

    fn set_vertex_params(
        &mut self,
        style: &Style,
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
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, FontHash, Style>>, Error>;

    fn hide(
        &self,
        vertices: &mut [Vertex],
    );
}
