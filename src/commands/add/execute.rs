use anyhow::Result;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::installer::InstallEngine;
use crate::registry::build_http_client;
use crate::registry::download_tarball;
use crate::registry::npm::NpmRegistryAdapter;
use crate::utils::validation::validate_package_name;

/// Execute the `add` command: validates package, resolves metadata, downloads tarball,
/// verifies integrity, extracts into staging, safely replaces target, and updates package.json.
pub async fn execute_add(package: &str, context: ExecutionContext) -> Result<()> {
    validate_package_name(package)?;

    output::print_banner();
    output::print_resolving();

    let client = build_http_client()?;
    let adapter = NpmRegistryAdapter::default();

    // 1. Resolve metadata from npm registry.
    let meta = adapter.resolve(&client, package).await?;
    output::print_package_info(&meta.name, &meta.version);

    // 2. Download package tarball.
    output::print_downloading();
    let archive_bytes = download_tarball(&client, &meta.tarball_url).await?;
    output::print_download_completed(archive_bytes.len());

    // 3. Verify integrity.
    output::print_verifying_integrity();
    // (Note: verify_sha512_integrity is also called within InstallEngine).
    crate::security::integrity::verify_sha512_integrity(&archive_bytes, &meta.integrity)?;
    output::print_integrity_passed();

    // 4. Staged extraction and atomic replacement.
    output::print_extracting();
    let engine = InstallEngine::new(context);
    let installed_path = engine.install_package(&meta, &archive_bytes)?;

    // 5. Display success confirmation.
    output::print_install_success(&meta.name, &meta.version, &installed_path);

    Ok(())
}
