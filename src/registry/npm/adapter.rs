use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde_json::Value;

use crate::core::package::PackageMetadata;
use crate::registry::npm::metadata::extract_version_metadata;
use crate::resolver::version_selector::select_version;

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

    /// Resolve package metadata from npm using an optional version requirement.
    ///
    /// - `version_req = None` → resolves `latest` dist-tag.
    /// - `version_req = Some("latest")` → resolves `latest` dist-tag.
    /// - `version_req = Some("^18.0.0")` → resolves highest matching semver.
    /// - `version_req = Some("18.3.1")` → resolves exact version.
    pub async fn resolve(
        &self,
        client: &Client,
        package: &str,
        version_req: Option<&str>,
    ) -> Result<PackageMetadata> {
        let raw = self.resolve_raw(client, package).await?;
        let version = select_version(&raw, version_req)
            .with_context(|| format!("Could not resolve version for '{package}'"))?;
        extract_version_metadata(package, &raw, &version)
    }

    /// Fetch the raw npm registry JSON for a package (the full metadata document).
    pub async fn resolve_raw(&self, client: &Client, package: &str) -> Result<Value> {
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

        response
            .json()
            .await
            .context("Could not parse npm package metadata")
    }
}
