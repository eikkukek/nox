use std::sync::Arc;

use parking_lot::Mutex;
use arc_swap::ArcSwap;

/// A simple lock that uses [`arc_swap::ArcSwap`] internally.
///
/// Loads are lock free, stores lock and clone the inner data.
#[derive(Default)]
pub struct SwapLock<T: Clone> {
    mtx: Mutex<()>,
    data: ArcSwap<T>,
}

impl<T: Clone> SwapLock<T> {

    #[inline(always)]
    pub fn new(val: T) -> Self {
        Self {
            mtx: Mutex::new(()),
            data: ArcSwap::new(Arc::new(val)),
        }
    }

    #[inline(always)]
    pub fn load(&self) -> arc_swap::Guard<Arc<T>> {
        self.data.load()
    }

    #[inline(always)]
    pub fn modify<F, U>(&self, f: F) -> U
        where F: FnOnce(&mut T) -> U
    {
        let _lock = self.mtx.lock();
        let mut data = self.data.load().as_ref().clone();
        let u = f(&mut data);
        self.data.store(Arc::new(data));
        u
    }
}
