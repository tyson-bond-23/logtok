use anyhow::Result;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

/// Copy text to the system clipboard. Returns an error message if clipboard
/// is unavailable (e.g., headless/SSH session) rather than panicking.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Clipboard not available: {}. Output written to stdout instead.", e))?;
    ctx.set_contents(text.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}
