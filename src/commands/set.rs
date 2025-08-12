//! # `apt-remote set` command
//!
//! This subcommand generates a `uri.toml` file for a specified cache image,
//! based on package information retrieved from a remote host via SSH.
//!
//! It supports installing packages, fixing broken dependencies,
//! updating package lists, and upgrading packages.

use crate::{
    ssh::{RemoteExecutor, create_ssh_session},
    uri::{Checksum, ChecksumKind, PackageEntry, UriFile, RemoteMode},
};

use anyhow::{Context, Result};
use clap::{ArgGroup, Args};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use std::{fs, collections::HashMap};

/// CLI arguments for the `apt-remote set` subcommand.
#[derive(Args)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(&["install", "fix", "update", "upgrade"])
        .multiple(false),
    ),
    override_usage = "apt-remote set <NAME> --target <user@host> (--install <packages...> | --fix | --update | --upgrade)",
)]
pub struct SetArgs {
    /// Cache image name (required)
    name: String,

    /// Remote target SSH (user@host)
    #[arg(short, long)]
    target: String,

    /// Packages to install
    #[arg(short, long, value_parser, num_args=1.., value_delimiter = ' ')]
    install: Vec<String>,

    /// Flag to run "apt-get --fix-broken"
    #[arg(short, long)]
    fix: bool,

    /// Flag to run "apt-get update"
    #[arg(long)]
    update: bool,

    /// Get upgradable packages
    #[arg(long)]
    upgrade: bool,
}

/// Executes the `set` subcommand.
///
/// Connects to a remote host, retrieves package URIs, sizes, and checksums,
/// and saves them in a `uri.toml` file inside the local cache.
///
/// # Errors
/// Returns an error if SSH connection fails, the remote command fails,
/// or if writing `uri.toml` fails.
pub fn run(args: SetArgs) -> Result<()> {
    // Extract basic args
    let name = &args.name;
    let target = &args.target;
    let packages = &args.install;

    // Determine operation mode
    let mode = if args.update {
        RemoteMode::Update
    } else if args.upgrade {
        RemoteMode::Upgrade
    } else {
        RemoteMode::Install
    };

    // Create cache directory for this image
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache directory")?
        .join("apt-remote")
        .join(name);
    fs::create_dir_all(&cache_dir)?;

    // Connect to the remote system
    let session = create_ssh_session(target)?;

    // Detect remote architecture
    let arch = session
        .exec("dpkg --print-architecture")?
        .trim()
        .to_string();

    // Set up progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner.set_message(format!("{}", "Getting package info...".cyan().bold()));
    spinner.enable_steady_tick(std::time::Duration::from_millis(200));

    // Build apt-get command string
    let mode_str = match mode {
        RemoteMode::Install => "install",
        RemoteMode::Update => "update",
        RemoteMode::Upgrade => "upgrade",
    };
    let verbosity = match mode {
        RemoteMode::Install | RemoteMode::Upgrade => "-qqq",
        RemoteMode::Update => "-q",
    };
    let fix = if args.fix { "-f" } else { "" };
    let pkg_list = packages.join(" ");
    let cmd = format!("apt-get {mode_str} --print-uris {verbosity} {fix} {pkg_list}");

    // Run command remotely and get output
    let output = session.exec(&cmd)?;

    spinner.finish();
    let mut total_size: u64 = 0;

    // Parse apt-get --print-uris output
    let pkg_data: Vec<Result<_>> = output
        .par_lines()
        .map(|line: &str| -> Result<_> {
            let mut parts = line.split(" ");

            // Extract URI
            let uri = parts.next().unwrap().replace("\'", "");

            // Extract filename from URI
            let filename = url::Url::parse(&uri)
                .ok()
                .and_then(|url| {
                    let segments = url.path_segments()?;
                    segments.last().map(|s| s.to_string())
                })
                .unwrap();

            // Skip "dest" field
            parts.next().unwrap();

            // Extract file size
            let size = parts.next().unwrap().parse::<u64>()?;

            // Extract checksum field
            let checksum_maybe = parts.next().unwrap().to_string();

            // Parse checksum if present
            let checksum = if checksum_maybe.is_empty() {
                None
            } else {
                let mut checksum_pair = checksum_maybe.split(":");
                let kind_str = checksum_pair.next().unwrap().to_string().to_lowercase();
                let kind = ChecksumKind::new(&kind_str)
                    .context(format!("{filename} has no valid checksum kind ({kind_str})"))?;
                let value = checksum_pair.next().unwrap().to_string();
                Some(Checksum { kind, value })
            };

            Ok((
                filename,
                PackageEntry {
                    uri,
                    size,
                    checksum,
                },
            ))
        })
        .collect::<Vec<Result<_>>>();

    // Prepare storage structures
    let mut install_order: Vec<String> = vec![];
    let mut packages: HashMap<String, PackageEntry> = Default::default();

    // Print summary
    let file_type = if args.update { "sources" } else { "packages" };
    println!("The following {} {} will be stored:\n", pkg_data.len(), file_type);

    // Store data depending on mode
    match mode {
        RemoteMode::Update => {
            for pkg_info in pkg_data {
                let (_, pkg_entry) = pkg_info?;
                println!("\t{}", pkg_entry.uri);
                packages.insert(pkg_entry.uri.split("//").nth(1).unwrap().replace("/", "_"), pkg_entry);
            }
        }
        RemoteMode::Install | RemoteMode::Upgrade => {
            for pkg_info in pkg_data {
                let (fname, pkg_entry) = pkg_info?;
                println!("\t{} ({})", fname, format_size(pkg_entry.size));
                total_size += pkg_entry.size;
                install_order.push(fname.clone());
                packages.insert(fname, pkg_entry);
            }
        }
    }

    // Only store total size if not update mode
    let total_size = if args.update { None } else { Some(total_size) };

    // Create UriFile struct
    let uri_file = UriFile {
        mode,
        arch,
        total_size,
        install_order,
        packages,
    };

    // Print total size if applicable
    if total_size.is_some() {
        println!("\nTotal size: {}", format_size(total_size.unwrap()));
    }
    println!("\n");

    // Save uri.toml in cache
    let uri_path = cache_dir.join("uri.toml");
    uri_file.save(&uri_path)?;

    Ok(())
}

/// Format byte sizes into KB, MB, or GB.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1000;
    const MB: u64 = KB * 1000;
    const GB: u64 = MB * 1000;

    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
        _ => format!("{} B", bytes),
    }
}
