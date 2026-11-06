//! Data model for the NEUPM lockfile.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root lockfile document written to `neupm-lock.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lockfile {
    /// Lockfile format version. Must be 1 for this implementation.
    pub lockfile_version: u32,

    /// Packages recorded in the lockfile, keyed by package name.
    pub packages: HashMap<String, LockEntry>,
}

impl Lockfile {
    /// Create a new empty lockfile.
    pub fn new() -> Self {
        Self {
            lockfile_version: 1,
            packages: HashMap::new(),
        }
    }

    /// Add or overwrite a lockfile entry.
    pub fn insert(&mut self, entry: LockEntry) {
        self.packages.insert(entry.name.clone(), entry);
    }

    /// Retrieve an entry by package name.
    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        self.packages.get(name)
    }

    /// All package names recorded in the lockfile.
    pub fn package_names(&self) -> impl Iterator<Item = &str> {
        self.packages.keys().map(|s| s.as_str())
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// One package entry in the lockfile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockEntry {
    /// Full package name (e.g., `react` or `@types/node`).
    pub name: String,
    /// Concrete resolved version.
    pub version: String,
    /// Tarball download URL.
    pub resolved: String,
    /// npm integrity string (e.g., `sha512-...`).
    pub integrity: String,
    /// Whether this is a direct (user-requested) dependency.
    #[serde(default)]
    pub is_direct: bool,
    /// Runtime dependency requirements as recorded in the package's own `package.json`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,
}

impl LockEntry {
    /// Build a `LockEntry` from resolved package metadata.
    pub fn from_metadata(meta: &crate::core::package::PackageMetadata, is_direct: bool) -> Self {
        Self {
            name: meta.name.clone(),
            version: meta.version.clone(),
            resolved: meta.tarball_url.clone(),
            integrity: meta.integrity.clone(),
            is_direct,
            dependencies: meta.dependencies.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_lockfile_insert_and_get() {
        let mut lf = Lockfile::new();
        let entry = LockEntry {
            name: "react".to_string(),
            version: "19.0.0".to_string(),
            resolved: "https://example.com/react.tgz".to_string(),
            integrity: "sha512-abc".to_string(),
            is_direct: true,
            dependencies: HashMap::new(),
        };
        lf.insert(entry.clone());
        assert_eq!(lf.get("react"), Some(&entry));
    }

    #[test]
    fn test_lockfile_serialization_round_trip() {
        let mut lf = Lockfile::new();
        lf.insert(LockEntry {
            name: "@types/node".to_string(),
            version: "22.0.0".to_string(),
            resolved: "https://example.com/types-node.tgz".to_string(),
            integrity: "sha512-xyz".to_string(),
            is_direct: false,
            dependencies: HashMap::new(),
        });

        let json = serde_json::to_string_pretty(&lf).unwrap();
        let parsed: Lockfile = serde_json::from_str(&json).unwrap();
        assert_eq!(lf, parsed);
    }

    #[test]
    fn test_lockfile_version_is_1() {
        let lf = Lockfile::new();
        assert_eq!(lf.lockfile_version, 1);
    }
}
