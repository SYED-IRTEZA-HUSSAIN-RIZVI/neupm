use neupm::core::package::PackageMetadata;
use neupm::lockfile::{
    LockEntry, Lockfile, LockfileValidationResult, read_lockfile,
    validate_lockfile_against_manifest, write_lockfile,
};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_lockfile_creation_and_atomic_persistence() {
    let dir = tempdir().unwrap();
    let lockfile_path = dir.path().join("neupm-lock.json");

    let mut lockfile = Lockfile::new();
    let meta = PackageMetadata {
        name: "react".to_string(),
        version: "18.3.1".to_string(),
        tarball_url: "https://registry.npmjs.org/react/-/react-18.3.1.tgz".to_string(),
        integrity: "sha512-abc123".to_string(),
        dependencies: {
            let mut m = HashMap::new();
            m.insert("loose-envify".to_string(), "^1.1.0".to_string());
            m
        },
        peer_dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
    };

    lockfile.insert(LockEntry::from_metadata(&meta, true));

    // Save
    write_lockfile(&lockfile_path, &lockfile).unwrap();
    assert!(lockfile_path.exists());

    // Read back
    let loaded = read_lockfile(&lockfile_path).unwrap().unwrap();
    assert_eq!(loaded.lockfile_version, 1);
    assert_eq!(loaded.packages.len(), 1);

    let entry = loaded.get("react").unwrap();
    assert_eq!(entry.version, "18.3.1");
    assert!(entry.is_direct);
    assert_eq!(entry.dependencies.get("loose-envify").unwrap(), "^1.1.0");
}

#[test]
fn test_frozen_lockfile_rejects_missing_dependency() {
    let mut lockfile = Lockfile::new();
    lockfile.insert(LockEntry {
        name: "react".to_string(),
        version: "18.3.1".to_string(),
        resolved: "https://example.com/react.tgz".to_string(),
        integrity: "sha512-xxx".to_string(),
        is_direct: true,
        dependencies: HashMap::new(),
    });

    let mut manifest_deps = HashMap::new();
    manifest_deps.insert("react".to_string(), "^18.3.1".to_string());
    manifest_deps.insert("lodash".to_string(), "^4.17.21".to_string()); // Not in lockfile!

    let validation = validate_lockfile_against_manifest(&lockfile, &manifest_deps);
    match validation {
        LockfileValidationResult::MissingPackages(missing) => {
            assert_eq!(missing, vec!["lodash"]);
        }
        _ => panic!("Expected MissingPackages validation result"),
    }
}

#[test]
fn test_frozen_lockfile_accepts_consistent_dependencies() {
    let mut lockfile = Lockfile::new();
    lockfile.insert(LockEntry {
        name: "react".to_string(),
        version: "18.3.1".to_string(),
        resolved: "https://example.com/react.tgz".to_string(),
        integrity: "sha512-xxx".to_string(),
        is_direct: true,
        dependencies: HashMap::new(),
    });
    lockfile.insert(LockEntry {
        name: "@types/node".to_string(),
        version: "26.6.6".to_string(),
        resolved: "https://example.com/node.tgz".to_string(),
        integrity: "sha512-yyy".to_string(),
        is_direct: true,
        dependencies: HashMap::new(),
    });

    let mut manifest_deps = HashMap::new();
    manifest_deps.insert("react".to_string(), "^18.3.1".to_string());
    manifest_deps.insert("@types/node".to_string(), "^26.6.6".to_string());

    assert_eq!(
        validate_lockfile_against_manifest(&lockfile, &manifest_deps),
        LockfileValidationResult::Valid
    );
}
