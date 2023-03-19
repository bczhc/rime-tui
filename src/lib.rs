use std::sync::{Mutex, MutexGuard, PoisonError};

pub mod cli;
pub mod fd_reader;
pub mod key_event;
pub mod rime;
pub mod tui;
pub mod xinput;

pub trait WithLockExt<T> {
    fn with_lock<F, R>(&self, block: F) -> Result<R, PoisonError<MutexGuard<T>>>
    where
        F: FnOnce(MutexGuard<'_, T>) -> R;
}

impl<T> WithLockExt<T> for Mutex<T> {
    fn with_lock<F, R>(&self, block: F) -> Result<R, PoisonError<MutexGuard<T>>>
    where
        F: FnOnce(MutexGuard<'_, T>) -> R,
    {
        match self.lock() {
            Ok(g) => Ok(block(g)),
            Err(e) => Err(e),
        }
    }
}
