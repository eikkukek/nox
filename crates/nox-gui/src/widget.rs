use nox::{
    alloc::arena_alloc::ArenaGuard, mem::{vec_types::GlobalVec}, *
};

use nox_font::RenderedText;

use nox_geom::*;

use crate::*;

#[derive(Default, Clone, Copy)]
pub struct UpdateResult {
    pub requires_triangulation: bool,
    pub requires_transfer_commands: bool,
    pub cursor_in_widget: bool,
}

#[derive(Clone, Copy)]
pub enum WidgetStatus<'a> {
    Inactive,
    Hovered(Option<&'a str>),
    Active,
}

pub trait HoverContents<I, Style: WindowStyle>
    where
        I: Interface,
{

    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        sampler: SamplerId,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &ArenaGuard,
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

pub trait Widget<I, Style>
    where
        I: Interface,
        Style: WindowStyle,
{

    fn get_offset(&self) -> Vec2;

    fn set_offset(&mut self, offset: Vec2);

    fn set_scroll_offset(&mut self, offset: Vec2);

    fn calc_size(
        &mut self,
        style: &Style,
        text_renderer: &mut TextRenderer, 
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
        text_renderer: &mut TextRenderer,
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

    #[allow(unused_variables)]
    fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        msaa_samples: MSAA,
        render_format: ColorFormat,
        resolve_mode: Option<ResolveMode>,
        add_read: &mut dyn FnMut(ReadInfo),
        add_signal_semaphore: &mut dyn FnMut(TimelineSemaphoreId, u64),
        add_pass: &mut dyn FnMut(PassId),
    ) -> Result<(), Error> { Ok(()) }

    #[allow(unused_variables)]
    fn render_commands(
        &self,
        render_commands: &mut RenderCommands,
        style: &Style,
        sampler: SamplerId,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        window_pos: Vec2,
        content_area: BoundingRect,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &ArenaGuard,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<Option<&dyn HoverContents<I, Style>>, Error> { Ok(None) }

    #[allow(unused_variables)]
    fn transfer_commands(
        &mut self,
        transfer_commands: &mut TransferCommands,
        window_semaphore: Option<(TimelineSemaphoreId, u64)>,
        sampler: SamplerId,
        texture_pipeline_layout: PipelineLayoutId,
        tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error> { Ok(()) }

    fn hide(
        &mut self,
        vertices: &mut [Vertex],
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &ArenaGuard,
    ) -> Result<(), Error>;
}
