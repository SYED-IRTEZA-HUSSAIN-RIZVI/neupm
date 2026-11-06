use anyhow::Result;
use std::env;

use crate::cli::{Cli, Commands};
use crate::commands;
use crate::core::context::ExecutionContext;

/// Run the NEUPM application from parsed CLI arguments.
pub async fn run(cli: Cli) -> Result<()> {
    let working_dir = env::current_dir().unwrap_or_else(|_| ".".into());
    let context = ExecutionContext::new(working_dir);

    match cli.command {
        Some(Commands::Add { package }) => {
            commands::add::execute_add(&package, context).await?;
        }

        Some(Commands::Install { frozen_lockfile }) => {
            commands::install::execute_install(frozen_lockfile, context).await?;
        }

        Some(Commands::Remove { package }) => {
            commands::remove::execute_remove(&package, context).await?;
        }

        Some(Commands::Update) => {
            commands::update::execute_update(context).await?;
        }

        Some(Commands::Init) => {
            commands::init::execute_init(context).await?;
        }

        Some(Commands::Doctor) => {
            commands::doctor::execute_doctor(context).await?;
        }

        Some(Commands::Audit) => {
            commands::audit::execute_audit(context).await?;
        }

        Some(Commands::List) => {
            commands::list::execute_list(context).await?;
        }

        None => {
            println!("NEUPM - Next-Generation Package Manager");
            println!("Run 'neupm --help' to see available commands.");
        }
    }

    Ok(())
}
