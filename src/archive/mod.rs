pub mod archive_root;
pub mod extractor;
pub mod inspector;
pub mod path_validator;

pub use archive_root::find_archive_root;
pub use extractor::extract_tarball;
pub use inspector::verify_extracted_package;
pub use path_validator::validate_entry_path;

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::fs;
    use std::io::Write;
    use tar::Builder as TarBuilder;
    use tar::Header;

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

    #[test]
    fn test_extract_standard_package_prefix() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"react\"}"),
            ("package/index.js", b"module.exports = {};"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        extract_tarball(&archive, staging.path()).unwrap();

        assert!(staging.path().join("package.json").exists());
        assert!(staging.path().join("index.js").exists());
    }

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

    #[test]
    fn test_reject_path_traversal() {
        let archive = build_tar_gz_raw(&[
            ("package/package.json", b"{\"name\":\"evil\"}"),
            ("package/../../../etc/passwd", b"root:x:0:0"),
        ]);

        let staging = tempfile::tempdir().unwrap();
        let result = extract_tarball(&archive, staging.path());
        assert!(result.is_err());
    }

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
    }

    #[test]
    fn test_no_files_escape_staging() {
        let archive = build_tar_gz(&[
            ("package/package.json", b"{\"name\":\"safe\"}"),
            ("package/sub/file.js", b"console.log('safe');"),
        ]);

        let parent = tempfile::tempdir().unwrap();
        let staging = parent.path().join("staging");
        fs::create_dir_all(&staging).unwrap();

        extract_tarball(&archive, &staging).unwrap();

        let parent_entries: Vec<_> = fs::read_dir(parent.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert_eq!(parent_entries, vec!["staging"]);
    }
}
