use std::sync::Arc;

use core::ops::Deref;

use nox_ash::vk;

use crate::gpu::prelude::*;

pub(super) struct Inner {
    pub vk: Arc<Vulkan>,
    pub handle: vk::Pipeline,
    pub shader_set: Arc<ShaderSetInner>,
}

impl Drop for Inner {

    fn drop(&mut self) {
        unsafe {
            self.vk.device().destroy_pipeline(self.handle, None);
        }
    }
}

#[derive(Clone)]
pub(crate) struct PipelineHandle {
    inner: Arc<Inner>,
}

impl PipelineHandle {

    #[inline(always)]
    pub(super) unsafe fn new(
        vk: Arc<Vulkan>,
        handle: vk::Pipeline,
        shader_set: Arc<ShaderSetInner>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                vk,
                handle,
                shader_set,
            }),
        }
    }

    #[inline(always)]
    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    #[inline(always)]
    pub fn shader_set(&self) -> &Arc<ShaderSetInner> {
        &self.shader_set
    }
}

impl Deref for PipelineHandle {

    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
