//! Parsing of user-supplied package specifications such as `react@^18.0.0` or `@types/node@22`.

use anyhow::{Result, bail};

/// A parsed package specification separating the package name from the version constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSpec {
    /// Full npm package name, including scope (e.g., `@types/node`).
    pub name: String,
    /// Raw version constraint as supplied by the user (e.g., `^18.0.0`, `latest`, `22`).
    /// `None` means "resolve the dist-tag `latest`".
    pub version_req: Option<String>,
}

impl ParsedSpec {
    /// Parse a user-supplied package specification string.
    ///
    /// Supported formats:
    /// - `react`                 → name=react, req=None
    /// - `react@latest`          → name=react, req=Some("latest")
    /// - `react@18.3.1`          → name=react, req=Some("18.3.1")
    /// - `react@^18.0.0`         → name=react, req=Some("^18.0.0")
    /// - `@types/node`           → name=@types/node, req=None
    /// - `@types/node@22`        → name=@types/node, req=Some("22")
    /// - `@types/node@^22.0.0`   → name=@types/node, req=Some("^22.0.0")
    pub fn parse(spec: &str) -> Result<Self> {
        if spec.is_empty() {
            bail!("Package specification must not be empty.");
        }

        if let Some(rest) = spec.strip_prefix('@') {
            // Scoped package: find second '@'
            if let Some(at_idx) = rest.find('@') {
                let name = &spec[..at_idx + 1]; // includes leading '@'
                let version_req = &spec[at_idx + 2..];
                if version_req.is_empty() {
                    bail!("Version constraint in '{spec}' must not be empty after '@'.");
                }
                return Ok(Self {
                    name: name.to_string(),
                    version_req: Some(version_req.to_string()),
                });
            }
            // No version — just the scoped name
            return Ok(Self {
                name: spec.to_string(),
                version_req: None,
            });
        }

        // Unscoped package
        if let Some((name, version_req)) = spec.split_once('@') {
            if version_req.is_empty() {
                bail!("Version constraint in '{spec}' must not be empty after '@'.");
            }
            return Ok(Self {
                name: name.to_string(),
                version_req: Some(version_req.to_string()),
            });
        }

        Ok(Self {
            name: spec.to_string(),
            version_req: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unscoped_no_version() {
        let s = ParsedSpec::parse("react").unwrap();
        assert_eq!(s.name, "react");
        assert_eq!(s.version_req, None);
    }

    #[test]
    fn test_unscoped_latest() {
        let s = ParsedSpec::parse("react@latest").unwrap();
        assert_eq!(s.name, "react");
        assert_eq!(s.version_req, Some("latest".to_string()));
    }

    #[test]
    fn test_unscoped_exact() {
        let s = ParsedSpec::parse("react@18.3.1").unwrap();
        assert_eq!(s.name, "react");
        assert_eq!(s.version_req, Some("18.3.1".to_string()));
    }

    #[test]
    fn test_unscoped_caret() {
        let s = ParsedSpec::parse("react@^18.0.0").unwrap();
        assert_eq!(s.name, "react");
        assert_eq!(s.version_req, Some("^18.0.0".to_string()));
    }

    #[test]
    fn test_unscoped_tilde() {
        let s = ParsedSpec::parse("react@~18.2.0").unwrap();
        assert_eq!(s.name, "react");
        assert_eq!(s.version_req, Some("~18.2.0".to_string()));
    }

    #[test]
    fn test_scoped_no_version() {
        let s = ParsedSpec::parse("@types/node").unwrap();
        assert_eq!(s.name, "@types/node");
        assert_eq!(s.version_req, None);
    }

    #[test]
    fn test_scoped_with_version() {
        let s = ParsedSpec::parse("@types/node@22").unwrap();
        assert_eq!(s.name, "@types/node");
        assert_eq!(s.version_req, Some("22".to_string()));
    }

    #[test]
    fn test_scoped_with_caret() {
        let s = ParsedSpec::parse("@types/node@^22.0.0").unwrap();
        assert_eq!(s.name, "@types/node");
        assert_eq!(s.version_req, Some("^22.0.0".to_string()));
    }

    #[test]
    fn test_empty_spec_is_error() {
        assert!(ParsedSpec::parse("").is_err());
    }

    #[test]
    fn test_trailing_at_is_error() {
        assert!(ParsedSpec::parse("react@").is_err());
        assert!(ParsedSpec::parse("@types/node@").is_err());
    }
}
