//! # URI and Package Metadata Handling for apt-remote
//!
//! This module defines data structures for representing package sources,
//! download metadata, and integrity checks. It also provides utilities for
//! loading and saving `uri.toml` files, as well as validating package URIs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

/// The type of checksum used to verify package integrity.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChecksumKind {
    /// SHA256 checksum.
    SHA256,
    /// MD5 checksum.
    MD5,
}

impl ChecksumKind {
    /// Create a new [`ChecksumKind`] from a string identifier.
    ///
    /// # Arguments
    /// * `kind_str` - The string name of the checksum type
    ///   (e.g., `"sha256sum"` or `"md5sum"`).
    ///
    /// # Errors
    /// Returns an error if the provided string does not match a known checksum type.
    pub fn new(kind_str: &str) -> Result<ChecksumKind> {
        // Map common checksum tool output names to enum variants
        let kind = match kind_str {
            "sha256sum" => ChecksumKind::SHA256,
            "md5sum" => ChecksumKind::MD5,
            _ => return Err(anyhow::anyhow!("Checksum not valid")),
        };
        Ok(kind)
    }
}

/// A checksum record for a package.
#[derive(Debug, Serialize, Deserialize)]
pub struct Checksum {
    /// The checksum algorithm.
    pub kind: ChecksumKind,
    /// The actual checksum value (hex-encoded).
    pub value: String,
}

/// Information about a single package entry in the `uri.toml` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct PackageEntry {
    /// The source URI for downloading the package.
    pub uri: String,
    /// The size of the package file in bytes.
    pub size: u64,
    /// Optional checksum for verifying file integrity.
    pub checksum: Option<Checksum>,
}

/// The mode of operation for remote installation.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RemoteMode {
    /// Install packages on the remote host.
    Install,
    /// Update the package index on the remote host.
    Update,
    /// Upgrade existing packages on the remote host.
    Upgrade,
}

/// Representation of the full `uri.toml` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct UriFile {
    /// The remote operation mode.
    pub mode: RemoteMode,
    /// The architecture for which the packages are intended.
    pub arch: String,
    /// The total size of all packages (optional).
    pub total_size: Option<u64>,
    /// The order in which packages should be installed.
    pub install_order: Vec<String>,
    /// Mapping of package name → package metadata.
    pub packages: HashMap<String, PackageEntry>,
}

impl UriFile {
    /// Load a `UriFile` from disk, validating URIs as it parses.
    ///
    /// # Arguments
    /// * `path` - Path to the TOML file.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read.
    /// - TOML parsing fails.
    /// - One or more package URIs are invalid.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Read the TOML file into a string
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;

        // Deserialize the TOML content into a UriFile struct
        let parsed: UriFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from {}", path.as_ref().display()))?;

        // Validate that each package URI uses a supported scheme
        for (pkg_name, pkg) in &parsed.packages {
            validate_uri(&pkg.uri)
                .with_context(|| format!("Invalid URI for package {}: {}", pkg_name, pkg.uri))?;
        }

        Ok(parsed)
    }

    /// Save the `UriFile` to disk as a TOML file.
    ///
    /// # Arguments
    /// * `path` - Destination file path.
    ///
    /// # Errors
    /// Returns an error if serialization fails or the file cannot be written.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Serialize this struct into a TOML string
        let toml_str =
            toml::to_string(self).context("Failed to serialize UriFile to TOML")?;

        // Write the TOML string to the specified path
        fs::write(&path, toml_str)
            .with_context(|| format!("Failed to write to {}", path.as_ref().display()))?;
        Ok(())
    }
}

/// Validate that a URI is well-formed and uses a supported scheme.
///
/// # Supported Schemes
/// - `http`
/// - `https`
/// - `ftp`
///
/// # Errors
/// Returns an error if the URI is malformed or uses an unsupported scheme.
fn validate_uri(uri: &str) -> Result<()> {
    // Attempt to parse the URI
    let parsed = Url::parse(uri).with_context(|| format!("Failed to parse URI: {uri}"))?;

    // Only allow certain protocols
    if !["http", "https", "ftp"].contains(&parsed.scheme()) {
        anyhow::bail!("Unsupported scheme: {}", parsed.scheme());
    }

    Ok(())
}
