use anyhow::Result;
use std::path::Path;

use super::package_json::PackageJson;
use crate::utils::atomic_write::atomic_write;

/// Save package.json atomically to disk.
pub fn save_manifest(path: &Path, manifest: &PackageJson) -> Result<()> {
    let content = manifest.to_formatted_string()?;
    atomic_write(path, &content)
}
