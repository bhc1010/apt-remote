use crate::model::UriFile;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use url::Url;

pub fn run(name: &str) -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to locate cache directory")?
        .join("apt-remote")
        .join(name);
    let sources_dir = cache_dir.join("sources");

    if sources_dir.exists() {
        let urls = extract_source_urls(&sources_dir)?;
        download_packages_metadata(&urls, &cache_dir)?;
    }

    let uri_file_path = cache_dir.join("uri.toml");
    let uri_file = UriFile::load(&uri_file_path)
        .context("Failed to load uri.toml metadata")?;

    let download_dir = cache_dir.join("debs");
    fs::create_dir_all(&download_dir)?;

    let client = Client::new();
    uri_file
        .packages
        .par_iter()
        .try_for_each(|(_, pkg)| -> Result<()> {
            let file_name = Path::new(&pkg.uri)
                .file_name()
                .context("Missing filename")?;
            let dest = download_dir.join(file_name);

            if dest.exists() {
                return Ok(()); // Already downloaded
            }

            let response = client
                .get(&pkg.uri)
                .send()
                .context("Failed to send request for deb")?
                .error_for_status()
                .context("Bad response for deb download")?;
            let mut file = File::create(&dest)?;
            file.write_all(&response.bytes()?)?;
            Ok(())
        })?;

    // Create tar.gz archive
    let archive_path = cache_dir.join("packages.tar.gz");
    let archive_file = File::create(&archive_path)?;
    let mut builder = Builder::new(flate2::write::GzEncoder::new(
        archive_file,
        flate2::Compression::default(),
    ));
    builder.append_dir_all("debs", &download_dir)?;
    builder.finish()?;

    Ok(())
}

fn extract_source_urls(sources_dir: &Path) -> Result<HashSet<Url>> {
    let mut urls = HashSet::new();

    for entry in fs::read_dir(sources_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file = File::open(&path)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                if line.starts_with("deb ") {
                    let parts: Vec<_> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(url) = Url::parse(parts[1]) {
                            urls.insert(url);
                        }
                    }
                }
            }
        }
    }

    Ok(urls)
}

fn download_packages_metadata(urls: &HashSet<Url>, cache_dir: &Path) -> Result<()> {
    let client = Client::new();
    let lists_dir = cache_dir.join("lists");
    fs::create_dir_all(&lists_dir)?;

    urls.par_iter().try_for_each(|url| {
        let mut pkg_url = url.clone();
        if !pkg_url.path().ends_with('/') {
            pkg_url.set_path(&format!("{}/", pkg_url.path()));
        }
        pkg_url.set_path(&format!("{}Packages.gz", pkg_url.path()));

        let file_name = pkg_url
            .path_segments()
            .and_then(|s| s.last())
            .context("Invalid Packages.gz path")?;
        let dest_path = lists_dir.join(file_name);

        if dest_path.exists() {
            return Ok(()); // Skip if already downloaded
        }

        let resp = client
            .get(pkg_url.as_str())
            .send()
            .context("Failed to send request")?
            .error_for_status()
            .context("Request failed")?;

        let mut file = fs::File::create(&dest_path)
            .with_context(|| format!("Failed to create file {:?}", dest_path))?;
        let content = resp.bytes().context("Failed to read response bytes")?;
        file.write_all(&content)?;

        Ok(())
    })?;

    Ok(())
}