//! Semantic version utilities for npm-compatible version range matching.
//!
//! This module wraps the `semver` crate with npm-specific semantics:
//! - Bare major integers (`22`) are treated as `>=22.0.0, <23.0.0` (caret range).
//! - Dist-tags (`latest`, `next`) are handled before semver parsing.

use anyhow::{Context, Result};
use semver::{Version, VersionReq};

/// Determine whether a given version string satisfies an npm version range string.
///
/// Returns `Ok(true)` if the version matches, `Ok(false)` if it does not.
/// Returns an error only if the range itself cannot be parsed.
pub fn version_satisfies(version_str: &str, range_str: &str) -> Result<bool> {
    let reqs = parse_npm_range_disjunction(range_str)
        .with_context(|| format!("Invalid version range: '{range_str}'"))?;
    let version =
        Version::parse(version_str).with_context(|| format!("Invalid version: '{version_str}'"))?;
    Ok(reqs.iter().any(|req| req.matches(&version)))
}

/// Select the best matching version from a list of candidates.
///
/// Returns the highest version that satisfies the range (including `||` disjunctions).
/// Returns `None` if no candidate matches.
pub fn select_best_version<'a>(
    candidates: &'a [String],
    range_str: &str,
) -> Result<Option<&'a str>> {
    let reqs = parse_npm_range_disjunction(range_str)
        .with_context(|| format!("Invalid version range: '{range_str}'"))?;

    let mut best: Option<(&str, Version)> = None;

    for candidate in candidates {
        // Skip prerelease versions unless range explicitly includes them.
        let version = match Version::parse(candidate) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if !reqs.iter().any(|req| req.matches(&version)) {
            continue;
        }

        match &best {
            None => best = Some((candidate.as_str(), version)),
            Some((_, best_v)) if version > *best_v => best = Some((candidate.as_str(), version)),
            _ => {}
        }
    }

    Ok(best.map(|(s, _)| s))
}

/// Parse an npm range that may contain `||` (OR) disjunctions into a list of `VersionReq`.
pub fn parse_npm_range_disjunction(range: &str) -> Result<Vec<VersionReq>> {
    let mut reqs = Vec::new();
    for part in range.split("||") {
        let part = part.trim();
        if !part.is_empty() {
            reqs.push(parse_npm_range(part)?);
        }
    }
    if reqs.is_empty() {
        reqs.push(parse_npm_range("*")?);
    }
    Ok(reqs)
}

/// Parse an npm-style version range string into a `VersionReq`.
///
/// Handles npm-specific extensions:
/// - Bare major (`22`) → `^22`
/// - Bare major.minor (`22.3`) → `^22.3`
/// - Wildcards (`*`, `x`, `1.x`, `1.2.x`)
/// - Space-separated comparators (`>=1.0.0 <2.0.0` → `>=1.0.0, <2.0.0`)
pub fn parse_npm_range(range: &str) -> Result<VersionReq> {
    let normalized = normalize_npm_range(range);
    VersionReq::parse(&normalized).with_context(|| {
        format!("Could not parse version range '{range}' (normalized: '{normalized}')")
    })
}

/// Normalize an npm range string to semver-crate syntax.
fn normalize_npm_range(range: &str) -> String {
    let trimmed = range.trim();

    if trimmed.is_empty() || trimmed == "*" || trimmed == "x" || trimmed == "X" {
        return ">=0.0.0".to_string();
    }

    // Space-separated comparators like ">=1.0.0 <2.0.0" -> ">=1.0.0, <2.0.0"
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() > 1 {
        let mut parts = Vec::new();
        for token in tokens {
            parts.push(normalize_single_comparator(token));
        }
        return parts.join(", ");
    }

    normalize_single_comparator(trimmed)
}

fn normalize_single_comparator(trimmed: &str) -> String {
    // Wildcards: 1.x -> ^1, 1.2.x -> ~1.2
    if let Some(prefix) = trimmed
        .strip_suffix(".x")
        .or_else(|| trimmed.strip_suffix(".X"))
        .or_else(|| trimmed.strip_suffix(".*"))
    {
        if prefix.contains('.') {
            return format!("~{prefix}");
        } else {
            return format!("^{prefix}");
        }
    }

    // Already has a range operator — pass through.
    if trimmed.starts_with('^')
        || trimmed.starts_with('~')
        || trimmed.starts_with('>')
        || trimmed.starts_with('<')
        || trimmed.starts_with('=')
    {
        return trimmed.to_string();
    }

    // Bare version or partial version — treat as caret range.
    // e.g., "22" → "^22", "22.3" → "^22.3", "22.3.1" → "22.3.1" (exact)
    let dots = trimmed.chars().filter(|&c| c == '.').count();
    if dots < 2 && trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return format!("^{trimmed}");
    }

    trimmed.to_string()
}

/// Parse and validate a semver version string.
pub fn parse_version(v: &str) -> Result<Version> {
    Version::parse(v).with_context(|| format!("Invalid semver version: '{v}'"))
}

/// Compare two version strings. Returns `Ok(Ordering)`.
pub fn compare_versions(a: &str, b: &str) -> Result<std::cmp::Ordering> {
    let va = parse_version(a)?;
    let vb = parse_version(b)?;
    Ok(va.cmp(&vb))
}

/// Check whether a version string represents a pre-release version.
pub fn is_prerelease(version_str: &str) -> bool {
    Version::parse(version_str)
        .map(|v| !v.pre.is_empty())
        .unwrap_or(false)
}

/// Return `true` if the range_str is a dist-tag (not a semver expression).
pub fn is_dist_tag(range_str: &str) -> bool {
    // Dist-tags are alphabetical identifiers like "latest", "next", "beta".
    // They do not start with digits or semver operators.
    let t = range_str.trim();
    !t.is_empty()
        && !t.starts_with('^')
        && !t.starts_with('~')
        && !t.starts_with('>')
        && !t.starts_with('<')
        && !t.starts_with('=')
        && !t.starts_with('*')
        && !t
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_version_match() {
        assert!(version_satisfies("18.3.1", "18.3.1").unwrap());
        assert!(!version_satisfies("18.3.0", "18.3.1").unwrap());
    }

    #[test]
    fn test_caret_range() {
        assert!(version_satisfies("18.3.1", "^18.0.0").unwrap());
        assert!(version_satisfies("18.9.9", "^18.0.0").unwrap());
        assert!(!version_satisfies("19.0.0", "^18.0.0").unwrap());
        assert!(!version_satisfies("17.9.9", "^18.0.0").unwrap());
    }

    #[test]
    fn test_tilde_range() {
        assert!(version_satisfies("18.2.5", "~18.2.0").unwrap());
        assert!(!version_satisfies("18.3.0", "~18.2.0").unwrap());
    }

    #[test]
    fn test_or_disjunction_range() {
        assert!(version_satisfies("3.2.0", "^3.0.0 || ^4.0.0").unwrap());
        assert!(version_satisfies("4.1.0", "^3.0.0 || ^4.0.0").unwrap());
        assert!(!version_satisfies("5.0.0", "^3.0.0 || ^4.0.0").unwrap());

        let candidates = vec![
            "3.2.0".to_string(),
            "4.1.0".to_string(),
            "5.0.0".to_string(),
        ];
        let best = select_best_version(&candidates, "^3.0.0 || ^4.0.0").unwrap();
        assert_eq!(best, Some("4.1.0"));
    }

    #[test]
    fn test_space_separated_comparators() {
        assert!(version_satisfies("1.5.0", ">=1.0.0 <2.0.0").unwrap());
        assert!(!version_satisfies("2.0.0", ">=1.0.0 <2.0.0").unwrap());
    }

    #[test]
    fn test_wildcard_x() {
        assert!(version_satisfies("1.9.0", "1.x").unwrap());
        assert!(!version_satisfies("2.0.0", "1.x").unwrap());
        assert!(version_satisfies("1.2.9", "1.2.x").unwrap());
        assert!(!version_satisfies("1.3.0", "1.2.x").unwrap());
    }

    #[test]
    fn test_bare_major() {
        assert!(version_satisfies("22.3.1", "22").unwrap());
        assert!(!version_satisfies("23.0.0", "22").unwrap());
    }

    #[test]
    fn test_wildcard() {
        assert!(version_satisfies("1.2.3", "*").unwrap());
        assert!(version_satisfies("99.0.0", "*").unwrap());
    }

    #[test]
    fn test_select_best_version() {
        let candidates = vec![
            "17.0.0".to_string(),
            "18.0.0".to_string(),
            "18.2.0".to_string(),
            "18.3.1".to_string(),
            "19.0.0".to_string(),
        ];
        let best = select_best_version(&candidates, "^18.0.0").unwrap();
        assert_eq!(best, Some("18.3.1"));
    }

    #[test]
    fn test_select_best_no_match() {
        let candidates = vec!["17.0.0".to_string(), "16.0.0".to_string()];
        let best = select_best_version(&candidates, "^18.0.0").unwrap();
        assert_eq!(best, None);
    }

    #[test]
    fn test_is_dist_tag() {
        assert!(is_dist_tag("latest"));
        assert!(is_dist_tag("next"));
        assert!(is_dist_tag("beta"));
        assert!(!is_dist_tag("^18.0.0"));
        assert!(!is_dist_tag("18.3.1"));
        assert!(!is_dist_tag("*"));
    }

    #[test]
    fn test_is_prerelease() {
        assert!(is_prerelease("1.0.0-alpha.1"));
        assert!(!is_prerelease("1.0.0"));
    }

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;
        assert_eq!(compare_versions("1.0.0", "2.0.0").unwrap(), Ordering::Less);
        assert_eq!(
            compare_versions("2.0.0", "1.0.0").unwrap(),
            Ordering::Greater
        );
        assert_eq!(compare_versions("1.0.0", "1.0.0").unwrap(), Ordering::Equal);
    }
}
