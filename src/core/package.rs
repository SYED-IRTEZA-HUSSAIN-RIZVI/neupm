use std::collections::HashMap;

/// Metadata resolved from a package registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub integrity: String,
    /// Runtime dependencies declared in the resolved version's package.json.
    pub dependencies: HashMap<String, String>,
    /// Peer dependencies (informational; handled separately).
    pub peer_dependencies: HashMap<String, String>,
    /// Optional dependencies (failure tolerated).
    pub optional_dependencies: HashMap<String, String>,
}

/// Package specification for installation requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpec {
    pub name: String,
    pub requested_version: Option<String>,
}

impl PackageSpec {
    pub fn parse(spec: &str) -> Self {
        // Handle scoped packages like @types/node@^26.0.0
        if let Some(rest) = spec.strip_prefix('@') {
            if let Some(idx) = rest.find('@') {
                let name = &spec[..idx + 1];
                let version = &spec[idx + 2..];
                return Self {
                    name: name.to_string(),
                    requested_version: Some(version.to_string()),
                };
            }
        } else if let Some((name, version)) = spec.split_once('@') {
            return Self {
                name: name.to_string(),
                requested_version: Some(version.to_string()),
            };
        }

        Self {
            name: spec.to_string(),
            requested_version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        let p1 = PackageSpec::parse("react");
        assert_eq!(p1.name, "react");
        assert_eq!(p1.requested_version, None);

        let p2 = PackageSpec::parse("react@19.0.0");
        assert_eq!(p2.name, "react");
        assert_eq!(p2.requested_version, Some("19.0.0".to_string()));

        let p3 = PackageSpec::parse("@types/node");
        assert_eq!(p3.name, "@types/node");
        assert_eq!(p3.requested_version, None);

        let p4 = PackageSpec::parse("@types/node@26.6.6");
        assert_eq!(p4.name, "@types/node");
        assert_eq!(p4.requested_version, Some("26.6.6".to_string()));
    }
}
