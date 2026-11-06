use anyhow::{Result, bail};
use std::path::{Component, Path};

/// Validate a path component to reject path traversal, absolute paths,
/// Windows drive-letter paths, UNC paths, and null bytes.
pub fn validate_entry_path(path: &Path) -> Result<()> {
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
                if s_str.contains('\0') {
                    bail!("Malicious archive entry: null byte in path: {path_str}");
                }
            }
            Component::CurDir => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_entry_paths() {
        assert!(validate_entry_path(Path::new("index.js")).is_ok());
        assert!(validate_entry_path(Path::new("lib/sub/deep.d.ts")).is_ok());
    }

    #[test]
    fn test_rejects_path_traversal() {
        assert!(validate_entry_path(Path::new("../evil.js")).is_err());
        assert!(validate_entry_path(Path::new("lib/../../evil.js")).is_err());
    }

    #[test]
    fn test_rejects_absolute_and_drive_paths() {
        assert!(validate_entry_path(Path::new("/etc/passwd")).is_err());
        assert!(validate_entry_path(Path::new("C:\\Windows\\system.ini")).is_err());
        assert!(validate_entry_path(Path::new("D:test.js")).is_err());
    }

    #[test]
    fn test_rejects_unc_paths() {
        assert!(validate_entry_path(Path::new("\\\\server\\share\\pkg")).is_err());
        assert!(validate_entry_path(Path::new("//server/share/pkg")).is_err());
    }
}
