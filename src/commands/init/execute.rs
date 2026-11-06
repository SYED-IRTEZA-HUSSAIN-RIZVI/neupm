//! Implementation of `neupm init` command.

use anyhow::Result;
use serde_json::json;

use crate::cli::output;
use crate::core::context::ExecutionContext;
use crate::utils::atomic_write::atomic_write;

/// Execute the `init` command.
///
/// Creates a standard `package.json` file in the current working directory
/// if one does not already exist.
pub async fn execute_init(context: ExecutionContext) -> Result<()> {
    output::print_banner();

    let manifest_path = context.manifest();
    if manifest_path.exists() {
        println!("package.json already exists at {}", manifest_path.display());
        return Ok(());
    }

    let default_name = context
        .working_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_lowercase()
        .replace(' ', "-");

    let initial_json = json!({
        "name": default_name,
        "version": "1.0.0",
        "description": "",
        "main": "index.js",
        "scripts": {
            "test": "echo \"Error: no test specified\" && exit 1"
        },
        "keywords": [],
        "author": "",
        "license": "ISC",
        "dependencies": {}
    });

    let content = serde_json::to_string_pretty(&initial_json)?;
    let content_with_nl = format!("{content}\n");

    atomic_write(manifest_path, content_with_nl.as_bytes())?;

    println!("Initialized package.json at {}", manifest_path.display());
    println!("{content_with_nl}");

    Ok(())
}
