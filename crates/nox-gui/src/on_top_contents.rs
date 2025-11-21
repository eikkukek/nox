use nox::{
    mem::{
        vec_types::GlobalVec,
        slot_map::GlobalSlotMap,
        Allocator,
    },
    *,
};

use nox_geom::{BoundingRect, Vec2};

use crate::*;

pub struct OnTopContents {
    bounds: BoundingRect,
    painter_storage: PainterStorage,
    children: GlobalSlotMap<OnTopContents>,
}

impl OnTopContents {

    #[inline(always)]
    pub fn new() -> Self {
        Self {
            bounds: Default::default(),
            painter_storage: PainterStorage::new(),
            children: Default::default(),
        }
    }

    #[inline(always)]

    pub fn begin(&mut self) {
        self.painter_storage.begin();
        for (_, child) in &mut self.children {
            child.begin();
        }
    }

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
        for (_, child) in &mut self.children {
            child.end(window_semaphore, global_resources, tmp_alloc)?;
        }
        Ok(())
    }

    pub fn triangulate(&mut self) {
        self.painter_storage.triangulate();
        for (_, child) in &mut self.children {
            child.triangulate();
        }
    }

    pub fn render(
        &mut self,
        frame_graph: &mut dyn FrameGraph,
        render_format: ColorFormat,
        add_read: &mut dyn FnMut(ReadInfo),
    ) -> Result<(), Error> {
        self.painter_storage.render(frame_graph, render_format, add_read)?;
        for (_, child) in &mut self.children {
            child.render(frame_graph, render_format, add_read)?;
        }
        Ok(())
    }

    #[allow(unused_variables)]
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
        for (_, child) in &mut self.children {
            child.transfer_commands(
                transfer_commands,
                window_semaphore,
                sampler,
                texture_pipeline_layout,
                tmp_alloc
            )?;
        }
        Ok(())
    }

    pub fn render_commands(
        &mut self,
        render_commands: &mut RenderCommands,
        style: &impl WindowStyle,
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
        for (_, child) in &mut self.children {
            child.render_commands(
                render_commands, style,
                sampler, offset, bounds,
                base_pipeline, text_pipeline, texture_pipeline,
                texture_pipeline_layout, vertex_buffer,
                index_buffer, inv_aspect_ratio,
                unit_scale, tmp_alloc, get_custom_pipeline
            )?;
        }
        Ok(())
    }
}
