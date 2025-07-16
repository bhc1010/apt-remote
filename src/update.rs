use anyhow::{Context, Result};
use ssh2::Session;
use std::fs;
use std::path::PathBuf;

use crate::ssh::{create_ssh_session, RemoteExecutor};

pub fn update(name: &str, target: &str) -> Result<()> {
    let source_dir = dirs::cache_dir()
        .context("Failed to get cache directory")?
        .join("apt-remote")
        .join(name)
        .join("sources");

    if !source_dir.exists() {
        return Err(anyhow::anyhow!(
            "No sources metadata found for image '{}'",
            name
        ));
    }

    let session = create_ssh_session(target)?;

    // Ensure the remote lists directory is clean
    session.exec("sudo rm -rf /var/lib/apt/lists")?;
    session.exec("sudo mkdir -p /var/lib/apt/lists/partial")?;

    // Transfer all *.gz files to the remote lists directory
    for entry in fs::read_dir(&source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("gz") {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .context("Invalid filename")?;

            let remote_path = format!("/var/lib/apt/lists/{}", file_name);
            session.scp_send(&remote_path, &path)?;
        }
    }

    // Run apt-get update (this will just read the transferred files)
    session.exec("sudo apt-get update")?;

    Ok(())
}