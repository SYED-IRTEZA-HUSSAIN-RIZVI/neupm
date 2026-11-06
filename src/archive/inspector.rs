use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// Verify that an extracted package directory contains a valid package.json.
pub fn verify_extracted_package(staging_dir: &Path) -> Result<()> {
    let pkg_json_path = staging_dir.join("package.json");

    if !pkg_json_path.exists() {
        bail!("Invalid package: package.json is missing from the extracted archive.");
    }

    let content =
        fs::read_to_string(&pkg_json_path).context("Failed to read extracted package.json")?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).context("Extracted package.json is not valid JSON")?;

    if !parsed.is_object() {
        bail!("Extracted package.json is not a JSON object.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_verify_extracted_package_valid() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{\"name\":\"test\"}").unwrap();
        assert!(verify_extracted_package(dir.path()).is_ok());
    }

    #[test]
    fn test_verify_extracted_package_missing() {
        let dir = tempdir().unwrap();
        assert!(verify_extracted_package(dir.path()).is_err());
    }

    #[test]
    fn test_verify_extracted_package_invalid_json() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "not json").unwrap();
        assert!(verify_extracted_package(dir.path()).is_err());
    }
}
