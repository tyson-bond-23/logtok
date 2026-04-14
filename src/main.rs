mod cli;
mod compactor;
mod detector;
mod error;
mod json_processor;
mod processor;
mod tokenizer;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use std::fs;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate input file exists and is readable
    let metadata = fs::metadata(&cli.file)
        .with_context(|| format!("Cannot access file: {}", cli.file.display()))?;

    if !metadata.is_file() {
        anyhow::bail!("Not a regular file: {}", cli.file.display());
    }

    // Validate block size range (1KB to 100MB)
    if cli.block_size < 1024 || cli.block_size > 104_857_600 {
        anyhow::bail!(
            "Invalid block size: {} (must be between 1024 and 104857600)",
            cli.block_size
        );
    }

    // Run the processing pipeline
    processor::process_file(
        &cli.file,
        cli.output.as_deref(),
        cli.block_size,
        cli.quiet,
    )?;

    Ok(())
}
