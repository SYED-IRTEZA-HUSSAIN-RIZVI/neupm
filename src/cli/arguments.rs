use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "neupm",
    version,
    about = "NEUPM - Next-Generation Package Manager",
    long_about = "A high-performance package manager built in Rust."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Install project dependencies
    Install {
        /// Prevent updating lockfile and fail if lockfile is out of sync with package.json
        #[arg(long, default_value_t = false)]
        frozen_lockfile: bool,
    },

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
