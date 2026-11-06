use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use tar::Archive;

use super::archive_root::find_archive_root;
use super::path_validator::validate_entry_path;

/// Securely extract an npm tarball into the given staging directory.
///
/// Disallows path traversal, absolute paths, drive letters, UNC paths,
/// symbolic links, and hard links.
pub fn extract_tarball(archive_bytes: &[u8], staging_dir: &Path) -> Result<()> {
    // Pass 1: Identify root prefix.
    let root_prefix =
        find_archive_root(archive_bytes).context("Failed to identify archive root directory")?;

    // Pass 2: Extract files, stripping the root directory prefix.
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(decoder);

    for entry_result in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let mut entry = entry_result.context("Failed to read archive entry")?;

        let original_path = entry
            .path()
            .context("Failed to read entry path")?
            .to_path_buf();

        // Validate the raw path from the archive.
        validate_entry_path(&original_path)?;

        // Reject symbolic links and hard links.
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            bail!(
                "Malicious archive entry: symbolic or hard link detected: {}",
                original_path.display()
            );
        }

        // Strip the top-level prefix directory.
        let stripped_path = match original_path.strip_prefix(&root_prefix) {
            Ok(p) => p.to_path_buf(),
            Err(_) => continue,
        };

        if stripped_path.as_os_str().is_empty() {
            continue;
        }

        validate_entry_path(&stripped_path)?;

        let dest_path = staging_dir.join(&stripped_path);

        let canonical_staging =
            fs::canonicalize(staging_dir).context("Failed to canonicalize staging directory")?;

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;

            let canonical_parent = fs::canonicalize(parent)
                .with_context(|| format!("Failed to canonicalize: {}", parent.display()))?;

            if !canonical_parent.starts_with(&canonical_staging) {
                bail!(
                    "Malicious archive entry: extraction would escape staging directory: {}",
                    original_path.display()
                );
            }
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("Failed to create directory: {}", dest_path.display()))?;
        } else if entry.header().entry_type().is_file() {
            let mut contents = Vec::new();
            entry
                .read_to_end(&mut contents)
                .with_context(|| format!("Failed to read entry: {}", original_path.display()))?;

            fs::write(&dest_path, &contents)
                .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;
        }
    }

    Ok(())
}
