use clap::Parser;
use std::path::PathBuf;

/// Tokenize sensitive data out of log files for safe AI analysis
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about)]
pub struct Cli {
    /// Path to the log file to tokenize
    pub file: Option<PathBuf>,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Block size in bytes for processing
    #[arg(long, default_value = "65536")]
    pub block_size: usize,

    /// Suppress progress bar output
    #[arg(short, long)]
    pub quiet: bool,

    /// Preview what would be tokenized without writing output (DET-04)
    #[arg(long)]
    pub dry_run: bool,

    /// Reset (delete) the encrypted token store and exit
    #[arg(long)]
    pub reset_store: bool,

    /// Path to .logtok.toml config file (overrides discovery)
    #[arg(long)]
    pub config: Option<PathBuf>,
}
