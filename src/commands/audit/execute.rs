//! Implementation of `neupm audit` command.

use anyhow::Result;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::lockfile::read_lockfile;
use crate::manifest::PackageJson;

/// Execute the `audit` command.
///
/// Inspects dependencies recorded in `package.json` and `neupm-lock.json`
/// for security anomalies:
/// - Unencrypted (HTTP) tarball URLs.
/// - Missing or malformed integrity hashes.
/// - Unpinned or wildcard version ranges.
///
/// Reports real, verifiable findings without fabricating CVE data.
pub async fn execute_audit(context: ExecutionContext) -> Result<()> {
    output::print_banner();
    println!("NEUPM Security Audit");
    println!("====================\n");

    let mut issues_found = 0;

    // 1. Audit manifest
    let manifest_path = context.manifest();
    if manifest_path.exists()
        && let Ok(manifest) = PackageJson::load_or_default(manifest_path)
    {
        let deps = manifest.dependencies();
        for (name, range) in &deps {
            if range == "*" || range == "latest" || range.starts_with(">=") {
                println!(
                    "[WARN] Package '{name}' uses loose version constraint '{range}'. \
                     Consider pinning to a caret or exact version to prevent unintended updates."
                );
                issues_found += 1;
            }
        }
    }

    // 2. Audit lockfile
    let lockfile_path = context.lockfile();
    match read_lockfile(lockfile_path)? {
        Some(lockfile) => {
            println!(
                "Auditing {} locked package(s) for integrity and transport security...",
                lockfile.packages.len()
            );

            for (name, entry) in &lockfile.packages {
                // Check HTTPS
                if !entry.resolved.starts_with("https://") {
                    println!(
                        "[ALERT] Insecure transport: '{name}' downloads over unencrypted URL: {}",
                        entry.resolved
                    );
                    issues_found += 1;
                }

                // Check integrity format
                if !entry.integrity.starts_with("sha512-") {
                    println!(
                        "[WARN] Package '{name}' uses non-SHA512 integrity: {}",
                        entry.integrity
                    );
                    issues_found += 1;
                } else {
                    let hash_part = &entry.integrity["sha512-".len()..];
                    if base64::Engine::decode(&base64::engine::general_purpose::STANDARD, hash_part)
                        .is_err()
                    {
                        println!(
                            "[ALERT] Package '{name}' has invalid Base64 in integrity string: {}",
                            entry.integrity
                        );
                        issues_found += 1;
                    }
                }
            }
        }
        None => {
            println!("[INFO] No neupm-lock.json found to audit. Run 'neupm install' first.");
        }
    }

    println!("\n----------------------------------------");
    if issues_found == 0 {
        println!(
            "Security Audit Passed: No known security risks or transport vulnerabilities found."
        );
    } else {
        println!("Security Audit Finished: {issues_found} potential security issue(s) identified.");
    }
    println!("----------------------------------------");

    Ok(())
}
