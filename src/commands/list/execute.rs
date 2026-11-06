//! Implementation of `neupm list` command.

use anyhow::Result;
use std::fs;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::lockfile::read_lockfile;
use crate::manifest::PackageJson;

/// Execute the `list` command.
///
/// Displays the project name, version, and the list of installed packages
/// with their declared and actually installed versions.
pub async fn execute_list(context: ExecutionContext) -> Result<()> {
    output::print_banner();

    let manifest_path = context.manifest();
    if !manifest_path.exists() {
        println!("No package.json found in current directory.");
        return Ok(());
    }

    let manifest = PackageJson::load_or_default(manifest_path)?;
    let project_name = manifest.name().unwrap_or("unnamed-project");
    let project_version = manifest.version().unwrap_or("0.0.0");

    println!(
        "{}@{} ({})\n",
        project_name,
        project_version,
        context.working_dir.display()
    );

    let deps = manifest.dependencies();
    if deps.is_empty() {
        println!("No dependencies declared in package.json.");
        return Ok(());
    }

    println!("Direct dependencies ({}):", deps.len());

    let mut dep_keys: Vec<_> = deps.keys().collect();
    dep_keys.sort();

    let total = dep_keys.len();
    for (i, &name) in dep_keys.iter().enumerate() {
        let is_last = i == total - 1;
        let prefix = if is_last { "└── " } else { "├── " };

        let declared_req = &deps[name];
        let installed_pkg_json = context.node_modules().join(name).join("package.json");

        let status = if installed_pkg_json.exists() {
            match fs::read_to_string(&installed_pkg_json) {
                Ok(content) => {
                    let parsed: serde_json::Value =
                        serde_json::from_str(&content).unwrap_or_default();
                    let installed_version = parsed["version"].as_str().unwrap_or("unknown");
                    format!("{name}@{installed_version} (declared: {declared_req})")
                }
                Err(_) => format!("{name} (declared: {declared_req}) [read error]"),
            }
        } else {
            format!("{name} (declared: {declared_req}) [MISSING from node_modules]")
        };

        println!("{prefix}{status}");
    }

    // Check lockfile for total recorded packages.
    if let Ok(Some(lockfile)) = read_lockfile(context.lockfile()) {
        let transitive_count = lockfile.packages.values().filter(|e| !e.is_direct).count();
        if transitive_count > 0 {
            println!("\nTransitive dependencies recorded in lockfile: {transitive_count}");
        }
    }

    Ok(())
}
