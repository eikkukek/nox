use crate::renderer::{
    *,
    image::ImageRangeInfo,
    global_resources::{ImageId},
    frame_state::ResourceId,
};

use super::*;

pub trait PassAttachmentBuilder<'a> {

    fn with_read(&mut self, read_info: ReadInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_depth_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_depth_stencil_write(&mut self, write: WriteInfo) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_render_area(&mut self, render_area: RenderArea) -> &mut dyn PassAttachmentBuilder<'a>;

    fn with_signal_semaphore(&mut self, id: TimelineSemaphoreId, value: u64);
}

pub trait FrameGraph<'a> {

    fn edit_resources(
        &mut self,
        f: &mut dyn FnMut(&mut GlobalResources) -> Result<(), Error>
    ) -> Result<(), Error>;

    fn frame_index(&self) -> u32;

    fn frame_buffer_size(&self) -> image::Dimensions;

    fn set_render_image(&mut self, id: ResourceId, range_info: Option<ImageRangeInfo>) -> Result<(), Error>;

    fn add_image(&mut self, id: ImageId) -> Result<ResourceId, Error>;

    fn add_transient_image(
        &mut self, 
        f: &mut dyn FnMut(&mut ImageBuilder),
    ) -> Result<ResourceId, Error>;

    fn add_pass(
        &mut self,
        info: PassInfo,
        f: &mut dyn FnMut(&mut dyn PassAttachmentBuilder),
    ) -> Result<PassId, Error>;
}
