use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod ssh;
mod uri;

use commands::{set, get, install, update, clear};

#[derive(Parser)]
#[command(name = "apt-remote")]
#[command(about = "Manage offline APT package installation over SSH", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
