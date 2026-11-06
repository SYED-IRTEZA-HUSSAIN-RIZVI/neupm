use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Restore a backup directory back to the destination path during rollback.
pub fn restore_backup(backup_dir: &Path, final_dir: &Path) -> Result<()> {
    if backup_dir.exists() {
        fs::rename(backup_dir, final_dir).with_context(|| {
            format!(
                "Failed to restore backup from {} to {}",
                backup_dir.display(),
                final_dir.display()
            )
        })?;
    }
    Ok(())
}
