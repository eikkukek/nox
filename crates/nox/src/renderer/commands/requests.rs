use nox_mem::{
    GlobalAlloc,
    GLOBAL_ALLOC,
    vec_types::{Vector, AllocVec},
    slot_map,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CommandRequestId(u32);

impl Default for CommandRequestId {

    fn default() -> Self {
        Self(u32::MAX)
    }
}

pub struct TransferRequests {
    pub(crate) transfer_requests: AllocVec<'static, u64, GlobalAlloc, slot_map::Dyn>,
}

impl TransferRequests {

    pub(crate) fn new() -> Self {
        TransferRequests {
            transfer_requests: AllocVec::new(&GLOBAL_ALLOC).unwrap(),
        }
    }

    pub fn task_count(&self) -> usize {
        self.transfer_requests.len()
    }

    pub fn reserve_transfer_requests(&mut self, capacity: u32) {
        self.transfer_requests.reserve(capacity as usize).unwrap();
    }

    pub fn add_request(&mut self, staging_block_size: u64) -> CommandRequestId {
        let index = self.transfer_requests.len() as u32;
        self.transfer_requests.push(staging_block_size).unwrap();
        CommandRequestId(index)
    }

    pub(crate) fn transfer_iter(&self) -> impl Iterator<Item = (CommandRequestId, u64)> {
        self.transfer_requests
            .iter()
            .enumerate()
            .map(|(i, v)| (CommandRequestId(i as u32), *v))
    }
}
