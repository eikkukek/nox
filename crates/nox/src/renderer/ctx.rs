use super::*;

pub struct RendererContext {
    pub(super) global_resources: Arc<RwLock<GlobalResources>>,
    pub(super) transfer_requests: Rc<UnsafeCell<TransferRequests>>,
    pub(super) frame_buffer_size: image::Dimensions,
    pub(crate) device: Arc<ash::Device>,
    pub(crate) physical_device_info: Arc<PhysicalDeviceInfo>,
}

impl RendererContext {

    #[inline(always)]
    pub fn edit_resources(
        &self,
        mut f: impl FnMut(&mut GlobalResources) -> Result<(), ResourceError>,
    ) -> Result<(), ResourceError>
    {
        let mut resources = self.global_resources.write().unwrap();
        f(&mut resources)
    }

    #[inline(always)]
    pub fn edit_transfer_requests(
        &mut self,
        mut f: impl FnMut(&mut TransferRequests),
    )
    {
        let requests = unsafe {
            &mut *self.transfer_requests.get()
        };
        f(requests)
    }

    #[inline(always)]
    pub fn frame_buffer_size(&self) -> image::Dimensions {
        self.frame_buffer_size
    }

    #[inline(always)]
    pub fn buffer_size(&self, buffer: BufferId) -> Option<u64> {
        self.global_resources.read().unwrap().buffer_size(buffer)
    }
}
