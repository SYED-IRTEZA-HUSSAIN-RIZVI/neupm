use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tempfile::Builder;

use super::replacement::safe_replace;
use crate::archive::{extract_tarball, verify_extracted_package};
use crate::core::context::ExecutionContext;
use crate::core::package::PackageMetadata;
use crate::manifest::update_dependency_in_manifest;
use crate::security::integrity::verify_sha512_integrity;

/// Installer engine coordinating package staging, verification, and replacement.
pub struct InstallEngine {
    context: ExecutionContext,
}

impl InstallEngine {
    pub fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Install package files into `node_modules` without modifying `package.json`.
    pub fn install_package_files(
        &self,
        package_meta: &PackageMetadata,
        archive_bytes: &[u8],
    ) -> Result<PathBuf> {
        // 1. Verify integrity before extraction.
        verify_sha512_integrity(archive_bytes, &package_meta.integrity)?;

        // 2. Prepare node_modules and target destination directory.
        let node_modules = self.context.node_modules();
        fs::create_dir_all(node_modules).context("Could not create node_modules directory")?;

        let installation_path = node_modules.join(&package_meta.name);
        let parent_dir = installation_path
            .parent()
            .context("Invalid package installation path")?;
        fs::create_dir_all(parent_dir).context("Could not create package directory")?;

        // 3. Create staging directory on the same filesystem/drive for atomic rename.
        let staging_temp = Builder::new()
            .prefix(".neupm-staging-")
            .tempdir_in(parent_dir)
            .context("Could not create temporary staging directory")?;

        // 4. Extract package into staging directory.
        extract_tarball(archive_bytes, staging_temp.path())?;

        // 5. Verify extracted package structure and package.json.
        verify_extracted_package(staging_temp.path())?;

        // 6. Atomically replace target installation with rollback protection.
        let staging_path = staging_temp.keep();
        match safe_replace(&staging_path, &installation_path) {
            Ok(()) => {}
            Err(e) => {
                let _ = fs::remove_dir_all(&staging_path);
                return Err(e);
            }
        }

        Ok(installation_path)
    }

    /// Execute the complete staged installation of a package from tarball bytes,
    /// including updating `package.json`.
    pub fn install_package(
        &self,
        package_meta: &PackageMetadata,
        archive_bytes: &[u8],
    ) -> Result<PathBuf> {
        let installation_path = self.install_package_files(package_meta, archive_bytes)?;

        // Update project package.json after successful installation.
        update_dependency_in_manifest(
            self.context.manifest(),
            &package_meta.name,
            &package_meta.version,
        )?;

        Ok(installation_path)
    }
}
