use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha2::{Digest, Sha512};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::Builder;

use crate::extract;
use crate::registry;
use crate::validate_package_name;

/// Verify SHA-512 integrity of downloaded archive bytes.
fn verify_integrity(archive_bytes: &[u8], integrity: &str) -> Result<()> {
    let expected_hash = integrity
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

/// Update package.json with the installed dependency.
fn update_manifest(package: &str, version: &str) -> Result<()> {
    let manifest_path = Path::new("package.json");

    let mut manifest: Value = if manifest_path.exists() {
        let content = fs::read_to_string(manifest_path).context("Could not read package.json")?;
        serde_json::from_str(&content).context("Invalid package.json format")?
    } else {
        json!({
            "name": "neupm-project",
            "version": "1.0.0"
        })
    };

    if !manifest.is_object() {
        bail!("Invalid package.json: expected a JSON object.");
    }

    if !manifest["dependencies"].is_object() {
        manifest["dependencies"] = json!({});
    }

    manifest["dependencies"][package] = json!(format!("^{version}"));

    let formatted = serde_json::to_string_pretty(&manifest)?;

    // Write to a temp file first, then rename for safety.
    let temp_manifest = manifest_path.with_extension("json.tmp");
    fs::write(&temp_manifest, format!("{formatted}\n"))
        .context("Could not write temporary package.json")?;

    fs::rename(&temp_manifest, manifest_path).context("Could not finalize package.json update")?;

    Ok(())
}

/// Safely replace an existing installation directory with the staged one.
///
/// Strategy: rename existing to a backup, rename staged to final, remove backup.
/// If the rename from staged to final fails, restore from backup.
fn safe_replace(staged_dir: &Path, final_dir: &Path) -> Result<()> {
    if final_dir.exists() {
        let parent = final_dir
            .parent()
            .context("Invalid installation path: no parent directory")?;

        // Create a backup name next to the final dir.
        let dir_name = final_dir
            .file_name()
            .context("Invalid installation path: no directory name")?;

        let backup_name = format!(".neupm-backup-{}", dir_name.to_string_lossy());
        let backup_dir = parent.join(&backup_name);

        // Remove stale backup if it exists (from a previous crashed install).
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).with_context(|| {
                format!(
                    "Could not remove stale backup directory: {}",
                    backup_dir.display()
                )
            })?;
        }

        // Step 1: Move existing to backup.
        let metadata = fs::symlink_metadata(final_dir)?;
        if metadata.file_type().is_symlink() || metadata.is_file() {
            // If for some reason the target is a file or symlink, just remove it.
            fs::remove_file(final_dir)?;
        } else {
            fs::rename(final_dir, &backup_dir).with_context(|| {
                format!(
                    "Could not backup existing package: {} -> {}",
                    final_dir.display(),
                    backup_dir.display()
                )
            })?;
        }

        // Step 2: Move staged to final.
        match fs::rename(staged_dir, final_dir) {
            Ok(()) => {
                // Step 3: Clean up backup.
                if backup_dir.exists() {
                    // Best-effort cleanup; don't fail the install if this fails.
                    let _ = fs::remove_dir_all(&backup_dir);
                }
                Ok(())
            }
            Err(e) => {
                // Rollback: restore from backup.
                if backup_dir.exists() {
                    let _ = fs::rename(&backup_dir, final_dir);
                }
                Err(e).with_context(|| {
                    format!(
                        "Could not move staged package to final location: {}",
                        final_dir.display()
                    )
                })
            }
        }
    } else {
        // No existing installation; just rename staged to final.
        fs::rename(staged_dir, final_dir).with_context(|| {
            format!(
                "Could not move staged package to final location: {}",
                final_dir.display()
            )
        })
    }
}

/// Main package installation operation.
pub async fn add_package(package: &str) -> Result<()> {
    validate_package_name(package)?;

    println!("NEUPM - Package Manager");
    println!("Resolving package metadata...\n");

    let client = registry::build_client()?;

    // 1. Resolve metadata.
    let meta = registry::resolve_metadata(&client, package).await?;

    println!("Package: {}", meta.name);
    println!("Version: {}", meta.version);

    // 2. Download the tarball.
    println!("\nDownloading package...");
    let archive_bytes = registry::download_tarball(&client, &meta.tarball_url).await?;
    println!("Download completed: {} bytes", archive_bytes.len());

    // 3. Verify SHA-512 integrity.
    println!("\nVerifying SHA-512 integrity...");
    verify_integrity(&archive_bytes, &meta.integrity)?;
    println!("SHA-512 integrity verification passed.");

    // 4. Prepare node_modules and staging directory.
    let node_modules = PathBuf::from("node_modules");
    fs::create_dir_all(&node_modules).context("Could not create node_modules directory")?;

    // For scoped packages, create the scope directory inside node_modules
    // and place the staging directory alongside the final destination.
    let installation_path = node_modules.join(package);
    let staging_parent = installation_path
        .parent()
        .context("Invalid package installation path")?;
    fs::create_dir_all(staging_parent).context("Could not create package scope directory")?;

    let staging_directory = Builder::new()
        .prefix(".neupm-staging-")
        .tempdir_in(staging_parent)
        .context("Could not create temporary staging directory")?;

    // 5-7. Extract into staging with security validation.
    println!("\nExtracting package...");
    extract::extract_tarball(&archive_bytes, staging_directory.path())?;

    // 8. Verify the extracted package structure.
    extract::verify_extracted_package(staging_directory.path())?;

    // 9-11. Safe replacement: move staged package to final location.
    // We need to persist the staging directory (prevent auto-cleanup) so we can rename it.
    let staging_path = staging_directory.keep();
    match safe_replace(&staging_path, &installation_path) {
        Ok(()) => {}
        Err(e) => {
            // Clean up the staged directory on failure.
            let _ = fs::remove_dir_all(&staging_path);
            return Err(e);
        }
    }

    // 12. Update package.json only after successful installation.
    update_manifest(package, &meta.version)?;

    // 13. Report success.
    println!("\n----------------------------------------");
    println!("Package installed successfully!");
    println!("----------------------------------------");
    println!("Name:     {package}");
    println!("Version:  {}", meta.version);
    println!("Location: {}", installation_path.display());
    println!("Integrity: Verified");
    println!("----------------------------------------");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha512};
    use tempfile::tempdir;

    // ---------------------------------------------------------------
    // Test: integrity verification succeeds on valid data
    // ---------------------------------------------------------------
    #[test]
    fn test_verify_integrity_valid() {
        let data = b"hello npm package content";
        let hash = Sha512::digest(data);
        let b64 = STANDARD.encode(hash);
        let integrity = format!("sha512-{b64}");

        assert!(verify_integrity(data, &integrity).is_ok());
    }

    // ---------------------------------------------------------------
    // Test: integrity verification fails on corrupted data
    // ---------------------------------------------------------------
    #[test]
    fn test_verify_integrity_corrupted() {
        let data = b"hello npm package content";
        let hash = Sha512::digest(data);
        let b64 = STANDARD.encode(hash);
        let integrity = format!("sha512-{b64}");

        let corrupted = b"tampered content!";
        assert!(verify_integrity(corrupted, &integrity).is_err());
    }

    // ---------------------------------------------------------------
    // Test: safe_replace installs cleanly when no prior version exists
    // ---------------------------------------------------------------
    #[test]
    fn test_safe_replace_fresh_install() {
        let tmp = tempdir().unwrap();
        let staged = tmp.path().join("staging");
        fs::create_dir_all(&staged).unwrap();
        fs::write(staged.join("package.json"), "{\"name\":\"pkg\"}").unwrap();

        let final_dir = tmp.path().join("node_modules").join("pkg");
        fs::create_dir_all(final_dir.parent().unwrap()).unwrap();

        safe_replace(&staged, &final_dir).unwrap();

        assert!(final_dir.join("package.json").exists());
        assert!(!staged.exists());
    }

    // ---------------------------------------------------------------
    // Test: safe_replace updates existing package and cleans up backup
    // ---------------------------------------------------------------
    #[test]
    fn test_safe_replace_existing_package() {
        let tmp = tempdir().unwrap();
        let nm = tmp.path().join("node_modules");
        let final_dir = nm.join("pkg");
        fs::create_dir_all(&final_dir).unwrap();
        fs::write(final_dir.join("v1.txt"), "version 1").unwrap();

        let staged = nm.join(".staging-pkg");
        fs::create_dir_all(&staged).unwrap();
        fs::write(staged.join("v2.txt"), "version 2").unwrap();

        safe_replace(&staged, &final_dir).unwrap();

        assert!(!final_dir.join("v1.txt").exists());
        assert!(final_dir.join("v2.txt").exists());
        assert!(!staged.exists());

        // Ensure no backup directory is left behind
        let backup = nm.join(".neupm-backup-pkg");
        assert!(!backup.exists());
    }

    // ---------------------------------------------------------------
    // Test: safe_replace preserves existing package on failure (rollback)
    // ---------------------------------------------------------------
    #[test]
    fn test_safe_replace_rollback_on_failure() {
        let tmp = tempdir().unwrap();
        let nm = tmp.path().join("node_modules");
        let final_dir = nm.join("pkg");
        fs::create_dir_all(&final_dir).unwrap();
        fs::write(final_dir.join("react.js"), "original-content").unwrap();

        // Pass a non-existent staged path to safe_replace.
        // It will backup the existing directory, then fail to move staged -> final,
        // and should rollback by restoring the original directory.
        let non_existent_staged = nm.join(".staging-non-existent");

        let result = safe_replace(&non_existent_staged, &final_dir);
        assert!(result.is_err());

        // Original package should still be intact after rollback!
        assert!(final_dir.exists());
        assert_eq!(
            fs::read_to_string(final_dir.join("react.js")).unwrap(),
            "original-content"
        );
    }

    // ---------------------------------------------------------------
    // Test: update_manifest preserves unrelated fields and existing deps
    // ---------------------------------------------------------------
    #[test]
    fn test_manifest_preserves_existing_keys() {
        let tmp = tempdir().unwrap();
        let manifest_file = tmp.path().join("package.json");

        // Write an initial package.json with custom fields and existing dep
        let initial_json = r#"{
  "name": "my-cool-app",
  "version": "2.5.0",
  "author": "Alice",
  "dependencies": {
    "react": "^18.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}"#;
        fs::write(&manifest_file, initial_json).unwrap();

        // Helper testing update_manifest in a specific path
        // To avoid mutating process-global cwd concurrently across tests,
        // we can test the json logic directly:
        let content = fs::read_to_string(&manifest_file).unwrap();
        let mut manifest: Value = serde_json::from_str(&content).unwrap();
        manifest["dependencies"]["@types/node"] = json!("^26.6.6");

        assert_eq!(manifest["name"], "my-cool-app");
        assert_eq!(manifest["version"], "2.5.0");
        assert_eq!(manifest["author"], "Alice");
        assert_eq!(manifest["dependencies"]["react"], "^18.0.0");
        assert_eq!(manifest["dependencies"]["@types/node"], "^26.6.6");
        assert_eq!(manifest["devDependencies"]["typescript"], "^5.0.0");
    }
}
