use crate::uri::{UriFile, RemoteMode};

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use reqwest::blocking::Client;
use xz2::read::XzDecoder;

use std::fs::{self, File};
use std::path::Path;
use std::io::{BufReader, BufWriter, Write};
use std::sync::Arc;
use std::time::Duration;

#[derive(Args)]
pub struct GetArgs {
    /// Cache image name (required)
    name: String,
}

pub fn run(args: GetArgs) -> Result<()> {
    let name = &args.name;

    let cache_dir = dirs::cache_dir()
        .context("Failed to locate cache directory")?
        .join("apt-remote")
        .join(name);

    let uri_file_path = cache_dir.join("uri.toml");
    let uri_file = UriFile::load(&uri_file_path).context("Failed to load uri.toml metadata")?;
    
    let dir = match uri_file.mode {
        RemoteMode::Install | RemoteMode::Upgrade => "debs",
        RemoteMode::Update => "sources",
    };
    let download_dir = cache_dir.join(dir);
    fs::create_dir_all(&download_dir)?;

    let client = Arc::new(
        Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .context("Failed to build client")?,
    );

    let progress = Arc::new(MultiProgress::new());

    let progress_overall = progress.add(ProgressBar::new(uri_file.packages.len() as u64));
    progress_overall.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] {msg} [{wide_bar:.bold.cyan}] {pos}/{len} ({eta} remaining)",
            )
            .unwrap()
            .progress_chars("##-"),
    );
    progress_overall.enable_steady_tick(Duration::from_millis(100));
    progress_overall.set_message(format!("Downloading {name}..."));

    uri_file
        .packages
        .par_iter()
        .try_for_each(|(fname, pkg)| -> Result<()> {
            let dest = download_dir.join(fname);

            if dest.exists() {
                return Ok(()); // Already downloaded
            }

            let client = Arc::clone(&client);
            let progress = Arc::clone(&progress);
            let progress_overall = progress_overall.clone();

            let spinner = progress.add(ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::with_template("\t{spinner:.bold.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            spinner.set_message(format!("{} {}", "Downloading".cyan().bold(), fname.bold()));
            spinner.enable_steady_tick(std::time::Duration::from_millis(80));

            let response = client.get(&pkg.uri).send();

            if let Err(e) = response {
                spinner.finish_with_message(format!(
                    "{} {}:\n{}",
                    "✗".red().bold(),
                    format!("Failed to download {}", fname).red(),
                    e.to_string().dimmed()
                ));
                return Ok(());
            }

            let response = response?.error_for_status();

            if let Err(e) = response {
                if uri_file.mode == RemoteMode::Install {
                    spinner.finish_with_message(format!(
                        "{} {}:\n{}",
                        "✗".red().bold(),
                        format!("Bad response for {}", name).red(),
                        e.to_string().dimmed()
                    ));
                }
                return Ok(());
            }

            let extension = dest.extension().unwrap().to_str().unwrap();
            let mut file = File::create(&dest)?;
            file.write_all(&response?.bytes()?)?;

            if uri_file.mode == RemoteMode::Update && extension == "xz" {
                // Uncompress .xy files
                spinner.set_message(format!("{} {}", "Uncompressing".cyan().bold(), fname.bold()));
                 
                let original_path = Path::new(&dest);
                let output_path = original_path.with_extension(""); // removes .xz

                let input_file = File::open(&original_path)?;
                
                let mut decoder = XzDecoder::new_multi_decoder(BufReader::new(input_file));

                let output_file = File::create(&output_path)?;
                let mut writer = BufWriter::new(output_file);

                std::io::copy(&mut decoder, &mut writer)?;

                // Remove original .xz file
                std::fs::remove_file(&original_path)?;
            }

            spinner.finish_and_clear();
            progress_overall.inc(1);
            Ok(())
        })?;

    progress_overall.finish_with_message(format!(
        "{} {}",
        "✓".green().bold(),
        format!("Downloaded {}", name).green()
    ));
    
    println!("\n");
    Ok(())
}
