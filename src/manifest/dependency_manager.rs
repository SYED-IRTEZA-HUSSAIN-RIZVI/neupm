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
}
