//! Implementation of `neupm update` command.

use anyhow::{Context, Result, bail};

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::installer::InstallEngine;
use crate::lockfile::{LockEntry, read_lockfile, write_lockfile};
use crate::manifest::PackageJson;
use crate::registry::build_http_client;
use crate::registry::download_tarball;
use crate::resolver::ResolutionEngine;
use crate::security::integrity::verify_sha512_integrity;

/// Execute the `update` command.
///
/// Re-resolves direct dependencies declared in `package.json` against the registry,
/// respecting declared version constraints, downloads newer versions, and updates
/// `neupm-lock.json`.
pub async fn execute_update(context: ExecutionContext) -> Result<()> {
    output::print_banner();
    println!("Checking for package updates...");

    let manifest_path = context.manifest();
    if !manifest_path.exists() {
        bail!("No package.json found at {}", manifest_path.display());
    }

    let manifest = PackageJson::load_or_default(manifest_path)?;
    let direct_deps = manifest.dependencies();

    if direct_deps.is_empty() {
        println!("No dependencies declared in package.json.");
        return Ok(());
    }

    let mut lockfile = read_lockfile(context.lockfile())?.unwrap_or_default();
    let client = build_http_client()?;
    let resolver = ResolutionEngine::new(&client);
    let engine = InstallEngine::new(context.clone());

    let mut updated_count = 0;

    for (name, range) in &direct_deps {
        let current_version = lockfile.get(name).map(|e| e.version.as_str());

        let resolution = resolver
            .resolve(name, Some(range.as_str()))
            .await
            .with_context(|| format!("Failed to resolve updates for '{name}'"))?;

        let direct_pkg = resolution
            .packages
            .iter()
            .find(|p| p.is_direct)
            .context("Direct package not found in resolution")?;

        let new_version = &direct_pkg.metadata.version;

        if Some(new_version.as_str()) != current_version {
            println!(
                "Updating {name}: {} -> {new_version}",
                current_version.unwrap_or("none")
            );

            // Install all packages in the updated resolution graph
            for resolved in &resolution.packages {
                let meta = &resolved.metadata;
                let archive_bytes = download_tarball(&client, &meta.tarball_url).await?;
                verify_sha512_integrity(&archive_bytes, &meta.integrity)?;
                engine.install_package_files(meta, &archive_bytes)?;
                lockfile.insert(LockEntry::from_metadata(meta, resolved.is_direct));
            }

            updated_count += 1;
        } else {
            println!("{name}@{new_version} is already up to date.");
        }
    }

    write_lockfile(context.lockfile(), &lockfile)?;

    println!(
        "\nUpdate complete: {updated_count} package(s) updated, {} total locked.",
        lockfile.packages.len()
    );

    Ok(())
}
