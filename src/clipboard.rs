//! Clipboard operations for promptctl.

use arboard::Clipboard;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("failed to access clipboard: {0}")]
    Access(#[from] arboard::Error),
}

/// Copy text to the system clipboard
pub fn copy_to_clipboard(text: &str) -> Result<(), ClipboardError> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}
