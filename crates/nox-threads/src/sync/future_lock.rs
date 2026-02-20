use core::{
    mem::{ManuallyDrop, MaybeUninit},
    sync::atomic::{self, AtomicBool},
    result,
    error,
};

use nox_error::Context;

use crate::{
    futures::{
        future::Future,
        executor::block_on,
    },
    error::{Result, Error},
};

use parking_lot::Mutex;

union Inner<T, F, E = Error>
    where 
        T: 'static,
        F: Future<Output = result::Result<T, E>>,
        E: error::Error + Send + Sync + 'static,
{
    pending: ManuallyDrop<F>,
    ready: ManuallyDrop<T>,
}

/// A lock that waits for a future when it's first accessed.
///
/// # Example
/// ``` rust
/// use std::thread::sleep;
/// use std::time::Duration;
///
/// use nox_threads::{ThreadPool, SpawnExt};
/// use nox_threads::sync::FutureLock;
/// use nox_threads::error::Error;
///
/// let pool = ThreadPool::new().unwrap();
/// let lock: FutureLock<_, ()> = FutureLock::new(pool.spawn_with_handle(async {
///     sleep(Duration::from_secs(1));
///     Ok::<_, Error>(10)
/// })).unwrap();
///
/// assert_eq!(lock.load().ok(), Some(&10));
/// assert_eq!(lock.load().ok(), Some(&10));
/// ```
pub struct FutureLock<T, F, E = Error>
    where
        F: Future<Output = result::Result<T, E>>,
        T: 'static,
        E: error::Error + Send + Sync + 'static,
{
    inner: MaybeUninit<Inner<T, F, E>>,
    is_ready: AtomicBool,
    mtx: Mutex<i8>,
}

impl<T, F, E> FutureLock<T, F, E>
    where
        F: Future<Output = result::Result<T, E>>,
        T: 'static,
        E: error::Error + Send + Sync + 'static,
{
    
    #[inline(always)]
    pub fn new(f: F) -> Self {
        Self {
            inner: MaybeUninit::new(Inner { pending: ManuallyDrop::new(f) }),
            is_ready: AtomicBool::new(false),
            mtx: Mutex::new(0),
        }
    }
    
    /// Loads the inner value.
    ///
    /// No locking takes place if the future has finished.
    pub fn load(&self) -> Result<&T> {
        if self.is_ready.load(atomic::Ordering::Relaxed) {
            unsafe {
                Ok(&self.inner
                    .assume_init_ref().ready
                )
            }
        } else {
            let mut lock = self.mtx.lock();
            match *lock {
                -1 => {
                    Err(Error::just_context("lock poisoned"))
                },
                1 => unsafe {
                    Ok(&self.inner
                        .assume_init_ref().ready
                    )
                },
                0 => unsafe {
                    let ready = block_on(
                        ManuallyDrop::into_inner(self.inner
                            .assume_init_read().pending
                        )
                    ).inspect_err(|_| *lock = -1)
                    .context("future failed")?;
                    self.inner
                        .as_ptr().cast_mut()
                        .write(Inner { ready: ManuallyDrop::new(ready) });
                    *lock = 1;
                    self.is_ready.store(true, atomic::Ordering::Relaxed);
                    Ok(&self.inner.assume_init_ref().ready)
                },
                _ => unreachable!()
            }
        }
    }
}

impl<T, F, E> Drop for FutureLock<T, F, E>
    where
        F: Future<Output = result::Result<T, E>>,
        T: 'static,
        E: error::Error + Send + Sync + 'static,
{

    fn drop(&mut self) {
        let mut lock = self.mtx.lock();
        match *lock {
            1 => unsafe {
                ManuallyDrop::into_inner(self.inner
                    .assume_init_read().ready
                );
            }
            0 => unsafe {
                block_on(ManuallyDrop::into_inner(self.inner
                    .assume_init_read().pending
                )).ok();
            }
            _ => {}
        }
        *lock = -1;
    }
}
