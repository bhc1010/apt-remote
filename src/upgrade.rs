use crate::ssh::{create_ssh_session, RemoteExecutor};
use anyhow::{Context, Result};
use std::path::Path;

pub fn upgrade(name: &str, target: &str) -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache dir")?
        .join("apt-remote")
        .join(name);

    // Local directory holding Packages.gz files (apt metadata)
    let sources_list_dir = cache_dir.join("sources");

    // Bail early if sources directory doesn't exist
    if !sources_list_dir.exists() {
        anyhow::bail!("Sources cache directory not found: {:?}", sources_list_dir);
    }

    let session = create_ssh_session(target)
        .context("Failed to create SSH session")?;

    // Only after verifying local sources exist, clean remote lists folder
    session.exec("sudo rm -rf /var/lib/apt/lists")
        .context("Failed to remove /var/lib/apt/lists on remote")?;

    session.exec("sudo mkdir -p /var/lib/apt/lists")
        .context("Failed to create /var/lib/apt/lists on remote")?;

    session.scp_send_dir(&sources_list_dir, "/var/lib/apt/lists")
        .context("Failed to SCP Packages.gz files to remote /var/lib/apt/lists")?;

    session.exec("sudo apt-get update")
        .context("Failed to run apt-get update on remote")?;

    session.exec("sudo apt-get upgrade -y")
        .context("Failed to run apt-get upgrade on remote")?;

    Ok(())
}