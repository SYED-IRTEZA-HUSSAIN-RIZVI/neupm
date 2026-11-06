use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde_json::Value;

const REGISTRY_URL: &str = "https://registry.npmjs.org";

/// Metadata resolved from the npm registry for a package.
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub integrity: String,
}

/// Build a reusable HTTP client with NEUPM user agent.
pub fn build_client() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("neupm/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to initialize HTTP client")
}

/// Resolve package metadata (latest version, tarball URL, integrity) from npm registry.
pub async fn resolve_metadata(client: &Client, package: &str) -> Result<PackageMetadata> {
    // Encode scoped package names for the URL (e.g., @types/node -> @types%2fnode).
    let encoded_package = package.replace('/', "%2f");
    let metadata_url = format!("{REGISTRY_URL}/{encoded_package}");

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

    let version = metadata["dist-tags"]["latest"]
        .as_str()
        .context("Latest package version is unavailable")?
        .to_string();

    let version_data = &metadata["versions"][&version];

    let tarball_url = version_data["dist"]["tarball"]
        .as_str()
        .context("Package tarball URL is unavailable")?
        .to_string();

    let integrity = version_data["dist"]["integrity"]
        .as_str()
        .context("Package integrity information unavailable")?
        .to_string();

    Ok(PackageMetadata {
        name: package.to_string(),
        version,
        tarball_url,
        integrity,
    })
}

/// Download the package tarball bytes.
pub async fn download_tarball(client: &Client, tarball_url: &str) -> Result<Vec<u8>> {
    let response = client
        .get(tarball_url)
        .send()
        .await
        .context("Package download failed")?
        .error_for_status()
        .context("Package server returned an unsuccessful response")?;

    let archive_bytes = response
        .bytes()
        .await
        .context("Could not read package archive")?;

    Ok(archive_bytes.to_vec())
}
