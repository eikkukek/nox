use rustc_hash::FxHashMap;

use crate::gpu::*;

use crate::dev::error::{Error, Result, Context};

const VERTEX_SHADER: &'static str = "
#version 450

vec3 positions[] = {

};
";

pub struct FullScreenPass {
    pipelines: FxHashMap<ImageFormat, GraphicsPipelineId>,
    current_pipeline: Option<GraphicsPipelineId>,
    pipeline_layout: PipelineLayoutId,
    shader_resource: ShaderResourceId,
    cache: Option<PipelineCacheId>,
}

impl FullScreenPass {

    pub fn init(gpu: &mut GpuContext) -> Result<Self> {
        let mut info = GraphicsPipelineInfo::new(Default::default());
    }
}
