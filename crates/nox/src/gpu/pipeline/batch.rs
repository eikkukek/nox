use compact_str::format_compact;

use nox_ash::vk;

use nox_threads::{
    futures::future::RemoteHandle,
    executor::SpawnExt,
    sync::{FutureLock, SwapLock},
};
use nox_mem::{
    vec::{Vec32, FixedVec32, Vector},
    Display,
    vec32,
};
use nox_alloc::arena::Arena;

use crate::{
    gpu::prelude::*,
    dev::error::*,
    sync::*,
    log,
};

#[must_use]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("(batch id: {0}, pipeline index: {0})")]
pub struct GraphicsPipelineId(PipelineBatchId, u32);

impl GraphicsPipelineId {
   
    /// Gets the batch portion of the id.
    #[inline(always)]
    pub fn batch_id(self) -> PipelineBatchId {
        self.0
    }

    /// Gets the pipeline portion of the id.
    #[inline(always)]
    pub fn pipeline_id(self) -> u32 {
        self.1
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display("(batch id: {0}, pipeline index: {0})")]
pub struct ComputePipelineId(PipelineBatchId, u32);

impl ComputePipelineId {
   
    /// Gets the batch portion of the id.
    #[inline(always)]
    pub fn batch_id(self) -> PipelineBatchId {
        self.0
    }

    /// Gets the pipeline portion of the id.
    #[inline(always)]
    pub fn pipeline_id(self) -> u32 {
        self.1
    }
}

pub(crate) struct PipelineBatchInner {
    id: PipelineBatchId,
    graphics_pipelines: SwapLock<Vec32<Option<GraphicsPipeline>>>,
    compute_pipelines: SwapLock<Vec32<Option<ComputePipeline>>>,
}

impl PipelineBatchInner {

    fn new(
        id: PipelineBatchId,
        graphics_pipelines: Vec32<Option<GraphicsPipeline>>,
        compute_pipelines: Vec32<Option<ComputePipeline>>
    ) -> Self {
        Self {
            id,
            graphics_pipelines: SwapLock::new(graphics_pipelines),
            compute_pipelines: SwapLock::new(compute_pipelines),
        }
    }
}

#[derive(Clone)]
pub(crate) struct PipelineBatch {
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

    pub fn destroy_graphics_pipelines(&self, ids: &[GraphicsPipelineId]) -> Result<()> {
        if ids.is_empty() { return Ok(()) }
        let inner = self.inner.load()?;
        let batch_id = inner.id;
        inner.graphics_pipelines
            .modify(|pipelines| {
                for &id in ids {
                    if id.batch_id() != batch_id {
                        return Err(Error::just_context(format_compact!(
                            "graphics pipeline id {id} batch id is different from this batch id {batch_id}",
                        )))
                    }
                    pipelines
                        .get_mut(id.1 as usize)
                        .ok_or_else(|| Error::just_context(format_compact!(
                            "invalid graphics pipeline id {id}"
                        )))?.take();
                }
                Ok(())
            })?;
        Ok(())
    }

    pub fn destroy_compute_pipelines(&self, ids: &[ComputePipelineId]) -> Result<()> {
        if ids.is_empty() { return Ok(()) }
        let inner = self.inner.load()?;
        let batch_id = inner.id;
        inner.compute_pipelines
            .modify(|pipelines| {
                for &id in ids {
                    if id.batch_id() != batch_id {
                        return Err(Error::just_context(format_compact!(
                            "compute pipeline id {id} batch id is different from this batch id {batch_id}",
                        )))
                    }
                    pipelines
                        .get_mut(id.1 as usize)
                        .ok_or_else(|| Error::just_context(format_compact!(
                            "invalid compute pipeline id {id}"
                        )))?.take();
                }
                Ok(())
            })?;
        Ok(())
    }
}

pub struct PipelineBatchBuilder {
    this_id: PipelineBatchId,
    gpu: Gpu,
    graphics_create_infos: Option<Vec32<GraphicsPipelineCreateInfo>>,
    compute_create_infos: Option<Vec32<ComputePipelineCreateInfo>>,
    cache: Option<PipelineCache>,
    built: bool,
}

impl PipelineBatchBuilder {

    #[inline(always)]
    pub(crate) fn new(
        gpu: Gpu,
        cache: Option<PipelineCache>,
    ) -> Self
    {
        Self {
            this_id: gpu.reserve_pipeline_batch_slot(),
            gpu,
            graphics_create_infos: Some(vec32![]),
            compute_create_infos: Some(vec32![]),
            cache,
            built: false,
        }
    }

    #[inline(always)]
    pub fn with_graphics_pipeline<F>(
        &mut self,
        create_info: GraphicsPipelineCreateInfo
    ) -> GraphicsPipelineId
    {
        let infos = self.graphics_create_infos.as_mut().unwrap();
        let idx = infos.len();
        infos.push(create_info);
        GraphicsPipelineId(self.this_id, idx)
    }

    #[inline(always)]
    pub fn with_compute_pipeline<F>(
        &mut self,
        create_infos: ComputePipelineCreateInfo,
    ) -> Result<ComputePipelineId>
    {
        let infos = self.compute_create_infos.as_mut().unwrap();
        let idx = infos.len();
        infos.push(create_infos);
        Ok(ComputePipelineId(self.this_id, idx))
    }

    /// You should always call this once you have finished building.
    ///
    /// Unbuilt pipeline batches are built on [`Drop`], but you should *not* rely on that.
    #[inline(always)]
    pub fn build(mut self) -> Result<()> {
        self.built = true;
        let thread_pool = self.gpu.thread_pool().clone();
        let gpu = self.gpu.clone();
        let create_infos = self.graphics_create_infos.take().unwrap();
        let cache = self.cache.clone();
        let this_id = self.this_id;
        let graphics = thread_pool.spawn_with_handle(async move {
            let tmp_alloc = gpu.tmp_alloc();
            let tmp_alloc = tmp_alloc.guard();
            let pipeline_count = create_infos.len();
            if pipeline_count == 0 {
                return Ok(Default::default())
            }
            let mut prepared_create_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            for info in &create_infos {
                prepared_create_infos.push(info.prepare(&gpu, &tmp_alloc)
                    .context("failed to convert graphics pipeline info")?);
            }
            let mut vk_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            for info in &prepared_create_infos {
                vk_infos.push(info.0.as_create_info());
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
                    create_infos.len(),
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
                        prepared_create_infos[i].1.clone(),
                        &create_infos[i]
                    ))
                }).collect();
            Ok(graphics_pipelines)
        }).context("send error")?;
        let gpu = self.gpu.clone();
        let mut create_infos = self.compute_create_infos.take().unwrap();
        let cache = self.cache.clone();
        let compute = thread_pool.spawn_with_handle(async move {
            let tmp_alloc = gpu.tmp_alloc();
            let tmp_alloc = tmp_alloc.guard();
            let pipeline_count = create_infos.len();
            if pipeline_count == 0 {
                return Ok(Default::default())
            }
            let mut vk_infos = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            let mut shader_sets = FixedVec32
                ::with_capacity(pipeline_count, &tmp_alloc)?;
            for info in &mut create_infos {
                let (vk_info, shader_set) = info
                    .prepare(&gpu)
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
                    vk_infos.len(),
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
            Ok(PipelineBatchInner::new(this_id, graphics_pipelines, compute_pipelines))
        }).context("send error")?;
        self.gpu.init_pipeline_batch(
            this_id, PipelineBatch::new(fut),
        );
        Ok(())
    }
}

impl Drop for PipelineBatchBuilder {

    #[inline(always)]
    fn drop(&mut self) {
        if !self.built {
            log::warn!(
                "pipeline batch (id: {}) was not built, pipeline batches should always be explicitly built and you should not rely on automatic builds",
                self.this_id
            );
            let cloned = Self {
                this_id: self.this_id,
                gpu: self.gpu.clone(),
                graphics_create_infos: self.graphics_create_infos.clone(),
                compute_create_infos: self.compute_create_infos.clone(),
                cache: self.cache,
                built: false,
            };
            cloned.build().ok();
        }
    }
}
