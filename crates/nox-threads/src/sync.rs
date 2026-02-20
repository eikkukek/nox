//! Provides [`FutureLock`] and [`SwapLock`].

mod future_lock;
mod swap_lock;

pub use future_lock::FutureLock;
pub use swap_lock::SwapLock;
