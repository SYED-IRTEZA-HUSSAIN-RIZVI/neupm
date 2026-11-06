//! Implementation of `neupm doctor` command.

use anyhow::Result;
use std::process::Command;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::lockfile::{
    LockfileValidationResult, read_lockfile, validate_lockfile_against_manifest,
};
use crate::manifest::PackageJson;
use crate::registry::build_http_client;

/// Execute the `doctor` command.
///
/// Diagnoses the project environment, configuration, and dependencies.
pub async fn execute_doctor(context: ExecutionContext) -> Result<()> {
    output::print_banner();
    println!("NEUPM Doctor - System & Project Diagnostic");
    println!("==========================================\n");

    let mut warnings = 0;
    let mut errors = 0;

    // 1. NEUPM Version
    println!("[OK] NEUPM version: {}", env!("CARGO_PKG_VERSION"));

    // 2. Node.js environment
    match Command::new("node").arg("--version").output() {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("[OK] Node.js runtime detected: {ver}");
        }
        _ => {
            println!("[WARN] Node.js not detected in system PATH.");
            warnings += 1;
        }
    }

    // 3. Manifest (package.json)
    let manifest_path = context.manifest();
    if manifest_path.exists() {
        match PackageJson::load_or_default(manifest_path) {
            Ok(manifest) => {
                let deps = manifest.dependencies();
                println!(
                    "[OK] package.json valid (name: '{}', deps: {})",
                    manifest.name().unwrap_or("unnamed"),
                    deps.len()
                );

                // 4. Lockfile check
                let lockfile_path = context.lockfile();
                match read_lockfile(lockfile_path) {
                    Ok(Some(lockfile)) => {
                        match validate_lockfile_against_manifest(&lockfile, &deps) {
                            LockfileValidationResult::Valid => {
                                println!(
                                    "[OK] neupm-lock.json valid and in sync ({} packages recorded)",
                                    lockfile.packages.len()
                                );
                            }
                            LockfileValidationResult::MissingPackages(missing) => {
                                println!(
                                    "[WARN] neupm-lock.json is missing {} package(s): {}",
                                    missing.len(),
                                    missing.join(", ")
                                );
                                warnings += 1;
                            }
                            LockfileValidationResult::UnknownVersion(v) => {
                                println!("[ERROR] neupm-lock.json has unsupported version {v}");
                                errors += 1;
                            }
                        }
                    }
                    Ok(None) => {
                        if !deps.is_empty() {
                            println!(
                                "[WARN] No neupm-lock.json found. Run 'neupm install' to create one."
                            );
                            warnings += 1;
                        } else {
                            println!("[INFO] No neupm-lock.json (no dependencies declared).");
                        }
                    }
                    Err(e) => {
                        println!("[ERROR] neupm-lock.json is corrupt: {e}");
                        errors += 1;
                    }
                }

                // 5. node_modules check
                let nm = context.node_modules();
                if nm.exists() {
                    let mut missing_dirs = Vec::new();
                    for dep_name in deps.keys() {
                        let pkg_dir = nm.join(dep_name);
                        if !pkg_dir.exists() {
                            missing_dirs.push(dep_name.clone());
                        }
                    }
                    if missing_dirs.is_empty() {
                        println!(
                            "[OK] node_modules exists and contains all declared dependencies."
                        );
                    } else {
                        println!(
                            "[WARN] node_modules is missing {} package(s): {}",
                            missing_dirs.len(),
                            missing_dirs.join(", ")
                        );
                        warnings += 1;
                    }
                } else if !deps.is_empty() {
                    println!("[WARN] node_modules does not exist. Run 'neupm install'.");
                    warnings += 1;
                }
            }
            Err(e) => {
                println!("[ERROR] package.json exists but is invalid: {e}");
                errors += 1;
            }
        }
    } else {
        println!("[WARN] No package.json found in current directory. Run 'neupm init'.");
        warnings += 1;
    }

    // 6. Network connectivity to npm registry
    print!("Checking npm registry connectivity... ");
    match build_http_client() {
        Ok(client) => match client
            .get("https://registry.npmjs.org")
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!("[OK] connected to https://registry.npmjs.org");
            }
            Ok(resp) => {
                println!("[WARN] registry returned HTTP {}", resp.status());
                warnings += 1;
            }
            Err(e) => {
                println!("[WARN] registry connection failed: {e}");
                warnings += 1;
            }
        },
        Err(e) => {
            println!("[ERROR] HTTP client build failed: {e}");
            errors += 1;
        }
    }

    println!(
        "\nDiagnostic complete: {} error(s), {} warning(s).",
        errors, warnings
    );

    Ok(())
}
