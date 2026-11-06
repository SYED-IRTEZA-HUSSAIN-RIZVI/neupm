use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::package::PackageMetadata;

/// Parse raw npm registry JSON response into PackageMetadata for the `latest` version.
pub fn parse_npm_latest_metadata(package_name: &str, metadata: &Value) -> Result<PackageMetadata> {
    let version = metadata["dist-tags"]["latest"]
        .as_str()
        .context("Latest package version is unavailable")?;
    extract_version_metadata(package_name, metadata, version)
}

/// Extract PackageMetadata for a specific version from the full npm registry document.
pub fn extract_version_metadata(
    package_name: &str,
    metadata: &Value,
    version: &str,
) -> Result<PackageMetadata> {
    let version_data = &metadata["versions"][version];

    if version_data.is_null() {
        anyhow::bail!(
            "Version '{version}' of package '{package_name}' is not available in registry metadata."
        );
    }

    let tarball_url = version_data["dist"]["tarball"]
        .as_str()
        .context("Package tarball URL is unavailable")?
        .to_string();

    let integrity = version_data["dist"]["integrity"]
        .as_str()
        .context("Package integrity information unavailable")?
        .to_string();

    let dependencies = extract_string_map(&version_data["dependencies"]);
    let peer_dependencies = extract_string_map(&version_data["peerDependencies"]);
    let optional_dependencies = extract_string_map(&version_data["optionalDependencies"]);

    Ok(PackageMetadata {
        name: package_name.to_string(),
        version: version.to_string(),
        tarball_url,
        integrity,
        dependencies,
        peer_dependencies,
        optional_dependencies,
    })
}

/// Extract a `{ name: version_range }` map from a JSON value.
/// Returns an empty map if the value is null or not an object.
fn extract_string_map(value: &Value) -> HashMap<String, String> {
    match value.as_object() {
        Some(obj) => obj
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        None => HashMap::new(),
    }
}
