use std::mem;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use x11_clipboard::Clipboard;

pub mod cli;
pub mod fd_reader;
pub mod key_event;
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

type ClipboardError = x11_clipboard::error::Error;

pub fn put_clipboard<T: Into<Vec<u8>>>(text: T) -> Result<(), ClipboardError> {
    let clipboard = Clipboard::new()?;
    clipboard.store(
        clipboard.setter.atoms.clipboard,
        clipboard.setter.atoms.utf8_string,
        text,
    )?;

    // FIXME: this crate will make the stored content cleaned when `clipboard` is dropped
    //  or the program exits.
    mem::forget(clipboard);
    Ok(())
}

pub fn load_clipboard() -> Result<String, ClipboardError> {
    let clipboard = Clipboard::new()?;
    let result = clipboard.load(
        clipboard.setter.atoms.clipboard,
        clipboard.setter.atoms.utf8_string,
        clipboard.setter.atoms.property,
        Duration::from_secs(1),
    );

    match result {
        Ok(data) => Ok(String::from_utf8_lossy(&data).to_string()),
        Err(ClipboardError::Timeout) => Ok(String::new()),
        Err(e) => Err(e),
    }
}

pub const DISTRIBUTION_NAME: &str = "Rime";
pub const DISTRIBUTION_CODE_NAME: &str = "Rime";
pub const DISTRIBUTION_VERSION: &str = "0.0.0";
pub const APP_NAME: &str = "rime-tui";
