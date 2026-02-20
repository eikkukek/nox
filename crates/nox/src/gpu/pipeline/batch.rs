use std::sync::Arc;

use nox_ash::vk;

use nox_threads::{
    futures::future::RemoteHandle,
    executor::SpawnExt,
    sync::{FutureLock, SwapLock},
};
use nox_mem::vec::{Vec32, FixedVec32, Vector};
use nox_alloc::arena::Arena;

use crate::dev::error::*;

use crate::gpu::prelude::*;

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GraphicsPipelineId(pub(crate) PipelineBatchId, pub(crate) u32);

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComputePipelineId(pub(crate) PipelineBatchId, pub(crate) u32);

pub(crate) struct PipelineBatchInner {
    graphics_pipelines: SwapLock<Vec32<Option<GraphicsPipeline>>>,
    compute_pipelines: SwapLock<Vec32<Option<ComputePipeline>>>,
}

impl PipelineBatchInner {

    fn new(
        graphics_pipelines: Vec32<Option<GraphicsPipeline>>,
        compute_pipelines: Vec32<Option<ComputePipeline>>
    ) -> Self {
        Self {
            graphics_pipelines: SwapLock::new(graphics_pipelines),
            compute_pipelines: SwapLock::new(compute_pipelines),
        }
    }
}

#[derive(Clone)]
pub struct PipelineBatch {
    inner: Arc<FutureLock<PipelineBatchInner, RemoteHandle<Result<PipelineBatchInner>>>>,
}

impl PipelineBatch {

    #[inline(always)]
    fn new(f: RemoteHandle<Result<PipelineBatchInner>>) -> Self {
        Self { inner: Arc::new(FutureLock::new(f)) }
    }

    #[inline(always)]
    pub fn get_graphics_pipeline(&self, idx: u32) -> Result<GraphicsPipeline> {
        self.inner
            .load()?.graphics_pipelines
            .load()
            .get(idx as usize)
            .ok_or_else(|| Error::just_context("invalid id"))?
            .as_ref()
            .ok_or_else(|| Error::just_context("graphics pipeline destroyed"))
            .cloned()
    }

    #[inline(always)]
    pub fn get_compute_pipeline(&self, idx: u32) -> Result<ComputePipeline> {
        self.inner
            .load()?.compute_pipelines
            .load()
            .get(idx as usize)
            .ok_or_else(|| Error::just_context("invalid id"))?
            .as_ref()
            .ok_or_else(|| Error::just_context("compute pipeline is destroyed"))
            .cloned()
    }

    pub fn destroy_graphics_pipelines(&self, idx: &[u32]) -> Result<()> {
        self.inner
            .load()?.graphics_pipelines
            .modify(|pipelines| {
                for &idx in idx {
                    if let Some(p) = pipelines.get_mut(idx as usize) {
                        p.take();
                    }
                }
            });
        Ok(())
    }

    pub fn destroy_compute_pipelines(&self, idx: &[u32]) -> Result<()> {
        self.inner
            .load()?.compute_pipelines
            .modify(|pipelines| {
                for &idx in idx {
                    if let Some(p) = pipelines.get_mut(idx as usize) {
                        p.take();
                    }
                }
            });
        Ok(())
    }
}

pub struct PipelineBatchBuilder {
    this_id: PipelineBatchId,
    gpu: Arc<Gpu>,
    graphics_attributes: Option<Vec32<GraphicsPipelineAttributes>>,
    compute_attributes: Option<Vec32<ComputePipelineAttributes>>,
    cache: Option<PipelineCache>,
}

impl PipelineBatchBuilder {

    #[inline(always)]
    pub(crate) fn new(
        gpu: Arc<Gpu>,
        n_graphics_pipelines: u32,
        n_compute_pipelines: u32,
        cache: Option<PipelineCache>,
    ) -> Self
    {
        Self {
            this_id: gpu.reserve_pipeline_batch_slot(),
            gpu,
            graphics_attributes: Some(Vec32::with_capacity(n_graphics_pipelines)),
            compute_attributes: Some(Vec32::with_capacity(n_compute_pipelines)),
            cache,
        }
    }

    #[inline(always)]
    pub fn with_graphics_pipeline<F>(
        &mut self,
        shader_set_id: ShaderSetId,
        f: F,
    ) -> Result<GraphicsPipelineId>
        where
            F: FnOnce(&mut GraphicsPipelineAttributes) -> &mut GraphicsPipelineAttributes
    {
        let mut attributes = GraphicsPipelineAttributes::new(shader_set_id);
        f(&mut attributes);
        let attr = self.graphics_attributes.as_mut().unwrap();
        let idx = attr.len();
        attr.push(attributes);
        Ok(GraphicsPipelineId(self.this_id, idx))
    }

    #[inline(always)]
    pub fn with_compute_pipeline<F>(
        &mut self,
        attributes: ComputePipelineAttributes,
    ) -> Result<ComputePipelineId>
    {
        let attr = self.compute_attributes.as_mut().unwrap();
        let idx = attr.len();
        attr.push(attributes);
        Ok(ComputePipelineId(self.this_id, idx))
    }

    #[inline(always)]
    pub fn build(mut self) -> Result<()> {
        let thread_pool = self.gpu.thread_pool().clone();
        let gpu = self.gpu.clone();
        let attributes = self.graphics_attributes.take().unwrap();
        let cache = self.cache.clone();
        let graphics = thread_pool.spawn_with_handle(async move {
            let tmp_alloc = gpu.tmp_alloc();
            let tmp_alloc = tmp_alloc.guard();
            let pipeline_count = attributes.len();
            if pipeline_count == 0 {
                return Ok(Default::default())
            }
            let mut create_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            let mut vk_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            for attr in &attributes {
                create_infos.push(attr.as_create_info(&gpu, &tmp_alloc)
                    .context("failed to convert graphics pipeline info")?);
                let (info, _) = unsafe { create_infos.last().unwrap_unchecked() };
                const VIEWPORT_STATE: vk::PipelineViewportStateCreateInfo = vk::PipelineViewportStateCreateInfo {
                    s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
                    p_next: core::ptr::null(),
                    flags: vk::PipelineViewportStateCreateFlags::empty(),
                    viewport_count: 1,
                    p_viewports: core::ptr::null(),
                    scissor_count: 1,
                    p_scissors: core::ptr::null(),
                    _marker: core::marker::PhantomData,
                };
                vk_infos.push(vk::GraphicsPipelineCreateInfo {
                    s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
                    p_next: &info.rendering_info as *const _ as *const _,
                    stage_count: info.shader_stage_infos.len() as u32,
                    p_stages: info.shader_stage_infos.as_ptr(),
                    p_vertex_input_state: &info.vertex_input_state,
                    p_input_assembly_state: &info.input_assembly_state,
                    p_tessellation_state: &info.tesellation_state,
                    p_viewport_state: &VIEWPORT_STATE,
                    p_rasterization_state: &info.rasterization_state,
                    p_multisample_state: &info.multisample_state,
                    p_depth_stencil_state: &info.depth_stencil_state,
                    p_color_blend_state: &info.color_blend_state,
                    p_dynamic_state: &info.dynamic_state,
                    layout: info.layout,
                    ..Default::default()
                });
            }
            let mut pipelines = FixedVec32
                ::with_capacity(create_infos.len(), &tmp_alloc)?;
            unsafe {
                let device = gpu.vk().device();
                let pipeline_cache = cache 
                    .map(|cache| cache.handle().into_inner())
                    .unwrap_or(vk::PipelineCache::null());
                let result = (device.fp_v1_0().create_graphics_pipelines)(
                    device.handle(),
                    pipeline_cache,
                    attributes.len(),
                    vk_infos.as_ptr(),
                    core::ptr::null(),
                    pipelines.as_mut_ptr(),
                );
                if result != vk::Result::SUCCESS {
                    return Err(Error::new(result, "failed to create graphics pipelines"))
                }
                pipelines.set_len(create_infos.len());
            }
            let graphics_pipelines: Vec32<_> = pipelines
                .iter()
                .copied()
                .enumerate()
                .map(|(i, handle)| unsafe {
                    Some(GraphicsPipeline::new(
                        gpu.vk().clone(),
                        handle,
                        create_infos[i].1.clone(),
                        &attributes[i]
                    ))
                }).collect();
            Ok(graphics_pipelines)
        }).context("send error")?;
        let gpu = self.gpu.clone();
        let attributes = self.compute_attributes.take().unwrap();
        let cache = self.cache.clone();
        let compute = thread_pool.spawn_with_handle(async move {
            let tmp_alloc = gpu.tmp_alloc();
            let tmp_alloc = tmp_alloc.guard();
            let pipeline_count = attributes.len();
            if pipeline_count == 0 {
                return Ok(Default::default())
            }
            let mut vk_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            let mut shader_sets = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            for attr in &attributes {
                let (vk_info, shader_set) = attr
                    .as_create_info(&gpu)
                    .context("failed to convert compute pipeline info")?;
                shader_sets.push(shader_set);
                vk_infos.push(vk_info);
            }
            let mut pipelines = FixedVec32
                ::with_capacity(vk_infos.len(), &tmp_alloc)?;
            unsafe {
                let device = gpu.vk().device();
                let pipeline_cache = cache
                    .map(|cache| cache.handle().into_inner())
                    .unwrap_or(vk::PipelineCache::null());
                let result = (device.fp_v1_0().create_compute_pipelines)(
                    device.handle(),
                    pipeline_cache,
                    attributes.len(),
                    vk_infos.as_ptr(),
                    core::ptr::null(),
                    pipelines.as_mut_ptr(),

                );
                if result != vk::Result::SUCCESS {
                    return Err(Error::new(result, "failed to create compute pipelines"))
                }
                pipelines.set_len(vk_infos.len());
            };
            let compute_pipelines: Vec32<_> =
                shader_sets 
                    .into_iter()
                    .enumerate()
                    .map(|(i, shader_set)| {
                        Some(unsafe { ComputePipeline::new(
                            gpu.vk().clone(),
                            pipelines[i],
                            shader_set,
                        )})
                    }).collect();
            Ok(compute_pipelines)
        }).context("send error")?;
        let fut = thread_pool.spawn_with_handle(async move {
            let graphics_pipelines = graphics.await
                .context("failed to create graphics pipelines")?;
            let compute_pipelines = compute.await
                .context("failed to create compute pipelines")?;
            Ok(PipelineBatchInner::new(graphics_pipelines, compute_pipelines))
        }).context("send error")?;
        self.gpu.init_pipeline_batch(
            self.this_id, PipelineBatch::new(fut),
        );
        Ok(())
    }
}
