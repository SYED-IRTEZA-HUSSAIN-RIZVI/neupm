use anyhow::Result;
use std::path::Path;

use super::package_json::PackageJson;
use super::writer::save_manifest;

/// Update the project's package.json with the given dependency version.
pub fn update_dependency_in_manifest(
    manifest_path: &Path,
    package: &str,
    version: &str,
) -> Result<()> {
    let mut manifest = PackageJson::load_or_default(manifest_path)?;
    manifest.set_dependency(package, &format!("^{version}"));
    save_manifest(manifest_path, &manifest)
}

/// Remove a dependency from the project's package.json.
///
/// Returns `Ok(true)` if the package was found and removed, `Ok(false)` if not present.
pub fn remove_dependency_from_manifest(manifest_path: &Path, package: &str) -> Result<bool> {
    let mut manifest = PackageJson::load_or_default(manifest_path)?;
    let removed = manifest.remove_dependency(package);
    if removed {
        save_manifest(manifest_path, &manifest)?;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_update_dependency_preserves_custom_fields() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("package.json");

        let initial = r#"{
  "name": "my-app",
  "version": "1.2.3",
  "author": "Antigravity",
  "dependencies": {
    "react": "^18.0.0"
  }
}"#;
        fs::write(&path, initial).unwrap();

        update_dependency_in_manifest(&path, "@types/node", "26.6.6").unwrap();

        let updated = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();

        assert_eq!(parsed["name"], "my-app");
        assert_eq!(parsed["author"], "Antigravity");
        assert_eq!(parsed["dependencies"]["react"], "^18.0.0");
        assert_eq!(parsed["dependencies"]["@types/node"], "^26.6.6");
    }

    #[test]
    fn test_remove_dependency() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("package.json");

        let initial = r#"{"name":"app","dependencies":{"react":"^19.0.0","lodash":"^4.0.0"}}"#;
        fs::write(&path, initial).unwrap();

        let removed = remove_dependency_from_manifest(&path, "lodash").unwrap();
        assert!(removed);

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["dependencies"]["lodash"].is_null());
        assert_eq!(parsed["dependencies"]["react"], "^19.0.0");
    }

    #[test]
    fn test_remove_nonexistent_dependency_returns_false() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("package.json");
        fs::write(&path, r#"{"name":"app","dependencies":{}}"#).unwrap();
        let removed = remove_dependency_from_manifest(&path, "nonexistent").unwrap();
        assert!(!removed);
    }
}
