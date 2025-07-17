use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct PackageEntry {
    pub uri: String,
    pub sha: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SourceEntry {
    pub uri: String,
    pub sha: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UriFile {
    pub arch: String,
    pub packages: BTreeMap<String, PackageEntry>,

    #[serde(default)]
    pub sources: BTreeMap<String, BTreeMap<String, SourceEntry>>,
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
                .with_context(|| format!("Invalid URI for package {pkg_name}: {}", pkg.uri))?;
        }

        for (suite, components) in &parsed.sources {
            for (component, entry) in components {
                validate_uri(&entry.uri).with_context(|| {
                    format!(
                        "Invalid URI for source [{suite}.{component}]: {}",
                        entry.uri
                    )
                })?;
            }
        }

        Ok(parsed)
    }
}

fn validate_uri(uri: &str) -> Result<()> {
    let parsed = Url::parse(uri)
        .with_context(|| format!("Failed to parse URI: {uri}"))?;

    if !["http", "https", "ftp"].contains(&parsed.scheme()) {
        anyhow::bail!("Unsupported scheme: {}", parsed.scheme());
    }

    Ok(())
}