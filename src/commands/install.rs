use crate::ssh::{RemoteExecutor, SecureUpload, create_ssh_session};
use crate::uri::{ChecksumKind, UriFile, RemoteMode};

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ssh2::Session;

use std::{path::Path, time::Duration};

#[derive(Args)]
#[command(override_usage="apt-remote install <NAME> --target <user@host>")]
pub struct InstallArgs {
    /// Cache image name (required)
    name: String,

    /// Remote target SSH (user@host)
    #[arg(short, long)]
    target: String,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let name = &args.name;
    let target = &args.target;

    let session = create_ssh_session(target)?;
    let user = session.exec("whoami")?;
    let user = user.trim();
    let password = rpassword::prompt_password(format!("[sudo] password for {}: ", user))
        .ok()
        .unwrap();

    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache dir")?
        .join("apt-remote")
        .join(name);
    let mut uri_file = UriFile::load(&cache_dir.join("uri.toml"))
        .context("Failed to load uri.toml metadata")?;

    if uri_file.mode == RemoteMode::Update {
        println!("This uri file is in update mode: please run 'apt-remote update <NAME> --target <user@host>");
        return Ok(());
    }

    let remote_str = format!("/tmp/apt-remote/{name}");
    let remote_path = Path::new(&remote_str);
    session.exec(&format!("mkdir -p {}", remote_str))?;
    session.exec(&format!("cd {}", remote_str))?;

    let progress = MultiProgress::new();

    // Upload the archive
    upload_archive(
        &session,
        name,
        &user,
        &mut uri_file,
        &cache_dir,
        &remote_path,
        &progress,
    )?;

    // Perform checksum verification on remote system
    if let Err(err) = verify_remote_checksums(&session, &mut uri_file, &remote_path, &progress) {
        session.exec("cd $HOME")?;
        return Err(err);
    }

    // Install the packages
    install_archive(
        &session,
        &password,
        &name,
        &mut uri_file,
        &remote_path,
        &progress,
    )?;

    // Cleanup
    session.sudo(
        &format!(
            "mv {} /var/cache/apt/archives",
            remote_path.join("*").to_str().unwrap()
        ),
        &password,
    )?;
    session.exec(&format!("rm -rf {remote_str}"))?;

    Ok(())
}

fn upload_archive(
    session: &Session,
    name: &str,
    user: &str,
    uri_file: &mut UriFile,
    cache_dir: &Path,
    remote_path: &Path,
    progress: &MultiProgress,
) -> Result<()> {
    let progress_upload = progress.add(ProgressBar::new(uri_file.packages.len() as u64));
    progress_upload.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {msg:25} [{wide_bar:.bold.cyan}] {pos}/{len} ({eta} remaining)",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    progress_upload.enable_steady_tick(Duration::from_millis(100));
    progress_upload.set_message(format!("Uploading {name} to {user}..."));

    let archive_path = cache_dir.join("debs");

    // Upload files to remote device
    uri_file
        .packages
        .iter()
        .for_each(|(fname, _)| {
            let spinner = progress.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::with_template("\t{spinner:.bold.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
            );
            spinner.enable_steady_tick(Duration::from_millis(100));
            spinner.set_message(format!("{fname}"));

            let file_path = archive_path.join(fname);
            let status = session.scp_upload(&file_path, &remote_path.join(fname));

            if let Err(e) = status {
                spinner.finish_with_message(format!(
                    "{} {}: {}",
                    "✗".red().bold(),
                    format!("File not sent: {fname}").red(),
                    e.to_string().dimmed()
                ));
            }

            spinner.finish_and_clear();
            progress_upload.inc(1);
    });

    progress_upload.finish_with_message(format!(
        "{} {}",
        "✓".green().bold(),
        format!("Uploaded {name}").green()
    ));
    Ok(())
}

fn verify_remote_checksums(
    session: &ssh2::Session,
    uri_file: &mut UriFile,
    remote_path: &Path,
    progress: &MultiProgress,
) -> Result<()> {
    let progress_verify = progress.add(ProgressBar::new(uri_file.packages.len() as u64));
    progress_verify.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {msg:25} [{wide_bar:.bold.cyan}] {pos}/{len} ({eta} remaining)",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    progress_verify.enable_steady_tick(Duration::from_millis(100));
    progress_verify.set_message(format!("Verifying checksums..."));

    // Perform checksum verification on remote system
    let mut mismatches = Vec::new();

    for (fname, pkg_info) in progress_verify.wrap_iter(&mut uri_file.packages.iter()) {
        let spinner = progress.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::with_template("\t{spinner:.bold.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(format!("{fname}"));

        let remote_path = remote_path.join(fname);
        let expected_checksum = pkg_info.checksum.as_ref().unwrap().value.clone();

        let checksum = match pkg_info.checksum.as_ref().unwrap().kind {
            ChecksumKind::SHA256 => "sha256sum",
            ChecksumKind::MD5 => "md5sum",
        };

        let output = session
            .exec(&format!("{checksum} {}", remote_path.to_str().unwrap()))
            .context(format!("Failed to compute {checksum} for {fname}"))?;

        let actual_checksum = output
            .split_whitespace()
            .next()
            .unwrap_or("ERROR: checksum output unwrap failed.")
            .to_string();

        if actual_checksum != expected_checksum {
            mismatches.push((fname, expected_checksum, actual_checksum));
            spinner.finish_with_message(format!(
                "{} {}",
                "✗".red().bold(),
                format!("File not sent: {fname}").red()
            ));
        } else {
            spinner.finish_and_clear();
        }
    }

    if mismatches.is_empty() {
        progress_verify.finish_with_message(format!(
            "{} {}",
            "✓".green().bold(),
            "Checksums verified".green()
        ));
        Ok(())
    } else {
        Err(anyhow::anyhow!("Remote checksum verification failed"))
    }
}

fn install_archive(
    session: &Session,
    password: &str,
    name: &str,
    uri_file: &mut UriFile,
    remote_path: &Path,
    progress: &MultiProgress,
) -> Result<()> {
    let progress_install = progress.add(ProgressBar::new(uri_file.packages.len() as u64));
    progress_install.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {msg:25} [{wide_bar:.bold.cyan}] {pos}/{len} ({eta} remaining)",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    progress_install.set_message(format!("Installing {name}..."));
    progress_install.enable_steady_tick(Duration::from_millis(100));

    for fname in progress_install.wrap_iter(&mut uri_file.install_order.iter()) {
        let spinner = progress.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::with_template("\t{spinner:.bold.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(format!("{fname}"));

        let status = session
            .sudo(
                &format!("dpkg -i {}", remote_path.join(fname).to_str().unwrap()),
                password,
            )
            .context("dpkg install failed");

        if let Err(e) = status {
            spinner.finish_with_message(format!(
                "{} {}: {}",
                "✗".red().bold(),
                format!("File not installed: {fname}").red(),
                e.to_string().dimmed()
            ));
        }

        spinner.finish_and_clear();
    }

    // Reconfigure with dpkg
    progress_install.set_message(format!("Reconfiguring {name}"));
    if let Err(e) = session.sudo("dpkg --configure -a", &password) {
        progress_install.finish_with_message(format!(
            "{} {}: {}",
            "✗".red().bold(),
            format!("dpkg failed to reconfigure").red(),
            e.to_string().dimmed()
        ));
    } else {
        progress_install.finish_with_message(format!(
            "{} {}",
            "✓".green().bold(),
            format!("Installed and configured {name}").green()
        ));
    }
    println!("\n");
    Ok(())
}
