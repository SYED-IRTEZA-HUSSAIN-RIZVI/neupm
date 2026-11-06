use anyhow::{Result, bail};

/// Validate package names before using them in URLs or filesystem paths.
pub fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains('\\') || name.starts_with('/') || name.ends_with('/') {
        bail!("Invalid npm package name.");
    }

    let parts: Vec<&str> = name.split('/').collect();

    if parts.iter().any(|part| {
        part.is_empty()
            || *part == "."
            || *part == ".."
            || !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '@'))
    }) {
        bail!("Invalid npm package name: {name}");
    }

    if name.starts_with('@') && parts.len() != 2 {
        bail!("Invalid scoped package name: {name}");
    }

    if !name.starts_with('@') && parts.len() != 1 {
        bail!("Invalid package name: {name}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(validate_package_name("react").is_ok());
        assert!(validate_package_name("lodash.debounce").is_ok());
        assert!(validate_package_name("@types/node").is_ok());
        assert!(validate_package_name("@scope/my-pkg_1.0").is_ok());
    }

    #[test]
    fn test_invalid_package_names() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("react/extra/parts").is_err());
        assert!(validate_package_name("@types").is_err());
        assert!(validate_package_name("..").is_err());
        assert!(validate_package_name("react\\native").is_err());
    }
}
