mod cli;
mod clipboard;
mod compactor;
mod config;
mod detokenizer;
mod detector;
mod error;
mod json_processor;
mod processor;
mod store;
mod tokenizer;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use std::fs;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config (walk up from CWD, or use --config override)
    let cfg = if let Some(config_path) = &cli.config {
        config::load_config(config_path)?
    } else if let Some(found) = config::find_config() {
        config::load_config(&found)?
    } else {
        config::LoktokConfig::default()
    };

    match cli.command {
        Commands::Tokenize {
            file,
            output,
            clipboard,
            block_size,
            dry_run,
        } => {
            // Validate input file exists and is readable
            let metadata = fs::metadata(&file)
                .with_context(|| format!("Cannot access file: {}", file.display()))?;
            if !metadata.is_file() {
                anyhow::bail!("Not a regular file: {}", file.display());
            }

            // Validate block size range (1KB to 100MB)
            if block_size < 1024 || block_size > 104_857_600 {
                anyhow::bail!(
                    "Invalid block size: {} (must be between 1024 and 104857600)",
                    block_size
                );
            }

            let detection_config = cfg.to_detection_config();
            let store_dir = std::env::current_dir()
                .context("Cannot determine current directory for token store")?
                .join(".loktok");
            let store_result = store::Store::new(&store_dir);

            // If clipboard requested, capture output to string first
            if clipboard {
                // Tokenize to a temp buffer, then copy to clipboard + write to output
                let temp_output = tempfile::NamedTempFile::new()
                    .context("Cannot create temp file for clipboard capture")?;
                let temp_path = temp_output.path().to_path_buf();

                processor::process_file_with_config(
                    &file,
                    Some(&temp_path),
                    block_size,
                    cli.quiet,
                    dry_run,
                    &detection_config,
                    store_result.ok().as_ref(),
                    cfg.ttl_seconds(),
                )?;

                let tokenized = fs::read_to_string(&temp_path)
                    .context("Cannot read tokenized output for clipboard")?;

                // Copy to clipboard (gracefully handle failure)
                match clipboard::copy_to_clipboard(&tokenized) {
                    Ok(()) => eprintln!("logtok: tokenized output copied to clipboard"),
                    Err(e) => eprintln!("logtok: {}", e),
                }

                // Also write to output destination (file or stdout)
                if let Some(ref out_path) = output {
                    fs::write(out_path, &tokenized)
                        .with_context(|| format!("Cannot write to {}", out_path.display()))?;
                } else {
                    print!("{}", tokenized);
                }
            } else {
                processor::process_file_with_config(
                    &file,
                    output.as_deref(),
                    block_size,
                    cli.quiet,
                    dry_run,
                    &detection_config,
                    store_result.ok().as_ref(),
                    cfg.ttl_seconds(),
                )?;
            }
        }

        Commands::Detokenize {
            file,
            detailed,
            store: store_path,
        } => {
            // Determine store directory
            let store_dir = match store_path {
                Some(p) => p,
                None => std::env::current_dir()
                    .context("Cannot determine current directory for token store")?
                    .join(".loktok"),
            };

            // Load store -- LOGTOK_KEY required for detokenize
            let store = store::Store::new(&store_dir)
                .context("Cannot open token store. Is LOGTOK_KEY set?")?;
            let token_data = store
                .load()
                .context("Cannot load token store. Check LOGTOK_KEY or store file integrity.")?;

            if token_data.token_to_value.is_empty() {
                anyhow::bail!(
                    "Token store is empty. Run `logtok tokenize` first to create token mappings."
                );
            }

            // Read input (file or stdin)
            let input = detokenizer::read_input(file.as_deref())?;

            // De-tokenize
            let result = detokenizer::detokenize(&input, &token_data.token_to_value);

            // Print summary to stderr
            if !cli.quiet {
                eprintln!(
                    "logtok: {} tokens replaced, {} unresolved",
                    result.replaced_count, result.unresolved_count
                );
            }

            // Write output
            detokenizer::write_output(&result, detailed.as_deref())?;
        }

        Commands::ResetStore => {
            let store_dir = std::env::current_dir()
                .context("Cannot determine current directory for token store")?
                .join(".loktok");
            let store = store::Store::new(&store_dir)?;
            store.reset()?;
            eprintln!("logtok: token store reset");
        }
    }

    Ok(())
}
