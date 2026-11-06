use clap::Parser;
use neupm::cli::{Cli, Commands};

#[test]
fn test_cli_parsing_add() {
    let args = vec!["neupm", "add", "react"];
    let parsed = Cli::try_parse_from(args).unwrap();
    assert_eq!(
        parsed.command,
        Some(Commands::Add {
            package: "react".to_string()
        })
    );
}

#[test]
fn test_cli_parsing_scoped_add() {
    let args = vec!["neupm", "add", "@types/node"];
    let parsed = Cli::try_parse_from(args).unwrap();
    assert_eq!(
        parsed.command,
        Some(Commands::Add {
            package: "@types/node".to_string()
        })
    );
}

#[test]
fn test_cli_parsing_subcommands() {
    assert_eq!(
        Cli::try_parse_from(vec!["neupm", "install"])
            .unwrap()
            .command,
        Some(Commands::Install {
            frozen_lockfile: false
        })
    );
    assert_eq!(
        Cli::try_parse_from(vec!["neupm", "install", "--frozen-lockfile"])
            .unwrap()
            .command,
        Some(Commands::Install {
            frozen_lockfile: true
        })
    );
    assert_eq!(
        Cli::try_parse_from(vec!["neupm", "doctor"])
            .unwrap()
            .command,
        Some(Commands::Doctor)
    );
    assert_eq!(
        Cli::try_parse_from(vec!["neupm", "audit"]).unwrap().command,
        Some(Commands::Audit)
    );
}
