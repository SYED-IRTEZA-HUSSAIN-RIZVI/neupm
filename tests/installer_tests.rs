use base64::{Engine, engine::general_purpose::STANDARD};
use flate2::Compression;
use flate2::write::GzEncoder;
use neupm::core::context::ExecutionContext;
use neupm::core::package::PackageMetadata;
use neupm::installer::InstallEngine;
use sha2::{Digest, Sha512};
use std::fs;
use std::io::Write;
use tar::Builder;
use tar::Header;
use tempfile::tempdir;

#[test]
fn test_integration_install_package_end_to_end() {
    let workspace = tempdir().unwrap();
    let context = ExecutionContext::new(workspace.path().to_path_buf());

    // Build a mock package archive
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let content = b"{\"name\":\"dummy-pkg\",\"version\":\"1.2.3\"}";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", &content[..])
            .unwrap();
        builder.finish().unwrap();
    }

    let mut gz_bytes = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        encoder.finish().unwrap();
    }

    let hash = Sha512::digest(&gz_bytes);
    let b64 = STANDARD.encode(hash);
    let integrity = format!("sha512-{b64}");

    let meta = PackageMetadata {
        name: "dummy-pkg".to_string(),
        version: "1.2.3".to_string(),
        tarball_url: "https://example.com/dummy.tgz".to_string(),
        integrity,
        dependencies: std::collections::HashMap::new(),
        peer_dependencies: std::collections::HashMap::new(),
        optional_dependencies: std::collections::HashMap::new(),
    };

    let engine = InstallEngine::new(context.clone());
    let installed_path = engine.install_package(&meta, &gz_bytes).unwrap();

    assert!(installed_path.exists());
    assert!(installed_path.join("package.json").exists());

    // Verify package.json in workspace was updated
    let manifest_file = context.manifest();
    assert!(manifest_file.exists());
    let manifest_content = fs::read_to_string(manifest_file).unwrap();
    assert!(manifest_content.contains("\"dummy-pkg\": \"^1.2.3\""));
}
