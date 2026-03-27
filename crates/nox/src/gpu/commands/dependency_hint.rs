use nox_ash::{vk, ash_style_enum};

use crate::gpu::prelude::*;

ash_style_enum! {
    /// These flags describes where [`command`][1] dependencies are waited on. Specifically, what the
    /// wait stage mask will be for the wait semaphore signaled by the dependency.
    ///
    /// [1]: Commands
    #[flags(Flags64)]
    pub enum MemoryDependencyHint {
        /// Setting this flag means that the wait stage will be set to the earliest possible value.
        ///
        /// The default value of [`MemoryDependencyHint`].
        #[display("none")]
        NONE = 0,
        /// The stage where vertex and index buffers are consumed.
        #[display("vertex input")]
        VERTEX_INPUT = vk::PipelineStageFlags2::VERTEX_INPUT.as_raw(),
        /// The stage where vertex shaders execute.
        #[display("vertex shader")]
        VERTEX_SHADER = vk::PipelineStageFlags2::VERTEX_SHADER.as_raw(),
        /// The stage where fragment shaders execute.
        #[display("fragment shader")]
        FRAGMENT_SHADER = vk::PipelineStageFlags2::FRAGMENT_SHADER.as_raw(),
        /// The stage where late fragment tests and depth/stencil store operations take place.
        #[display("depth stencil output")]
        DEPTH_STENCIL_OUTPUT = vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS.as_raw(),
        /// The stage where colors are output from a graphics pipeline.
        #[display("color output")]
        COLOR_OUTPUT = vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT.as_raw(),
        /// The stage where compute shaders execute.
        #[display("compute shader")]
        COMPUTE_SHADER = vk::PipelineStageFlags2::COMPUTE_SHADER.as_raw(),
        /// The stage where all transfer commands execute.
        #[display("transfer")]
        TRANSFER = vk::PipelineStageFlags2::TRANSFER.as_raw()
    }
}

impl From<MemoryDependencyHint> for vk::PipelineStageFlags2 {

    #[inline(always)]
    fn from(value: MemoryDependencyHint) -> Self {
        Self::from_raw(value.as_raw())
    }
}
