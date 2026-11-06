use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::collections::HashSet;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path};
use tar::Archive;

/// Inspect a tarball to find the common root prefix directory.
///
/// npm tarballs typically contain a single top-level directory (usually `package/`)
/// under which all files reside. Some packages may use a different name.
/// This function reads through all entries to determine that prefix.
fn find_archive_root(archive_bytes: &[u8]) -> Result<String> {
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(decoder);

    let mut top_level_dirs: HashSet<String> = HashSet::new();

    for entry_result in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let entry = entry_result.context("Failed to read archive entry")?;
        let path = entry
            .path()
            .context("Failed to read entry path")?
            .to_path_buf();

        // Get the first component of the path.
        let first_component = path.components().next().and_then(|c| match c {
            Component::Normal(s) => s.to_str().map(String::from),
            _ => None,
        });

        if let Some(dir_name) = first_component {
            top_level_dirs.insert(dir_name);
        }
    }

    if top_level_dirs.is_empty() {
        bail!("Archive is empty: no entries found.");
    }

    if top_level_dirs.len() != 1 {
        bail!(
            "Invalid npm archive: expected exactly one top-level directory, found {}: {:?}",
            top_level_dirs.len(),
            top_level_dirs
        );
    }

    // Safe: we just confirmed exactly one entry.
    Ok(top_level_dirs.into_iter().next().unwrap())
}

/// Validate a path component to reject path traversal, absolute paths,
/// Windows drive-letter paths, and UNC paths.
fn validate_entry_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Reject absolute paths.
    if path.is_absolute() {
        bail!("Malicious archive entry: absolute path detected: {path_str}");
    }

    // Reject Windows drive-letter paths (e.g., C:\...).
    if path_str.len() >= 2 {
        let bytes = path_str.as_bytes();
        if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            bail!("Malicious archive entry: Windows drive path detected: {path_str}");
        }
    }

    // Reject UNC paths.
    if path_str.starts_with("\\\\") || path_str.starts_with("//") {
        bail!("Malicious archive entry: UNC path detected: {path_str}");
    }

    // Reject parent directory traversal in any component.
    for component in path.components() {
        match component {
            Component::ParentDir => {
                bail!("Malicious archive entry: path traversal detected: {path_str}");
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("Malicious archive entry: absolute path component detected: {path_str}");
            }
            Component::Normal(s) => {
                let s_str = s.to_string_lossy();
                // Reject entries with embedded null bytes.
                if s_str.contains('\0') {
                    bail!("Malicious archive entry: null byte in path: {path_str}");
                }
            }
            Component::CurDir => {
                // "." components are harmless; skip.
            }
        }
    }

    Ok(())
}

/// Securely extract an npm tarball into the given staging directory.
///
/// The tarball is expected to contain a single top-level directory. All files
/// inside it are extracted directly into `staging_dir` (stripping the top-level
/// prefix). For example, `package/index.js` becomes `<staging_dir>/index.js`.
///
/// Security checks performed:
/// - Path traversal (../)
/// - Absolute paths
/// - Windows drive-letter paths
/// - UNC paths
/// - Symbolic links (rejected)
/// - Hard links (rejected)
/// - Null bytes in paths
pub fn extract_tarball(archive_bytes: &[u8], staging_dir: &Path) -> Result<()> {
    // First pass: identify the archive root directory.
    let root_prefix =
        find_archive_root(archive_bytes).context("Failed to identify archive root directory")?;

    // Second pass: extract files, stripping the top-level directory prefix.
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
            Err(_) => {
                // Entry doesn't start with the expected root prefix - skip it.
                // This shouldn't happen in a valid npm tarball, but handle gracefully.
                continue;
            }
        };

        // Skip the root directory entry itself (empty path after stripping).
        if stripped_path.as_os_str().is_empty() {
            continue;
        }

        // Validate the stripped path too.
        validate_entry_path(&stripped_path)?;

        let dest_path = staging_dir.join(&stripped_path);

        // Verify the destination stays within the staging directory.
        let canonical_staging =
            fs::canonicalize(staging_dir).context("Failed to canonicalize staging directory")?;

        // For the dest_path, we canonicalize the parent (which should exist or
        // be created) and verify containment.
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
            // Read the entry contents into memory, then write to disk.
            let mut contents = Vec::new();
            entry
                .read_to_end(&mut contents)
                .with_context(|| format!("Failed to read entry: {}", original_path.display()))?;

            fs::write(&dest_path, &contents)
                .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;
        }
        // Other entry types (pax headers, etc.) are silently skipped.
    }

    Ok(())
}

/// Verify that the extracted package contains a valid package.json.
pub fn verify_extracted_package(staging_dir: &Path) -> Result<()> {
    let pkg_json_path = staging_dir.join("package.json");

    if !pkg_json_path.exists() {
        bail!("Invalid package: package.json is missing from the extracted archive.");
    }

    let content =
        fs::read_to_string(&pkg_json_path).context("Failed to read extracted package.json")?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).context("Extracted package.json is not valid JSON")?;

    if !parsed.is_object() {
        bail!("Extracted package.json is not a JSON object.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tar::Builder as TarBuilder;
    use tar::Header;

    /// Helper: build a gzipped tar archive from a list of (path, content) entries.
    /// All entries are regular files.
    fn build_tar_gz(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_bytes);
            for (path, content) in entries {
                let mut header = Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_entry_type(tar::EntryType::Regular);
                header.set_cksum();
                builder
                    .append_data(&mut header, *path, &content[..])
                    .unwrap();
            }
            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        gz_bytes
    }

    /// Helper: build a gzipped tar with a symlink entry.
    fn build_tar_gz_with_symlink(
        file_entries: &[(&str, &[u8])],
        symlink_path: &str,
        symlink_target: &str,
    ) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_bytes);

            for (path, content) in file_entries {
                let mut header = Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_entry_type(tar::EntryType::Regular);
                header.set_cksum();
                builder
                    .append_data(&mut header, *path, &content[..])
                    .unwrap();
            }

            // Add a symlink entry.
            let mut header = Header::new_gnu();
            header.set_size(0);
            header.set_mode(0o777);
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_cksum();
            builder
                .append_link(&mut header, symlink_path, symlink_target)
                .unwrap();

            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        gz_bytes
    }

    /// Helper: build a gzipped tar with a hard link entry.
    fn build_tar_gz_with_hardlink(
        file_entries: &[(&str, &[u8])],
        hardlink_path: &str,
        hardlink_target: &str,
    ) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_bytes);

            for (path, content) in file_entries {
                let mut header = Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_entry_type(tar::EntryType::Regular);
                header.set_cksum();
                builder
                    .append_data(&mut header, *path, &content[..])
                    .unwrap();
            }

            // Add a hard link entry.
            let mut header = Header::new_gnu();
            header.set_size(0);
            header.set_mode(0o644);
            header.set_entry_type(tar::EntryType::Link);
            header.set_cksum();
            builder
                .append_link(&mut header, hardlink_path, hardlink_target)
                .unwrap();

            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        gz_bytes
    }

    // ---------------------------------------------------------------
    // Test: valid archive with standard "package/" prefix
    // ---------------------------------------------------------------
    #[test]
    fn test_extract_standard_package_prefix() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"test\"}"),
            ("package/index.js", b"module.exports = {};"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        extract_tarball(&archive, staging.path()).unwrap();

        assert!(staging.path().join("package.json").exists());
        assert!(staging.path().join("index.js").exists());

        let content = fs::read_to_string(staging.path().join("package.json")).unwrap();
        assert_eq!(content, "{\"name\":\"test\"}");
    }

    // ---------------------------------------------------------------
    // Test: valid archive with non-standard root name (like @types/node using "node/")
    // ---------------------------------------------------------------
    #[test]
    fn test_extract_non_standard_root_name() {
        let archive = build_tar_gz(&[
            ("node/package.json", b"{\"name\":\"@types/node\"}"),
            ("node/index.d.ts", b"declare module 'node';"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        extract_tarball(&archive, staging.path()).unwrap();

        assert!(staging.path().join("package.json").exists());
        assert!(staging.path().join("index.d.ts").exists());
    }

    // ---------------------------------------------------------------
    // Test: nested directories are preserved
    // ---------------------------------------------------------------
    #[test]
    fn test_extract_nested_directories() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"test\"}"),
            ("package/lib/utils.js", b"exports.util = true;"),
            ("package/lib/deep/nested.js", b"exports.deep = true;"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        extract_tarball(&archive, staging.path()).unwrap();

        assert!(staging.path().join("lib/utils.js").exists());
        assert!(staging.path().join("lib/deep/nested.js").exists());
    }

    /// Helper: build a gzipped tar archive with arbitrary/raw paths (bypassing tar::Builder path validation).
    fn build_tar_gz_raw(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_bytes = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_bytes);
            for (path, content) in entries {
                let mut header = Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                header.set_entry_type(tar::EntryType::Regular);
                let bytes = path.as_bytes();
                header.as_mut_bytes()[..bytes.len()].copy_from_slice(bytes);
                header.set_cksum();
                builder.append(&header, &content[..]).unwrap();
            }
            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        gz_bytes
    }

    // ---------------------------------------------------------------
    // Test: unit tests for validate_entry_path
    // ---------------------------------------------------------------
    #[test]
    fn test_validate_entry_path_rules() {
        assert!(validate_entry_path(Path::new("safe/index.js")).is_ok());
        assert!(validate_entry_path(Path::new("package/nested/dir/file.d.ts")).is_ok());

        // Path traversal
        assert!(validate_entry_path(Path::new("../evil.js")).is_err());
        assert!(validate_entry_path(Path::new("package/../../evil.js")).is_err());

        // Absolute paths
        assert!(validate_entry_path(Path::new("/etc/passwd")).is_err());

        // Windows drive paths
        assert!(validate_entry_path(Path::new("C:\\malicious.js")).is_err());
        assert!(validate_entry_path(Path::new("D:malicious.js")).is_err());

        // UNC paths
        assert!(validate_entry_path(Path::new("\\\\evil-server\\share\\pkg")).is_err());
        assert!(validate_entry_path(Path::new("//evil-server/share/pkg")).is_err());
    }

    // ---------------------------------------------------------------
    // Test: path traversal with ../
    // ---------------------------------------------------------------
    #[test]
    fn test_reject_path_traversal() {
        let archive = build_tar_gz_raw(&[
            ("package/package.json", b"{\"name\":\"evil\"}"),
            ("package/../../../etc/passwd", b"root:x:0:0"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("traversal") || err_msg.contains("Malicious"),
            "Expected path traversal error, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: symlinks are rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_reject_symlinks() {
        let archive = build_tar_gz_with_symlink(
            &[("package/package.json", b"{\"name\":\"evil\"}")],
            "package/evil-link",
            "/etc/passwd",
        );

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("symbolic or hard link"),
            "Expected symlink rejection, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: hard links are rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_reject_hard_links() {
        let archive = build_tar_gz_with_hardlink(
            &[("package/package.json", b"{\"name\":\"evil\"}")],
            "package/evil-hardlink",
            "/etc/shadow",
        );

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("symbolic or hard link"),
            "Expected hard link rejection, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: multiple top-level directories are rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_reject_multiple_top_level_dirs() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"test\"}"),
            ("other-dir/rogue.js", b"console.log('rogue');"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("expected exactly one top-level directory"),
            "Expected multi-root rejection, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: empty archive is rejected
    // ---------------------------------------------------------------
    #[test]
    fn test_reject_empty_archive() {
        let archive = build_tar_gz(&[]);

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("empty") || err_msg.contains("no entries"),
            "Expected empty archive error, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: verify_extracted_package rejects missing package.json
    // ---------------------------------------------------------------
    #[test]
    fn test_verify_rejects_missing_package_json() {
        let staging = tempfile::tempdir().unwrap();
        fs::write(staging.path().join("index.js"), "module.exports = {};").unwrap();

        let result = verify_extracted_package(staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("package.json is missing"),
            "Expected missing package.json error, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: verify_extracted_package rejects invalid JSON
    // ---------------------------------------------------------------
    #[test]
    fn test_verify_rejects_invalid_json() {
        let staging = tempfile::tempdir().unwrap();
        fs::write(staging.path().join("package.json"), "not json {{{").unwrap();

        let result = verify_extracted_package(staging.path());
        assert!(result.is_err());
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("not valid JSON"),
            "Expected invalid JSON error, got: {err_msg}"
        );
    }

    // ---------------------------------------------------------------
    // Test: verify_extracted_package accepts valid package
    // ---------------------------------------------------------------
    #[test]
    fn test_verify_accepts_valid_package() {
        let staging = tempfile::tempdir().unwrap();
        fs::write(
            staging.path().join("package.json"),
            "{\"name\":\"test\",\"version\":\"1.0.0\"}",
        )
        .unwrap();

        let result = verify_extracted_package(staging.path());
        assert!(result.is_ok());
    }

    // ---------------------------------------------------------------
    // Test: no files escape the staging directory
    // ---------------------------------------------------------------
    #[test]
    fn test_no_files_escape_staging() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"test\"}"),
            ("package/sub/file.js", b"ok"),
        ]);

        let parent = tempfile::tempdir().unwrap();
        let staging = parent.path().join("staging");
        fs::create_dir_all(&staging).unwrap();

        extract_tarball(&archive, &staging).unwrap();

        // Only the staging directory should have contents.
        // Verify no files were written to the parent.
        let parent_entries: Vec<_> = fs::read_dir(parent.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert_eq!(
            parent_entries,
            vec!["staging"],
            "Files escaped the staging directory: {parent_entries:?}"
        );
    }
}
