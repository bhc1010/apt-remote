use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub enum ChecksumKind {
    SHA256,
    MD5,
}

impl ChecksumKind {
    pub fn new(kind_str: &str) -> Result<ChecksumKind> {
        let kind = match kind_str {
            "sha256sum" => ChecksumKind::SHA256,
            "md5sum" => ChecksumKind::MD5,
            _ => return Err(anyhow::anyhow!("Checksum not valid")),
        };
        Ok(kind)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Checksum {
    pub kind: ChecksumKind,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageEntry {
    pub uri: String,
    pub size: u64,
    pub checksum: Option<Checksum>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RemoteMode {
    Install,
    Update,
    Upgrade,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UriFile {
    pub mode: RemoteMode,
    pub arch: String,
    pub total_size: Option<u64>,
    pub install_order: Vec<String>,
    pub packages: HashMap<String, PackageEntry>,
}

impl UriFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;

        let parsed: UriFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from {}", path.as_ref().display()))?;

        // Validate URIs
        for (pkg_name, pkg) in &parsed.packages {
            validate_uri(&pkg.uri)
                .with_context(|| format!("Invalid URI for package {}: {}", pkg_name, pkg.uri))?;
        }

        Ok(parsed)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_str =
            toml::to_string(self).context("Failed to serialize UriFile to TOML")?;
        fs::write(&path, toml_str)
            .with_context(|| format!("Failed to write to {}", path.as_ref().display()))?;
        Ok(())
    }
}

fn validate_uri(uri: &str) -> Result<()> {
    let parsed = Url::parse(uri).with_context(|| format!("Failed to parse URI: {uri}"))?;

    if !["http", "https", "ftp"].contains(&parsed.scheme()) {
        anyhow::bail!("Unsupported scheme: {}", parsed.scheme());
    }

    Ok(())
}
