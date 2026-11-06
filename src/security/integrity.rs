use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha512};

/// Verify SHA-512 integrity of downloaded archive bytes against npm integrity string.
pub fn verify_sha512_integrity(archive_bytes: &[u8], integrity_str: &str) -> Result<()> {
    let expected_hash = integrity_str
        .split_whitespace()
        .find_map(|entry| entry.strip_prefix("sha512-"))
        .context("SHA-512 integrity hash not provided")?;

    let expected_bytes = STANDARD
        .decode(expected_hash)
        .context("Invalid Base64 integrity hash")?;

    let actual_hash = Sha512::digest(archive_bytes);
    let actual_bytes = actual_hash.as_slice();

    if actual_bytes != expected_bytes.as_slice() {
        bail!("Package integrity verification failed.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_integrity() {
        let data = b"sample npm package content";
        let hash = Sha512::digest(data);
        let b64 = STANDARD.encode(hash);
        let integrity = format!("sha512-{b64}");

        assert!(verify_sha512_integrity(data, &integrity).is_ok());
    }

    #[test]
    fn test_tampered_integrity() {
        let data = b"sample npm package content";
        let hash = Sha512::digest(data);
        let b64 = STANDARD.encode(hash);
        let integrity = format!("sha512-{b64}");

        assert!(verify_sha512_integrity(b"tampered content", &integrity).is_err());
    }
}
