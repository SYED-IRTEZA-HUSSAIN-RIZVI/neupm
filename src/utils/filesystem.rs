use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Ensure a directory exists, creating parent directories if necessary.
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))
}

/// Safely remove a file or directory tree if it exists.
pub fn safe_remove(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to read metadata for: {}", path.display()))?;

    if meta.file_type().is_symlink() || meta.is_file() {
        fs::remove_file(path)
            .with_context(|| format!("Failed to remove file/symlink: {}", path.display()))?;
    } else {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    }

    Ok(())
}
