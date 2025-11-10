use nox_mem::{slot_map::*, vec_types::{GlobalVec, Vector}};

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CommandRequestId(pub(crate) SlotIndex<(LinearDeviceAllocId, GlobalVec<(TimelineSemaphoreId, u64)>)>);

pub struct TransferRequests {
    pub(crate) async_requests: GlobalSlotMap<(
        LinearDeviceAllocId,
        GlobalVec<(TimelineSemaphoreId, u64)>,
    )>,
}

impl TransferRequests {

    #[inline(always)]
    pub fn new() -> Self {
        TransferRequests {
            async_requests: GlobalSlotMap::new(),
        }
    }

    #[inline(always)]
    pub fn async_request_count(&self) -> u32 {
        self.async_requests.len()
    }

    #[inline(always)]
    pub fn add_async_request(&mut self, staging_alloc: LinearDeviceAllocId, signal_semaphores: &[(TimelineSemaphoreId, u64)]) -> CommandRequestId {
        let index = self.async_requests.insert((staging_alloc, GlobalVec::from(signal_semaphores)));
        CommandRequestId(index)
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self) {
        self.async_requests.clear();
    }

    #[inline(always)]
    pub(crate) fn async_transfer_iter(&self) -> impl Iterator<Item = (CommandRequestId, (LinearDeviceAllocId, &[(TimelineSemaphoreId, u64)]))> {
        self.async_requests
            .iter()
            .map(|(i, v)| (CommandRequestId(i), (v.0, v.1.as_slice())))
    }
}
