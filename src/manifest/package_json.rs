use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// In-memory representation and parser for package.json.
#[derive(Debug, Clone)]
pub struct PackageJson {
    raw: Value,
}

impl Default for PackageJson {
    fn default() -> Self {
        Self {
            raw: json!({
                "name": "neupm-project",
                "version": "1.0.0"
            }),
        }
    }
}

impl PackageJson {
    /// Load an existing package.json or create a default if not found.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path).context("Could not read package.json")?;
            let raw: Value =
                serde_json::from_str(&content).context("Invalid package.json format")?;
            if !raw.is_object() {
                bail!("Invalid package.json: expected a JSON object.");
            }
            Ok(Self { raw })
        } else {
            Ok(Self::default())
        }
    }

    /// Add or update a dependency version constraint.
    pub fn set_dependency(&mut self, package: &str, version_constraint: &str) {
        if !self.raw["dependencies"].is_object() {
            self.raw["dependencies"] = json!({});
        }
        self.raw["dependencies"][package] = json!(version_constraint);
    }

    /// Render formatted JSON with a trailing newline.
    pub fn to_formatted_string(&self) -> Result<String> {
        let formatted = serde_json::to_string_pretty(&self.raw)?;
        Ok(format!("{formatted}\n"))
    }
}
