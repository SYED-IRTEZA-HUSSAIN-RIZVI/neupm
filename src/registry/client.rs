use anyhow::{Context, Result};
use reqwest::Client;

/// Build a configured HTTP client for registry communication.
pub fn build_http_client() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("neupm/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to initialize HTTP client")
}
