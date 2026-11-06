//! Version selector: maps a parsed package spec to a concrete version using registry metadata.

use anyhow::{Context, Result, bail};
use serde_json::Value;

use super::semver_util::{is_dist_tag, select_best_version};

/// Select a concrete version string from npm registry metadata given a version requirement.
///
/// - `None` or `"latest"` → resolve the `latest` dist-tag.
/// - Any other dist-tag     → resolve that dist-tag.
/// - Semver range           → find the highest matching version.
pub fn select_version(metadata: &Value, version_req: Option<&str>) -> Result<String> {
    let req = version_req.unwrap_or("latest");

    // Resolve dist-tags first.
    if is_dist_tag(req) {
        let resolved = metadata["dist-tags"][req]
            .as_str()
            .with_context(|| format!("dist-tag '{req}' is not available for this package"))?;
        return Ok(resolved.to_string());
    }

    // Semver range: collect all available versions and pick the best.
    let versions_obj = metadata["versions"]
        .as_object()
        .context("npm metadata 'versions' field is missing or not an object")?;

    let all_versions: Vec<String> = versions_obj.keys().cloned().collect();

    if all_versions.is_empty() {
        bail!("No versions available for this package.");
    }

    let best = select_best_version(&all_versions, req)
        .with_context(|| format!("Could not evaluate version range '{req}'"))?;

    match best {
        Some(v) => Ok(v.to_string()),
        None => bail!(
            "No version satisfying '{}' found. Available: {}",
            req,
            summarize_versions(&all_versions)
        ),
    }
}

fn summarize_versions(versions: &[String]) -> String {
    if versions.len() <= 5 {
        return versions.join(", ");
    }
    let mut sorted = versions.to_vec();
    sorted.sort_by(|a, b| {
        semver::Version::parse(a)
            .ok()
            .zip(semver::Version::parse(b).ok())
            .map(|(va, vb)| va.cmp(&vb))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let last5: Vec<_> = sorted.iter().rev().take(5).collect();
    format!(
        "... ({} total, latest: {})",
        versions.len(),
        last5
            .iter()
            .rev()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_metadata(versions: &[&str], latest: &str) -> Value {
        let mut v_map = serde_json::Map::new();
        for &v in versions {
            v_map.insert(
                v.to_string(),
                json!({"dist": {"tarball": "url", "integrity": "sha512-abc"}}),
            );
        }
        json!({
            "dist-tags": {"latest": latest},
            "versions": v_map,
        })
    }

    #[test]
    fn test_select_latest_dist_tag() {
        let meta = make_metadata(&["18.0.0", "19.3.1"], "19.3.1");
        assert_eq!(select_version(&meta, None).unwrap(), "19.3.1");
        assert_eq!(select_version(&meta, Some("latest")).unwrap(), "19.3.1");
    }

    #[test]
    fn test_select_exact_version() {
        let meta = make_metadata(&["18.0.0", "18.3.1", "19.0.0"], "19.0.0");
        assert_eq!(select_version(&meta, Some("18.3.1")).unwrap(), "18.3.1");
    }

    #[test]
    fn test_select_caret_range() {
        let meta = make_metadata(&["17.0.0", "18.0.0", "18.3.1", "19.0.0"], "19.0.0");
        assert_eq!(select_version(&meta, Some("^18.0.0")).unwrap(), "18.3.1");
    }

    #[test]
    fn test_no_matching_version_error() {
        let meta = make_metadata(&["17.0.0", "16.0.0"], "17.0.0");
        assert!(select_version(&meta, Some("^18.0.0")).is_err());
    }

    #[test]
    fn test_missing_dist_tag_error() {
        let meta = make_metadata(&["18.0.0"], "18.0.0");
        assert!(select_version(&meta, Some("next")).is_err());
    }
}
