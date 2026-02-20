//! The sync prelude of [`nox`].

pub use std::sync::{
    Arc, OnceLock, LazyLock,
};

pub use parking_lot::{
    RwLock, RwLockWriteGuard, RwLockReadGuard,
    RwLockUpgradableReadGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard,
    Mutex, MutexGuard, FairMutex, FairMutexGuard,
    MappedMutexGuard, MappedFairMutexGuard,
    ReentrantMutex, ReentrantMutexGuard, MappedReentrantMutexGuard,
    Condvar,
};

pub use nox_threads::sync::{FutureLock, SwapLock};
