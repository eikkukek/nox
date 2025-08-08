use crate::renderer::{
    *,
    image::{ImageRangeInfo, ImageBuilder},
    global_resources::{ImageSourceID},
    frame_state::ResourceID,
};

use super::*;

pub trait PassAttachmentBuilder<'a> {

    fn with_read(&mut self, read_info: ReadInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_render_area(&mut self, render_area: RenderArea) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_dependency(&mut self, pass_id: PassID) -> &mut dyn PassAttachmentBuilder<'a>;
}

pub trait FrameGraph<'a> {

    fn frame_index(&self) -> u32;

    fn frame_buffer_size(&self) -> image::Dimensions;

    fn set_render_image(&mut self, id: ResourceID);

    fn add_image(&mut self, id: ImageSourceID) -> Result<ResourceID, Error>;

    fn add_transient_image(
        &mut self, 
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceID, Error>;

    fn add_transient_image_subresource(
        &mut self,
        resource_id: ResourceID,
        range_info: ImageRangeInfo,
    ) -> Result<ResourceID, Error>;

    fn add_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder),
    ) -> Result<PassID, Error>;
}

pub trait FrameGraphInit<'a> {
    
    fn init(&mut self, max_passes: u32) -> Result<&mut dyn FrameGraph<'a>, Error>;
}
