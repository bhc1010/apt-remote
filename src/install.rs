use crate::model::UriFile;
use crate::ssh::{create_ssh_session, RemoteExecutor};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use tempfile::tempdir;

pub fn run(name: &str, target: &str) -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache dir")?
        .join("apt-remote")
        .join(name);

    let uri_file = UriFile::load(&cache_dir.join("uri.toml"))
        .context("Failed to load uri.toml metadata")?;

    let archive_path = cache_dir.join("packages.tar.gz");

    let session = create_ssh_session(target)?;

    // Upload the archive
    let remote_tmp_dir = "/tmp/apt-remote-debs";
    let remote_archive = format!("{}/debs.tar.gz", remote_tmp_dir);

    session.exec(&format!("rm -rf {remote_tmp_dir}"))?;
    session.exec(&format!("mkdir -p {remote_tmp_dir}"))?;
    session.scp_send(&remote_archive, &archive_path)?;

    // Extract the archive remotely
    session.exec(&format!(
        "cd {remote_tmp_dir} && tar -xzf debs.tar.gz"
    ))?;

    // Perform SHA256 verification on remote system
    let remote_deb_dir = format!("{}/debs", remote_tmp_dir);
    verify_remote_checksums(&session, &uri_file, &remote_deb_dir)?;

    // Install the packages
    session.exec(&format!("sudo dpkg -i {remote_deb_dir}/*.deb"))
        .context("dpkg install failed")?;

    // Cleanup
    session.exec(&format!("rm -rf {remote_tmp_dir}"))?;

    Ok(())
}

fn verify_remote_checksums(session: &ssh2::Session, uri_file: &UriFile, remote_dir: &str) -> Result<()> {
    let mut mismatches = Vec::new();

    for (pkg_name, pkg_info) in &uri_file.packages {
        let file_name = Path::new(&pkg_info.uri)
            .file_name()
            .context("Missing filename")?
            .to_str()
            .context("Invalid UTF-8 in filename")?;

        let remote_path = format!("{}/{}", remote_dir, file_name);
        let expected_sha = pkg_info
            .sha
            .as_ref()
            .context("Missing sha256 for package")?;

        let output = session.exec(&format!("sha256sum {}", remote_path))
            .context(format!("Failed to compute sha256 for {}", file_name))?;
        let actual_sha = output.split_whitespace().next().unwrap_or("").to_string();

        if actual_sha != *expected_sha {
            mismatches.push((file_name.to_string(), expected_sha.clone(), actual_sha));
        }
    }

    if mismatches.is_empty() {
        Ok(())
    } else {
        eprintln!("Checksum mismatches:");
        for (file, expected, actual) in mismatches {
            eprintln!("  - {file}: expected {expected}, got {actual}");
        }
        Err(anyhow::anyhow!("Remote SHA256 verification failed"))
    }
}
