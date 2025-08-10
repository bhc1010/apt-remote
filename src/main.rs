//! # apt-remote
//!
//! `apt-remote` is a CLI tool for managing offline Debian package installation
//! via SSH. It supports installing packages and updating source lists on a device 
//! without direct internet access.
//!
//! ## Features
//! - Generate a `uri.toml` configuration file for package sources
//! - Download packages and source list metadata
//! - Install packages on a remote system over SSH
//! - Update package lists on the remote system

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod ssh;
mod uri;

use commands::{set, get, install, update, clear};

/// Command-line interface for the `apt-remote` application.
///
/// This struct is parsed from the command line using `clap`.
#[derive(Parser)]
#[command(name = "apt-remote")]
#[command(about = "Manage offline APT package installation over SSH", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for `apt-remote`.
#[derive(Subcommand)]
enum Commands {
    /// Generate uri.toml file
    Set(set::SetArgs),

    /// Download package files and metadata according to uri.toml file
    Get(get::GetArgs),

    /// Upload packages and install on remote system
    Install(install::InstallArgs),

    /// Upload apt package lists onto remote system
    Update(update::UpdateArgs),

    /// Clear all local cache (uri and deb files stored at $HOME/.cache/apt-remote)
    Clear,
}

/// Entry point for the `apt-remote` CLI application.
///
/// Parses command-line arguments, executes the appropriate subcommand

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set(args) => set::run(args)?,
        Commands::Get(args) => get::run(args)?,
        Commands::Install(args) => install::run(args)?,
        Commands::Update(args) => update::run(args)?,
        Commands::Clear => clear::run()?,
    }

    Ok(())
}
