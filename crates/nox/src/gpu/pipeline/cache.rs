use std::sync::Arc;

use nox_ash::vk;

use nox_mem::slice::AllocSlice;

use crate::dev::error::{Result, Error};

use crate::gpu::prelude::{Vulkan, Handle};

struct Inner {
    vk: Arc<Vulkan>,
    handle: vk::PipelineCache,
}

#[derive(Clone)]
pub struct PipelineCache {
    inner: Arc<Inner>,
}

impl PipelineCache {

    pub(crate) unsafe fn new(vk: Arc<Vulkan>, handle: vk::PipelineCache) -> Self {
        Self {
            inner: Arc::new(Inner { vk, handle })
        }
    }

    pub(crate) fn handle(&self) -> Handle<'_, vk::PipelineCache> {
        Handle::new(self.inner.handle)
    }

    #[inline(always)]
    pub fn retrieve_data(
        &self,
    ) -> Result<Box<[u8]>>
    {
        let device = self.inner.vk.device();
        let handle = self.inner.handle;
        unsafe {
            let mut cache_size = 0;
            let result = (device.fp_v1_0().get_pipeline_cache_data)(
                device.handle(),
                handle,
                &mut cache_size,
                Default::default(),
            );
            if result != vk::Result::SUCCESS {
                return Err(Error::new(result, "failed to get pipeline cache data"))
            }
            let mut data = Box::uninit_slice(cache_size);
            let result = (device.fp_v1_0().get_pipeline_cache_data)(
                device.handle(),
                handle,
                &mut cache_size,
                data.as_mut_ptr() as *mut core::ffi::c_void,
            );
            if result != vk::Result::SUCCESS {
                return Err(Error::new(result, "failed to get pipeline cache data"))
            }
            Ok(data)
        }
    }
}

impl Drop for Inner {
    
    fn drop(&mut self) {
        unsafe {
            self.vk.device().destroy_pipeline_cache(self.handle, None);
        }
    }
}
