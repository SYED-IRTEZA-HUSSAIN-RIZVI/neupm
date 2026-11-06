use anyhow::{Context, Result};
use serde_json::Value;

use crate::core::package::PackageMetadata;

/// Parse raw npm registry JSON response into strongly typed PackageMetadata.
pub fn parse_npm_metadata(package_name: &str, metadata: &Value) -> Result<PackageMetadata> {
    let version = metadata["dist-tags"]["latest"]
        .as_str()
        .context("Latest package version is unavailable")?;

    let version_data = &metadata["versions"][version];

    let tarball_url = version_data["dist"]["tarball"]
        .as_str()
        .context("Package tarball URL is unavailable")?
        .to_string();

    let integrity = version_data["dist"]["integrity"]
        .as_str()
        .context("Package integrity information unavailable")?
        .to_string();

    Ok(PackageMetadata {
        name: package_name.to_string(),
        version: version.to_string(),
        tarball_url,
        integrity,
    })
}
