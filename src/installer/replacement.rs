use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::rollback::restore_backup;

/// Safely replace an existing installation directory with the staged directory.
///
/// Renames existing target to `.neupm-backup-*`, moves staged to target,
/// and automatically restores backup if staged move encounters an error.
pub fn safe_replace(staged_dir: &Path, final_dir: &Path) -> Result<()> {
    if final_dir.exists() {
        let parent = final_dir
            .parent()
            .context("Invalid installation path: no parent directory")?;

        let dir_name = final_dir
            .file_name()
            .context("Invalid installation path: no directory name")?;

        let backup_name = format!(".neupm-backup-{}", dir_name.to_string_lossy());
        let backup_dir = parent.join(&backup_name);

        if backup_dir.exists() {
            let _ = fs::remove_dir_all(&backup_dir);
        }

        // Step 1: Backup existing package.
        let metadata = fs::symlink_metadata(final_dir)?;
        if metadata.file_type().is_symlink() || metadata.is_file() {
            fs::remove_file(final_dir)?;
        } else {
            fs::rename(final_dir, &backup_dir).with_context(|| {
                format!(
                    "Could not backup existing package: {} -> {}",
                    final_dir.display(),
                    backup_dir.display()
                )
            })?;
        }

        // Step 2: Move staged directory into place.
        match fs::rename(staged_dir, final_dir) {
            Ok(()) => {
                // Step 3: Cleanup backup.
                if backup_dir.exists() {
                    let _ = fs::remove_dir_all(&backup_dir);
                }
                Ok(())
            }
            Err(e) => {
                // Rollback: restore backup.
                let _ = restore_backup(&backup_dir, final_dir);
                Err(e).with_context(|| {
                    format!(
                        "Could not move staged package to final location: {}",
                        final_dir.display()
                    )
                })
            }
        }
    } else {
        fs::rename(staged_dir, final_dir).with_context(|| {
            format!(
                "Could not move staged package to final location: {}",
                final_dir.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_safe_replace_fresh() {
        let tmp = tempdir().unwrap();
        let staged = tmp.path().join("staging");
        fs::create_dir_all(&staged).unwrap();
        fs::write(staged.join("package.json"), "{\"name\":\"pkg\"}").unwrap();

        let final_dir = tmp.path().join("node_modules").join("pkg");
        fs::create_dir_all(final_dir.parent().unwrap()).unwrap();

        safe_replace(&staged, &final_dir).unwrap();

        assert!(final_dir.join("package.json").exists());
        assert!(!staged.exists());
    }

    #[test]
    fn test_safe_replace_existing() {
        let tmp = tempdir().unwrap();
        let nm = tmp.path().join("node_modules");
        let final_dir = nm.join("pkg");
        fs::create_dir_all(&final_dir).unwrap();
        fs::write(final_dir.join("v1.txt"), "v1").unwrap();

        let staged = nm.join(".staging");
        fs::create_dir_all(&staged).unwrap();
        fs::write(staged.join("v2.txt"), "v2").unwrap();

        safe_replace(&staged, &final_dir).unwrap();

        assert!(!final_dir.join("v1.txt").exists());
        assert!(final_dir.join("v2.txt").exists());
        assert!(!nm.join(".neupm-backup-pkg").exists());
    }

    #[test]
    fn test_safe_replace_rollback() {
        let tmp = tempdir().unwrap();
        let nm = tmp.path().join("node_modules");
        let final_dir = nm.join("pkg");
        fs::create_dir_all(&final_dir).unwrap();
        fs::write(final_dir.join("react.js"), "preserved-content").unwrap();

        let non_existent_staged = nm.join(".non-existent");
        let result = safe_replace(&non_existent_staged, &final_dir);
        assert!(result.is_err());

        assert!(final_dir.exists());
        assert_eq!(
            fs::read_to_string(final_dir.join("react.js")).unwrap(),
            "preserved-content"
        );
    }
}
