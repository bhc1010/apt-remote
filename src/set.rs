use anyhow::{Context, Result};
use ssh2::Session;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::{
    model::{PackageInfo, UriFile},
    ssh::{create_ssh_session, RemoteExecutor},
};

pub fn set(name: &str, target: &str, packages: &[String], update: bool) -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache directory")?
        .join("apt-remote")
        .join(name);
    fs::create_dir_all(&cache_dir)?;

    let session = create_ssh_session(target)?;

    let mut uri_file = UriFile {
        arch: None,
        packages: Default::default(),
    };
    
    let arch = session.exec("dpkg --print-architecture")?.trim().to_string();
    uri_file.arch = Some(arch);
    
    if update {
        let sources_dir = cache_dir.join("sources");
        fs::create_dir_all(&sources_dir)?;

        session
            .scp_recv("/etc/apt/sources.list", &sources_dir.join("sources.list"))
            .context("Failed to scp_recv /etc/apt/sources.list")?;

        session
            .scp_recv("/etc/apt/sources.list.d", &sources_dir.join("sources.list.d"))
            .context("Failed to scp_recv /etc/apt/sources.list.d")?;
    }

    // 3. Query for package URIs
    if !packages.is_empty() {
        let pkg_list = packages.join(" ");
        let output = session.exec(&format!("apt-get install --print-uris -y {}", pkg_list))?;

        for line in output.lines() {
            if let Some(start) = line.find('\'') {
                let rest = &line[start + 1..];
                if let Some(end) = rest.find('\'') {
                    let uri = &rest[..end];
                    if let Some(file_name) = uri.split('/').last() {
                        let sha = line
                            .split("SHA256:")
                            .nth(1)
                            .and_then(|s| s.split_whitespace().next())
                            .map(|s| s.to_string());

                        uri_file.packages.insert(
                            file_name.to_string(),
                            PackageInfo {
                                uri: uri.to_string(),
                                sha,
                            },
                        );
                    }
                }
            }
        }
    }

    // 4. Save uri.toml
    let uri_path = cache_dir.join("uri.toml");
    uri_file.save(&uri_path)?;

    Ok(())
}