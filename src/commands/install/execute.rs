//! Implementation of `neupm install` command.

use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::core::package::PackageMetadata;
use crate::installer::InstallEngine;
use crate::lockfile::{
    LockEntry, Lockfile, LockfileValidationResult, read_lockfile,
    validate_lockfile_against_manifest, write_lockfile,
};
use crate::manifest::PackageJson;
use crate::registry::build_http_client;
use crate::registry::download_tarball;
use crate::resolver::ResolutionEngine;
use crate::security::integrity::verify_sha512_integrity;

/// Execute the `install` command.
///
/// If `--frozen-lockfile` is specified:
/// - Verifies that `neupm-lock.json` exists and is consistent with `package.json`.
/// - Installs packages strictly according to the lockfile.
/// - Fails immediately without modifying any files if inconsistent.
///
/// If no lockfile exists or if not frozen:
/// - Reads declared dependencies from `package.json`.
/// - Resolves missing dependencies against the npm registry.
/// - Downloads, verifies, and installs packages.
/// - Writes or updates `neupm-lock.json`.
pub async fn execute_install(frozen_lockfile: bool, context: ExecutionContext) -> Result<()> {
    output::print_banner();

    // 1. Load project package.json.
    let manifest_path = context.manifest();
    if !manifest_path.exists() {
        bail!(
            "No package.json found at {}. Run 'neupm init' to create one.",
            manifest_path.display()
        );
    }

    let manifest = PackageJson::load_or_default(manifest_path)?;
    let direct_deps = manifest.dependencies();

    let lockfile_path = context.lockfile();
    let existing_lockfile = read_lockfile(lockfile_path)?;

    if frozen_lockfile {
        println!("Running installation in frozen-lockfile mode...");

        let lockfile = match existing_lockfile {
            Some(lf) => lf,
            None => {
                bail!(
                    "Frozen installation failed: lockfile '{}' does not exist.",
                    lockfile_path.display()
                );
            }
        };

        // Validate consistency.
        match validate_lockfile_against_manifest(&lockfile, &direct_deps) {
            LockfileValidationResult::Valid => {}
            LockfileValidationResult::MissingPackages(missing) => {
                bail!(
                    "Frozen installation failed: lockfile is out of sync with package.json. \
                     Missing packages: {}",
                    missing.join(", ")
                );
            }
            LockfileValidationResult::UnknownVersion(v) => {
                bail!("Frozen installation failed: unknown lockfile version {v}.");
            }
        }

        // Install all packages from the lockfile without touching lockfile or manifest.
        install_from_lockfile(&lockfile, &context).await?;
        println!("Frozen installation completed successfully.");
        return Ok(());
    }

    // Normal (non-frozen) install mode.
    if direct_deps.is_empty() {
        println!("No dependencies declared in package.json.");
        return Ok(());
    }

    // If an existing valid lockfile covers all dependencies, use it!
    if let Some(ref lockfile) = existing_lockfile
        && let LockfileValidationResult::Valid =
            validate_lockfile_against_manifest(lockfile, &direct_deps)
    {
        println!(
            "Using existing lockfile with {} recorded packages...",
            lockfile.packages.len()
        );
        install_from_lockfile(lockfile, &context).await?;
        println!("Installation completed successfully.");
        return Ok(());
    }

    // Otherwise, resolve dependencies from registry and build/update lockfile.
    println!(
        "Resolving dependencies for {} declared package(s)...",
        direct_deps.len()
    );

    let client = build_http_client()?;
    let resolver = ResolutionEngine::new(&client);
    let engine = InstallEngine::new(context.clone());

    let mut lockfile = existing_lockfile.unwrap_or_default();
    let mut installed_count = 0;

    for (pkg_name, version_req) in &direct_deps {
        println!("Resolving {pkg_name} ({version_req})...");
        let resolution = resolver
            .resolve(pkg_name, Some(version_req.as_str()))
            .await?;

        for resolved in &resolution.packages {
            let meta = &resolved.metadata;
            let target_dir = context.node_modules().join(&meta.name);

            // If already installed at the exact matching version, skip re-download
            let needs_install = if target_dir.join("package.json").exists() {
                match std::fs::read_to_string(target_dir.join("package.json")) {
                    Ok(content) => {
                        let parsed: serde_json::Value =
                            serde_json::from_str(&content).unwrap_or_default();
                        parsed["version"].as_str() != Some(&meta.version)
                    }
                    Err(_) => true,
                }
            } else {
                true
            };

            if needs_install {
                println!("Downloading {}@{}...", meta.name, meta.version);
                let archive_bytes = download_tarball(&client, &meta.tarball_url).await?;
                verify_sha512_integrity(&archive_bytes, &meta.integrity)?;
                engine.install_package_files(meta, &archive_bytes)?;
                installed_count += 1;
            }

            lockfile.insert(LockEntry::from_metadata(meta, resolved.is_direct));
        }
    }

    write_lockfile(context.lockfile(), &lockfile)?;

    println!(
        "\nInstallation completed! {} package(s) installed/verified, {} locked.",
        installed_count,
        lockfile.packages.len()
    );

    Ok(())
}

/// Helper to download, verify, and install all packages recorded in a lockfile.
async fn install_from_lockfile(lockfile: &Lockfile, context: &ExecutionContext) -> Result<()> {
    let client = build_http_client()?;
    let engine = InstallEngine::new(context.clone());

    println!(
        "Installing {} package(s) from lockfile...",
        lockfile.packages.len()
    );

    for (name, entry) in &lockfile.packages {
        let target_dir = context.node_modules().join(name);

        // Check if already installed
        let target_pkg_json = target_dir.join("package.json");
        if target_pkg_json.exists()
            && let Ok(content) = std::fs::read_to_string(&target_pkg_json)
        {
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            if parsed["version"].as_str() == Some(&entry.version) {
                continue; // Already installed correctly
            }
        }

        println!("Downloading {}@{}...", entry.name, entry.version);
        let archive_bytes = download_tarball(&client, &entry.resolved).await?;
        verify_sha512_integrity(&archive_bytes, &entry.integrity)?;

        let meta = PackageMetadata {
            name: entry.name.clone(),
            version: entry.version.clone(),
            tarball_url: entry.resolved.clone(),
            integrity: entry.integrity.clone(),
            dependencies: entry.dependencies.clone(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
        };

        engine.install_package_files(&meta, &archive_bytes)?;
    }

    Ok(())
}
