//! Lockfile validation: checks consistency between the lockfile and package.json.

use std::collections::HashMap;

use crate::lockfile::model::Lockfile;

/// Result of validating a lockfile against the current manifest.
#[derive(Debug, Clone, PartialEq)]
pub enum LockfileValidationResult {
    /// Lockfile is consistent with the manifest.
    Valid,
    /// One or more direct dependencies are missing from the lockfile.
    MissingPackages(Vec<String>),
    /// Lockfile format version is unrecognized.
    UnknownVersion(u32),
}

/// Validate a lockfile against the declared dependencies in `package.json`.
///
/// Returns `Valid` if every direct dependency in `manifest_deps` has a
/// corresponding entry in the lockfile.
///
/// Note: version range matching is intentionally loose here — if a package is
/// present in the lockfile it is considered satisfied. Future work: validate
/// that the locked version satisfies the manifest range.
pub fn validate_lockfile_against_manifest(
    lockfile: &Lockfile,
    manifest_deps: &HashMap<String, String>,
) -> LockfileValidationResult {
    if lockfile.lockfile_version != 1 {
        return LockfileValidationResult::UnknownVersion(lockfile.lockfile_version);
    }

    let missing: Vec<String> = manifest_deps
        .keys()
        .filter(|name| !lockfile.packages.contains_key(*name))
        .cloned()
        .collect();

    if missing.is_empty() {
        LockfileValidationResult::Valid
    } else {
        LockfileValidationResult::MissingPackages(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::model::{LockEntry, Lockfile};
    use std::collections::HashMap;

    fn make_entry(name: &str) -> LockEntry {
        LockEntry {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            resolved: "https://example.com/pkg.tgz".to_string(),
            integrity: "sha512-abc".to_string(),
            is_direct: true,
            dependencies: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_when_all_deps_present() {
        let mut lf = Lockfile::new();
        lf.insert(make_entry("react"));
        lf.insert(make_entry("lodash"));

        let mut deps = HashMap::new();
        deps.insert("react".to_string(), "^19.0.0".to_string());
        deps.insert("lodash".to_string(), "^4.0.0".to_string());

        assert_eq!(
            validate_lockfile_against_manifest(&lf, &deps),
            LockfileValidationResult::Valid
        );
    }

    #[test]
    fn test_missing_packages_reported() {
        let mut lf = Lockfile::new();
        lf.insert(make_entry("react"));

        let mut deps = HashMap::new();
        deps.insert("react".to_string(), "^19.0.0".to_string());
        deps.insert("lodash".to_string(), "^4.0.0".to_string());

        match validate_lockfile_against_manifest(&lf, &deps) {
            LockfileValidationResult::MissingPackages(missing) => {
                assert_eq!(missing, vec!["lodash"]);
            }
            other => panic!("Expected MissingPackages, got {other:?}"),
        }
    }

    #[test]
    fn test_unknown_lockfile_version() {
        let lf = Lockfile {
            lockfile_version: 99,
            packages: HashMap::new(),
        };
        assert_eq!(
            validate_lockfile_against_manifest(&lf, &HashMap::new()),
            LockfileValidationResult::UnknownVersion(99)
        );
    }

    #[test]
    fn test_no_manifest_deps_is_valid() {
        let lf = Lockfile::new();
        let deps = HashMap::new();
        assert_eq!(
            validate_lockfile_against_manifest(&lf, &deps),
            LockfileValidationResult::Valid
        );
    }
}
