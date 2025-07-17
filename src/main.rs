use anyhow::Result;
use clap::{Parser, Subcommand};

mod model;
mod set;
mod get;
mod install;
mod update;
mod upgrade; // new module for upgrade
mod ssh;

#[derive(Parser)]
#[command(name = "apt-remote")]
#[command(about = "Manage offline APT package installation over SSH", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture remote apt sources and optionally print package URIs
    Set {
        /// Cache image name (required)
        name: String,

        /// Remote target SSH (user@host)
        #[arg(long)]
        target: String,

        /// Packages to install
        #[arg(long, value_delimiter = ',')]
        install: Vec<String>,

        /// Capture sources and architecture for update
        #[arg(long)]
        update: bool,
    },

    /// Download package files and metadata according to the image
    Get {
        /// Cache image name (required)
        name: String,
    },

    /// Upload packages and install on remote system
    Install {
        /// Cache image name (required)
        name: String,

        /// Remote target SSH (user@host)
        #[arg(long)]
        target: String,
    },

    /// Update remote apt lists from cached sources and run upgrade
    Update {
        /// Cache image name (required)
        name: String,

        /// Remote target SSH (user@host)
        #[arg(long)]
        target: String,
    },

    /// Upgrade installed packages on remote system using cached metadata
    Upgrade {
        /// Cache image name (required)
        name: String,

        /// Remote target SSH (user@host)
        #[arg(long)]
        target: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set {
            name,
            target,
            install,
            update,
        } => set::run(&name, &target, install, update)?,

        Commands::Get { name } => get::run(&name)?,

        Commands::Install { name, target } => install::run(&name, &target)?,

        Commands::Update { name, target } => update::run(&name, &target)?,

        Commands::Upgrade { name, target } => upgrade::run(&name, &target)?,
    }

    Ok(())
}