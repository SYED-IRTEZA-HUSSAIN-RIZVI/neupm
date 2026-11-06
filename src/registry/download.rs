use anyhow::{Context, Result};
use reqwest::Client;

/// Download raw package tarball bytes from the given URL.
pub async fn download_tarball(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client
        .get(url)
        .send()
        .await
        .context("Package download failed")?
        .error_for_status()
        .context("Package server returned an unsuccessful response")?;

    let bytes = response
        .bytes()
        .await
        .context("Could not read package archive")?;

    Ok(bytes.to_vec())
}
