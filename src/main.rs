mod cli;
mod compactor;
mod config;
mod detector;
mod error;
mod json_processor;
mod processor;
mod store;
mod tokenizer;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use std::fs;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config (D-07: walk up from CWD, or use --config override)
    let cfg = if let Some(config_path) = &cli.config {
        config::load_config(config_path)?
    } else if let Some(found) = config::find_config() {
        config::load_config(&found)?
    } else {
        config::LoktokConfig::default()
    };

    // Handle --reset-store: delete store and exit (D-14)
    if cli.reset_store {
        let store_dir = std::env::current_dir()?.join(".logtok");
        let store = store::Store::new(&store_dir)?;
        store.reset()?;
        eprintln!("logtok: token store reset");
        return Ok(());
    }

    // Require file for all other operations
    let file = cli
        .file
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("log file path is required (unless using --reset-store)"))?;

    // Validate input file exists and is readable
    let metadata = fs::metadata(file)
        .with_context(|| format!("Cannot access file: {}", file.display()))?;

    if !metadata.is_file() {
        anyhow::bail!("Not a regular file: {}", file.display());
    }

    // Validate block size range (1KB to 100MB)
    if cli.block_size < 1024 || cli.block_size > 104_857_600 {
        anyhow::bail!(
            "Invalid block size: {} (must be between 1024 and 104857600)",
            cli.block_size
        );
    }

    // Build detection config from TOML
    let detection_config = cfg.to_detection_config();

    // Determine store directory
    let store_dir = std::env::current_dir()?.join(".logtok");

    // Load or create store (only if LOGTOK_KEY is set)
    // If LOGTOK_KEY is not set, proceed without store (in-memory only) (T-02-13)
    let store_result = store::Store::new(&store_dir);

    // Run pipeline
    processor::process_file_with_config(
        file,
        cli.output.as_deref(),
        cli.block_size,
        cli.quiet,
        cli.dry_run,
        &detection_config,
        store_result.ok().as_ref(),
        cfg.ttl_seconds(),
    )?;

    Ok(())
}
