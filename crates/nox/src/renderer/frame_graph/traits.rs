use crate::renderer::{
    image::{ImageRangeInfo, ImageBuilder},
    global_resources::{ImageSourceID},
    pipeline::PipelineID,
    frame_state::ResourceID,
    Error,
};

use super::{
    PassInfo,
    WriteInfo, ReadInfo,
    RenderArea,
};

pub trait PassPipelineBuilder<'a> {

    fn with_pipeline(&mut self, id: PipelineID) -> &mut dyn PassPipelineBuilder<'a>;
}

pub trait PassAttachmentBuilder<'a> {

    fn with_read(&mut self, read_info: ReadInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_render_area(&mut self, render_area: RenderArea) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_dependency(&mut self, pass_index: usize) -> &mut dyn PassAttachmentBuilder<'a>;

    fn as_pipeline_builder(&mut self) -> &mut dyn PassPipelineBuilder<'a>;
}

pub trait FrameGraph<'a> {

    fn frame_index(&self) -> u32;

    fn set_render_image(&mut self, id: ResourceID);

    fn add_image(&mut self, id: ImageSourceID) -> ResourceID;

    fn add_transient_image(
        &mut self, 
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceID, Error>;

    fn add_transient_image_subresource(
        &mut self,
        resource_id: ResourceID,
        range_info: ImageRangeInfo,
    ) -> Result<ResourceID, Error>;

    fn with_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder),
    ) -> Result<&mut dyn FrameGraph<'a>, Error>;
}

pub trait FrameGraphInit<'a> {
    
    fn init(&mut self, max_passes: u32) -> Result<&mut dyn FrameGraph<'a>, Error>;
}
