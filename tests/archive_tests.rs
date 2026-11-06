use flate2::Compression;
use flate2::write::GzEncoder;
use neupm::archive::extract_tarball;
use std::fs;
use std::io::Write;
use tar::Builder;
use tar::Header;
use tempfile::tempdir;

#[test]
fn test_integration_extract_valid_archive() {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let content = b"{\"name\":\"my-pkg\",\"version\":\"1.0.0\"}";
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

    let staging = tempdir().unwrap();
    extract_tarball(&gz_bytes, staging.path()).unwrap();

    let extracted_json = staging.path().join("package.json");
    assert!(extracted_json.exists());
    let read_back = fs::read_to_string(extracted_json).unwrap();
    assert!(read_back.contains("\"my-pkg\""));
}
