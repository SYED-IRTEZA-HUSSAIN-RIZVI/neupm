use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Safely write data to a file by writing to a temporary file first,
/// then renaming it atomically.
pub fn atomic_write(target_path: &Path, content: &str) -> Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));

    let temp_path = parent.join(format!(
        ".{}.tmp-{}",
        target_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "atomic".to_string()),
        std::process::id()
    ));

    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write temporary file: {}", temp_path.display()))?;

    fs::rename(&temp_path, target_path).with_context(|| {
        format!(
            "Failed to atomically rename {} to {}",
            temp_path.display(),
            target_path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write_creates_and_overwrites() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("test.json");

        atomic_write(&target, "{\"initial\":true}").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "{\"initial\":true}");

        atomic_write(&target, "{\"updated\":true}").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "{\"updated\":true}");
    }
}
