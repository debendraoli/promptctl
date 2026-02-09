//! Clipboard operations for promptctl.

use arboard::Clipboard;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ClipboardError {
    #[error("failed to access clipboard: {0}")]
    Access(#[from] arboard::Error),
}

#[allow(dead_code)]
pub fn copy_to_clipboard(text: &str) -> Result<(), ClipboardError> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}
