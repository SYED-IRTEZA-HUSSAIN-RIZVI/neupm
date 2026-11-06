//! Lockfile writer: serialize and write `neupm-lock.json` atomically.

use anyhow::{Context, Result};
use std::path::Path;

use super::model::Lockfile;
use crate::utils::atomic_write::atomic_write;

/// Serialize and atomically write the lockfile to the given path.
pub fn write_lockfile(path: &Path, lockfile: &Lockfile) -> Result<()> {
    let content = serde_json::to_string_pretty(lockfile).context("Could not serialize lockfile")?;
    let content_with_newline = format!("{content}\n");
    atomic_write(path, content_with_newline.as_bytes())
        .with_context(|| format!("Could not write lockfile: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::model::{LockEntry, Lockfile};
    use crate::lockfile::reader::read_lockfile;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("neupm-lock.json");

        let mut lf = Lockfile::new();
        lf.insert(LockEntry {
            name: "lodash".to_string(),
            version: "4.17.21".to_string(),
            resolved: "https://example.com/lodash.tgz".to_string(),
            integrity: "sha512-zzz".to_string(),
            is_direct: true,
            dependencies: HashMap::new(),
        });

        write_lockfile(&path, &lf).unwrap();

        let loaded = read_lockfile(&path).unwrap().unwrap();
        assert_eq!(loaded, lf);
    }

    #[test]
    fn test_write_creates_file_with_trailing_newline() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("neupm-lock.json");
        write_lockfile(&path, &Lockfile::new()).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.ends_with('\n'));
    }
}
