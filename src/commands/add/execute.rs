use anyhow::Result;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::installer::InstallEngine;
use crate::lockfile::{LockEntry, read_lockfile, write_lockfile};
use crate::manifest::update_dependency_in_manifest;
use crate::registry::build_http_client;
use crate::registry::download_tarball;
use crate::resolver::{ParsedSpec, ResolutionEngine};
use crate::security::integrity::verify_sha512_integrity;
use crate::utils::validation::validate_package_name;

/// Execute the `add` command:
/// 1. Parse package specification (e.g. `react`, `react@18.3.1`, `@types/node@22`).
/// 2. Validate package name.
/// 3. Recursively resolve all transitive dependencies via `ResolutionEngine`.
/// 4. Download and verify every package in the installation plan.
/// 5. Extract and install each package into `node_modules`.
/// 6. Update `neupm-lock.json` with all installed packages.
/// 7. Update `package.json` with the requested direct dependency.
pub async fn execute_add(spec_str: &str, context: ExecutionContext) -> Result<()> {
    // 1. Parse specification into name and version requirement.
    let spec = ParsedSpec::parse(spec_str)?;
    validate_package_name(&spec.name)?;

    output::print_banner();
    output::print_resolving();

    let client = build_http_client()?;
    let engine = InstallEngine::new(context.clone());

    // 2. Resolve complete dependency graph.
    let resolver = ResolutionEngine::new(&client);
    let resolution = resolver
        .resolve(&spec.name, spec.version_req.as_deref())
        .await?;

    // Find the direct package metadata.
    let direct_pkg = resolution
        .packages
        .iter()
        .find(|p| p.is_direct)
        .expect("Direct package must be present in resolution");

    output::print_package_info(&direct_pkg.metadata.name, &direct_pkg.metadata.version);

    if resolution.packages.len() > 1 {
        println!(
            "Resolved {} packages in dependency graph.",
            resolution.packages.len()
        );
    }

    // Load existing lockfile or create fresh.
    let mut lockfile = read_lockfile(context.lockfile())?.unwrap_or_default();

    let mut direct_installed_path = None;

    // 3. Download, verify, and install each package in dependency order.
    for resolved in &resolution.packages {
        let meta = &resolved.metadata;

        output::print_downloading();
        let archive_bytes = download_tarball(&client, &meta.tarball_url).await?;
        output::print_download_completed(archive_bytes.len());

        output::print_verifying_integrity();
        verify_sha512_integrity(&archive_bytes, &meta.integrity)?;
        output::print_integrity_passed();

        output::print_extracting();
        let installed_path = engine.install_package_files(meta, &archive_bytes)?;

        if resolved.is_direct {
            direct_installed_path = Some(installed_path);
        }

        // Record in lockfile.
        lockfile.insert(LockEntry::from_metadata(meta, resolved.is_direct));
    }

    // 4. Atomically write lockfile.
    write_lockfile(context.lockfile(), &lockfile)?;

    // 5. Update project package.json for the direct dependency.
    update_dependency_in_manifest(
        context.manifest(),
        &direct_pkg.metadata.name,
        &direct_pkg.metadata.version,
    )?;

    // 6. Display success confirmation.
    let final_path = direct_installed_path
        .unwrap_or_else(|| context.node_modules().join(&direct_pkg.metadata.name));
    output::print_install_success(
        &direct_pkg.metadata.name,
        &direct_pkg.metadata.version,
        &final_path,
    );

    Ok(())
}
