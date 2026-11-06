use neupm::core::package::PackageMetadata;
use neupm::resolver::dependency_graph::DependencyGraph;
use neupm::resolver::package_spec::ParsedSpec;
use neupm::resolver::semver_util::version_satisfies;
use neupm::resolver::version_selector::select_version;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_package_spec_parsing_comprehensive() {
    let s1 = ParsedSpec::parse("react").unwrap();
    assert_eq!(s1.name, "react");
    assert_eq!(s1.version_req, None);

    let s2 = ParsedSpec::parse("react@18.3.1").unwrap();
    assert_eq!(s2.name, "react");
    assert_eq!(s2.version_req, Some("18.3.1".to_string()));

    let s3 = ParsedSpec::parse("react@^18.0.0").unwrap();
    assert_eq!(s3.name, "react");
    assert_eq!(s3.version_req, Some("^18.0.0".to_string()));

    let s4 = ParsedSpec::parse("react@~18.2.0").unwrap();
    assert_eq!(s4.name, "react");
    assert_eq!(s4.version_req, Some("~18.2.0".to_string()));

    let s5 = ParsedSpec::parse("@types/node").unwrap();
    assert_eq!(s5.name, "@types/node");
    assert_eq!(s5.version_req, None);

    let s6 = ParsedSpec::parse("@types/node@22").unwrap();
    assert_eq!(s6.name, "@types/node");
    assert_eq!(s6.version_req, Some("22".to_string()));

    let s7 = ParsedSpec::parse("@types/node@^22.0.0").unwrap();
    assert_eq!(s7.name, "@types/node");
    assert_eq!(s7.version_req, Some("^22.0.0".to_string()));
}

#[test]
fn test_version_satisfaction_rules() {
    // Caret ranges
    assert!(version_satisfies("18.3.1", "^18.0.0").unwrap());
    assert!(!version_satisfies("19.0.0", "^18.0.0").unwrap());

    // Tilde ranges
    assert!(version_satisfies("18.2.4", "~18.2.0").unwrap());
    assert!(!version_satisfies("18.3.0", "~18.2.0").unwrap());

    // Bare major numbers
    assert!(version_satisfies("22.6.0", "22").unwrap());
    assert!(!version_satisfies("23.0.0", "22").unwrap());

    // Wildcards
    assert!(version_satisfies("1.2.3", "*").unwrap());
    assert!(version_satisfies("1.8.0", "1.x").unwrap());

    // Disjunctions (OR)
    assert!(version_satisfies("3.1.0", "^3.0.0 || ^4.0.0").unwrap());
    assert!(version_satisfies("4.2.0", "^3.0.0 || ^4.0.0").unwrap());
    assert!(!version_satisfies("5.0.0", "^3.0.0 || ^4.0.0").unwrap());
}

#[test]
fn test_version_selector_from_registry_metadata() {
    let raw_meta = json!({
        "dist-tags": {
            "latest": "19.3.1",
            "next": "19.4.0-canary"
        },
        "versions": {
            "17.0.2": {"dist": {"tarball": "url", "integrity": "sha512-1"}},
            "18.2.0": {"dist": {"tarball": "url", "integrity": "sha512-2"}},
            "18.3.1": {"dist": {"tarball": "url", "integrity": "sha512-3"}},
            "19.3.1": {"dist": {"tarball": "url", "integrity": "sha512-4"}}
        }
    });

    // Default latest
    assert_eq!(select_version(&raw_meta, None).unwrap(), "19.3.1");
    assert_eq!(select_version(&raw_meta, Some("latest")).unwrap(), "19.3.1");

    // Exact version
    assert_eq!(select_version(&raw_meta, Some("18.3.1")).unwrap(), "18.3.1");

    // Caret range
    assert_eq!(
        select_version(&raw_meta, Some("^18.0.0")).unwrap(),
        "18.3.1"
    );

    // Tilde range
    assert_eq!(
        select_version(&raw_meta, Some("~18.2.0")).unwrap(),
        "18.2.0"
    );

    // Unmatched range fails
    assert!(select_version(&raw_meta, Some("^20.0.0")).is_err());
}

#[test]
fn test_dependency_graph_cycle_detection() {
    let mut graph = DependencyGraph::new();

    // Create a cycle: A -> B -> C -> A
    let make_node = |name: &str, dep: &str| PackageMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        tarball_url: "url".to_string(),
        integrity: "sha512-x".to_string(),
        dependencies: {
            let mut m = HashMap::new();
            m.insert(dep.to_string(), "^1.0.0".to_string());
            m
        },
        peer_dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
    };

    graph.insert(&make_node("pkg-a", "pkg-b"));
    graph.insert(&make_node("pkg-b", "pkg-c"));
    graph.insert(&make_node("pkg-c", "pkg-a"));

    let order_result = graph.installation_order();
    assert!(order_result.is_err());
    let err_msg = order_result.unwrap_err().to_string();
    assert!(err_msg.contains("Dependency cycle detected"));
}
