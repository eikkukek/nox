use nox::{
    mem::{
        Allocator,
    },
    *,
};

use nox_geom::{BoundingRect, Vec2};

use crate::*;

pub struct OnTopContents {
    pub relative_bounds: BoundingRect,
    painter_storage: PainterStorage,
}

impl OnTopContents {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            relative_bounds: Default::default(),
            painter_storage: PainterStorage::new(),
        }
    }

    #[inline(always)]
    pub fn requires_transfer_commands(&self) -> bool {
        self.painter_storage.requires_transfer_commands()
    }
    
    #[inline(always)]
    pub fn begin(&mut self) {
        self.painter_storage.begin();
    }

    #[inline(always)]
    pub fn end(
        &mut self,
        window_semaphore: (TimelineSemaphoreId, u64),
        global_resources: &mut GlobalResources,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        self.painter_storage.end(
            window_semaphore,
            global_resources,
            tmp_alloc
        )?;
        Ok(())
    }

    #[inline(always)]
    pub fn triangulate(&mut self) {
        self.painter_storage.triangulate();
    }

    #[inline(always)]
    pub fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        render_format: ColorFormat,
        add_read: &mut dyn FnMut(ReadInfo),
    ) -> Result<(), Error> {
        self.painter_storage.render(frame_graph, render_format, add_read)?;
        Ok(())
    }

    #[inline(always)]
    pub fn transfer_commands(
        &mut self,
        transfer_commands: &mut TransferCommands,
        window_semaphore: (TimelineSemaphoreId, u64),
        sampler: SamplerId,
        texture_pipeline_layout: PipelineLayoutId,
        tmp_alloc: &impl Allocator,
    ) -> Result<(), Error>
    {
        self.painter_storage.transfer_commands(
            transfer_commands,
            window_semaphore,
            sampler,
            texture_pipeline_layout,
            tmp_alloc
        )?;
        Ok(())
    }

    #[inline(always)]
    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &impl UiStyle,
        sampler: SamplerId,
        offset: Vec2,
        bounds: BoundingRect,
        base_pipeline: GraphicsPipelineId,
        text_pipeline: GraphicsPipelineId,
        texture_pipeline: GraphicsPipelineId,
        texture_pipeline_layout: PipelineLayoutId,
        vertex_buffer: &mut RingBuf,
        index_buffer: &mut RingBuf,
        inv_aspect_ratio: f32,
        unit_scale: f32,
        tmp_alloc: &impl Allocator,
        get_custom_pipeline: &mut dyn FnMut(&str) -> Option<GraphicsPipelineId>,
    ) -> Result<(), Error>
    {
        self.painter_storage.render_commands(
            render_commands, style, sampler,
            offset, bounds, base_pipeline,
            text_pipeline, texture_pipeline,
            texture_pipeline_layout, vertex_buffer,
            index_buffer, inv_aspect_ratio,
            unit_scale, tmp_alloc, get_custom_pipeline
        )?;
        Ok(())
    }
}

pub struct OnTopContentsUiCtx<'a, 'b, Style> {
    contents: &'a mut OnTopContents,
    style: &'a Style,
    text_renderer: &'a mut TextRenderer<'b>,
    image_loader: &'a mut ImageLoader,
}

impl<'a, 'b, Style: UiStyle> OnTopContentsUiCtx<'a, 'b, Style> {

    pub fn painter(&'a mut self) -> Painter<'a> {
        Painter::new(
            &mut self.contents.painter_storage,
            self.style,
            self.text_renderer,
            self.image_loader,
        )
    }
}
