//! Provides [`FutureLock`], [`SwapLock`] and [`OnceLockExt`].

mod future_lock;
mod swap_lock;

pub use swap_lock::*;
pub use future_lock::FutureLock;
