use nox_mem::slot_map::*;

use crate::*;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandRequestId(SlotIndex<LinearDeviceAllocId>);

pub struct TransferRequests {
    pub(crate) async_requests: GlobalSlotMap<LinearDeviceAllocId>,
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
    pub fn add_async_request(&mut self, staging_alloc: LinearDeviceAllocId) -> CommandRequestId {
        let index = self.async_requests.insert(staging_alloc);
        CommandRequestId(index)
    }

    #[inline(always)]
    pub(crate) fn clear(&mut self) {
        self.async_requests.clear();
    }

    #[inline(always)]
    pub(crate) fn async_transfer_iter(&self) -> impl Iterator<Item = (CommandRequestId, LinearDeviceAllocId)> {
        self.async_requests
            .iter()
            .map(|(i, &v)| (CommandRequestId(i), v))
    }
}
