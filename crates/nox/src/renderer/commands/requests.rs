use nox_mem::{
    GlobalAlloc,
    GLOBAL_ALLOC,
    Vector,
    vec_types::AllocVec,
    slot_map,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CommandRequestID(u32);

#[derive(Clone, Copy)]
pub struct TransferRequest {
    pub staging_buffer_capacity: u32,
}

impl TransferRequest {

    pub fn new(staging_buffer_capacity: u32) -> Self {
        Self {
            staging_buffer_capacity,
        }
    }
}

pub struct CommandRequests {
    pub(crate) transfer_requests: AllocVec<'static, TransferRequest, GlobalAlloc, slot_map::Dyn>,
}

impl CommandRequests {

    pub(crate) fn new() -> Self {
        CommandRequests {
            transfer_requests: AllocVec::new(&GLOBAL_ALLOC).unwrap(),
        }
    }

    pub fn task_count(&self) -> usize {
        self.transfer_requests.len()
    }

    pub fn reserve_transfer_requests(&mut self, capacity: u32) {
        self.transfer_requests.reserve(capacity as usize).unwrap();
    }

    pub fn add_transfer_request(&mut self, request: TransferRequest) -> CommandRequestID {
        let index = self.transfer_requests.len() as u32;
        self.transfer_requests.push(request).unwrap();
        CommandRequestID(index)
    }

    pub fn transfer_iter(&self) -> impl Iterator<Item = (CommandRequestID, TransferRequest)> {
        self.transfer_requests
            .iter()
            .enumerate()
            .map(|(i, v)| (CommandRequestID(i as u32), *v))
    }
}
