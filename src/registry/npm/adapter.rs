use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde_json::Value;

use super::metadata::parse_npm_metadata;
use crate::core::package::PackageMetadata;

pub const DEFAULT_REGISTRY_URL: &str = "https://registry.npmjs.org";

/// Adapter for resolving package metadata from the standard npm registry.
pub struct NpmRegistryAdapter {
    registry_url: String,
}

impl Default for NpmRegistryAdapter {
    fn default() -> Self {
        Self::new(DEFAULT_REGISTRY_URL)
    }
}

impl NpmRegistryAdapter {
    pub fn new(registry_url: &str) -> Self {
        Self {
            registry_url: registry_url.trim_end_matches('/').to_string(),
        }
    }

    /// Resolve package metadata (latest version, tarball URL, integrity) from npm.
    pub async fn resolve(&self, client: &Client, package: &str) -> Result<PackageMetadata> {
        let encoded_package = package.replace('/', "%2f");
        let metadata_url = format!("{}/{}", self.registry_url, encoded_package);

        let response = client
            .get(&metadata_url)
            .send()
            .await
            .context("Could not connect to npm registry")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            bail!("Package '{package}' was not found in npm registry.");
        }

        let response = response
            .error_for_status()
            .context("npm registry returned an unsuccessful response")?;

        let metadata: Value = response
            .json()
            .await
            .context("Could not parse npm package metadata")?;

        parse_npm_metadata(package, &metadata)
    }
}
