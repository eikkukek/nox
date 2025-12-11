use nox_mem::{
    slot_map::*,
    vec_types::{GlobalVec, Vector}
};

use crate::gpu::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CommandRequestId(pub(crate) SlotIndex<(LinearDeviceAllocId, GlobalVec<(TimelineSemaphoreId, u64)>)>);

#[derive(Default)]
pub(crate) struct TransferRequests {
    pub async_requests: GlobalSlotMap<(
        LinearDeviceAllocId,
        GlobalVec<(TimelineSemaphoreId, u64)>,
    )>,
}

impl TransferRequests {

    #[inline(always)]
    pub fn async_request_count(&self) -> u32 {
        self.async_requests.len()
    }

    #[inline(always)]
    pub fn add_async_request(
        &mut self,
        staging_alloc: LinearDeviceAllocId,
        signal_semaphores: &[(TimelineSemaphoreId, u64)]
    ) -> CommandRequestId
    {
        let index = self.async_requests.insert((staging_alloc, GlobalVec::from(signal_semaphores)));
        CommandRequestId(index)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.async_requests.clear();
    }

    pub fn is_empty(&mut self) ->  bool {
        self.async_requests.is_empty()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (CommandRequestId, (LinearDeviceAllocId, &[(TimelineSemaphoreId, u64)]))> {
        self.async_requests
            .iter()
            .map(|(i, v)| (CommandRequestId(i), (v.0, v.1.as_slice())))
    }
}
