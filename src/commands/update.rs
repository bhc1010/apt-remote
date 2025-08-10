use crate::ssh::{create_ssh_session, RemoteExecutor, SecureUpload};

use anyhow::{Context, Result};
use clap::Args;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use colored::Colorize;

use std::time::Duration;
use std::path::Path;

#[derive(Args)]
#[command(override_usage="apt-remote install <NAME> --target <user@host>")]
pub struct UpdateArgs{
    /// Cache image name (required)
    name: String,

    /// Remote target SSH (user@host)
    #[arg(short, long)]
    target: String,
}

pub fn run(args: UpdateArgs) -> Result<()> {
    let name = &args.name;
    let target = &args.target;
    let user = target.split("@").nth(0).unwrap().trim();

    let session = create_ssh_session(target)?;
    let password = rpassword::prompt_password(format!("[sudo] password for {user}: "))
        .ok()
        .unwrap();

    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache dir")?
        .join("apt-remote")
        .join(name);

    let remote_str = format!("/tmp/apt-remote/{name}");
    let remote_path = Path::new(&remote_str);

    let source_path = cache_dir.join("sources");
    if !source_path.exists() {
        return Err(anyhow::anyhow!(
            "No sources metadata found for image '{}'",
            name
        ));
    }

    let src_paths = source_path.read_dir()?;
    let srcs = src_paths
        .filter_map(|entry| {
            entry.ok().and_then(|e|
                e.path().file_name()
                 .and_then(|n| n.to_str().map(|s| String::from(s)))
            )
        }).collect::<Vec<String>>();

    // Ensure the remote lists directory is clean
    session.exec(&format!("mkdir -p {remote_str}"))?;
    session.sudo("mv /var/lib/apt/lists /var/lib/apt/lists.old", &password)?;
    session.sudo("mkdir -p /var/lib/apt/lists/partial", &password)?;
    session.sudo("touch /var/lib/apt/lists/lock", &password)?;

    let progress = MultiProgress::new();

    let progress_overall = progress.add(ProgressBar::new(srcs.len() as u64));
    progress_overall.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {msg} [{wide_bar:.bold.cyan}] {pos}/{len} ({eta} remaining)",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    progress_overall.enable_steady_tick(Duration::from_millis(100));
    progress_overall.set_message(format!("Uploading package metadata to {target}..."));

    // Transfer all source files to the remote lists directory
    srcs.iter()
        .for_each(|fname| {
            let spinner = progress.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::with_template("\t{spinner:.bold.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
            );
            spinner.enable_steady_tick(Duration::from_millis(100));

            let local_fpath = source_path.join(fname);
            let remote_fpath = remote_path.join(fname);
            if !local_fpath.exists() {
                return
            }

            spinner.set_message(format!("{}", local_fpath.file_name().unwrap().to_str().unwrap()));
            let status = session.scp_upload(&local_fpath, &remote_fpath);

            if let Err(e) = status {
                spinner.finish_with_message(format!(
                    "{} {}: {}",
                    "✗".red().bold(),
                    format!("File not sent: {fname}").red(),
                    e.to_string().dimmed()
                ));
            }

            spinner.finish_and_clear();
            progress_overall.inc(1);
    });

    // Move lists and generate pkgcache.bin
    progress_overall.set_message("Generating cache...");
    session.sudo(&format!("mv {remote_str}/* /var/lib/apt/lists"), &password)?;
    session.sudo("apt-cache gencaches", &password)?;
    progress_overall.finish_with_message(format!("{} {}", "✓ Updated".green().bold(), target.green().bold()));

    Ok(())
}
