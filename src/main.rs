mod extract;
mod install;
mod registry;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "neupm",
    version,
    about = "NEUPM - Next-Generation Package Manager",
    long_about = "A high-performance package manager built in Rust."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install project dependencies
    Install,

    /// Add and install a package
    Add { package: String },

    /// Remove a package
    Remove { package: String },

    /// Update dependencies
    Update,

    /// Initialize a project
    Init,

    /// Check development environment
    Doctor,

    /// Audit dependencies
    Audit,

    /// List installed packages
    List,
}

/// Validate package names before using them in URLs or filesystem paths.
fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains('\\') || name.starts_with('/') || name.ends_with('/') {
        bail!("Invalid npm package name.");
    }

    let parts: Vec<&str> = name.split('/').collect();

    if parts.iter().any(|part| {
        part.is_empty()
            || *part == "."
            || *part == ".."
            || !part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '@'))
    }) {
        bail!("Invalid npm package name: {name}");
    }

    if name.starts_with('@') && parts.len() != 2 {
        bail!("Invalid scoped package name: {name}");
    }

    if !name.starts_with('@') && parts.len() != 1 {
        bail!("Invalid package name: {name}");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { package }) => {
            install::add_package(&package).await?;
        }

        Some(Commands::Install) => {
            println!("NEUPM: Install engine is not implemented yet.");
        }

        Some(Commands::Remove { package }) => {
            println!("NEUPM: Remove engine is not implemented yet: {package}");
        }

        Some(Commands::Update) => {
            println!("NEUPM: Update engine is not implemented yet.");
        }

        Some(Commands::Init) => {
            println!("NEUPM: Project initialization is not implemented yet.");
        }

        Some(Commands::Doctor) => {
            println!("NEUPM: Environment diagnostics are not implemented yet.");
        }

        Some(Commands::Audit) => {
            println!("NEUPM: Security auditing is not implemented yet.");
        }

        Some(Commands::List) => {
            println!("NEUPM: Package listing is not implemented yet.");
        }

        None => {
            println!("NEUPM - Next-Generation Package Manager");
            println!("Run 'neupm --help' to see available commands.");
        }
    }

    Ok(())
}
