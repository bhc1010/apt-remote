use anyhow::{Context, Result};
use std::fs;

pub fn run() -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to locate cache directory")?
        .join("apt-remote");

    for entry in fs::read_dir(cache_dir)? {
        fs::remove_dir_all(entry?.path())?;
    }

    Ok(())
}
