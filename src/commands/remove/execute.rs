//! Implementation of `neupm remove <package>` command.

use anyhow::{Context, Result, bail};
use std::fs;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::lockfile::{read_lockfile, write_lockfile};
use crate::manifest::remove_dependency_from_manifest;
use crate::utils::validation::validate_package_name;

/// Execute the `remove` command.
///
/// 1. Validates the package name.
/// 2. Removes the dependency from `package.json`.
/// 3. Deletes the package directory from `node_modules` (including empty scope directories).
/// 4. Removes the package from `neupm-lock.json`.
pub async fn execute_remove(package: &str, context: ExecutionContext) -> Result<()> {
    validate_package_name(package)?;

    output::print_banner();
    println!("Removing package '{package}'...");

    // 1. Remove from package.json.
    let manifest_path = context.manifest();
    let removed_from_manifest = if manifest_path.exists() {
        remove_dependency_from_manifest(manifest_path, package)
            .context("Failed to update package.json")?
    } else {
        false
    };

    // 2. Remove from node_modules.
    let pkg_dir = context.node_modules().join(package);
    let mut removed_from_disk = false;

    if pkg_dir.exists() {
        fs::remove_dir_all(&pkg_dir).with_context(|| {
            format!("Failed to remove package directory: {}", pkg_dir.display())
        })?;
        removed_from_disk = true;

        // If scoped package, clean up empty scope directory
        if let Some(parent) = pkg_dir.parent()
            && parent != context.node_modules()
            && let Ok(mut entries) = fs::read_dir(parent)
            && entries.next().is_none()
        {
            let _ = fs::remove_dir(parent);
        }
    }

    // 3. Remove from lockfile if present.
    let lockfile_path = context.lockfile();
    if let Some(mut lockfile) = read_lockfile(lockfile_path)?
        && lockfile.packages.remove(package).is_some()
    {
        write_lockfile(lockfile_path, &lockfile)?;
    }

    if !removed_from_manifest && !removed_from_disk {
        bail!("Package '{package}' is not installed or declared in this project.");
    }

    println!("\n----------------------------------------");
    println!("Package removed successfully!");
    println!("----------------------------------------");
    println!("Name:     {package}");
    println!("----------------------------------------");

    Ok(())
}
