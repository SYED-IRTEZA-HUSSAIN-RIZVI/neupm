//! Lockfile reader: parse `neupm-lock.json` from disk.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::model::Lockfile;

/// Read and parse the lockfile at the given path.
///
/// Returns `Ok(None)` if the file does not exist (no lockfile yet).
/// Returns `Err` if the file exists but cannot be parsed.
pub fn read_lockfile(path: &Path) -> Result<Option<Lockfile>> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Could not read lockfile: {}", path.display()))?;

    let lockfile: Lockfile = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse lockfile: {}", path.display()))?;

    Ok(Some(lockfile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::model::LockEntry;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_read_missing_lockfile_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("neupm-lock.json");
        assert!(read_lockfile(&path).unwrap().is_none());
    }

    #[test]
    fn test_read_valid_lockfile() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("neupm-lock.json");

        let lf = crate::lockfile::model::Lockfile {
            lockfile_version: 1,
            packages: {
                let mut m = HashMap::new();
                m.insert(
                    "react".to_string(),
                    LockEntry {
                        name: "react".to_string(),
                        version: "19.0.0".to_string(),
                        resolved: "https://example.com/react.tgz".to_string(),
                        integrity: "sha512-abc".to_string(),
                        is_direct: true,
                        dependencies: HashMap::new(),
                    },
                );
                m
            },
        };

        fs::write(&path, serde_json::to_string_pretty(&lf).unwrap()).unwrap();
        let loaded = read_lockfile(&path).unwrap().unwrap();
        assert_eq!(loaded, lf);
    }

    #[test]
    fn test_read_invalid_json_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("neupm-lock.json");
        fs::write(&path, "this is not json").unwrap();
        assert!(read_lockfile(&path).is_err());
    }
}
