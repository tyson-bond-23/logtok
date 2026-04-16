use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Tokenize sensitive data out of log files for safe AI analysis.
///
/// logtok removes credentials, PII, and infrastructure details from logs,
/// replacing them with deterministic tokens like [IP_001] and [KEY_002].
/// After AI diagnosis, de-tokenize the response to restore real values.
///
/// Examples:
///   logtok tokenize server.log              # tokenize to stdout
///   logtok tokenize server.log -o safe.log  # tokenize to file
///   logtok detokenize response.txt          # de-tokenize file
///   echo "..." | logtok detokenize          # de-tokenize from stdin
#[derive(Parser, Debug)]
#[command(name = "logtok", version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress progress bar and non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Path to .logtok.toml config file (overrides discovery)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Tokenize sensitive data in log files
    ///
    /// Reads a log file, detects sensitive values (API keys, IPs, emails, etc.),
    /// and replaces them with deterministic tokens like [IP_001], [KEY_002].
    Tokenize {
        /// Path to the log file to tokenize
        file: PathBuf,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Copy tokenized output to clipboard
        #[arg(long)]
        clipboard: bool,

        /// Block size in bytes for processing
        #[arg(long, default_value = "65536")]
        block_size: usize,

        /// Preview what would be tokenized without writing output
        #[arg(long)]
        dry_run: bool,
    },

    /// De-tokenize text, replacing tokens with real values
    ///
    /// Reads tokenized text (from Claude Code or any LLM response) and replaces
    /// [CATEGORY_NNN] tokens with the original sensitive values from the encrypted store.
    Detokenize {
        /// File containing tokenized text (reads stdin if omitted)
        file: Option<PathBuf>,

        /// Write detailed markdown report to file instead of stdout
        #[arg(long)]
        detailed: Option<PathBuf>,

        /// Path to .loktok directory containing store.enc (default: CWD/.loktok)
        #[arg(long)]
        store: Option<PathBuf>,
    },

    /// Delete the encrypted token store
    ResetStore,
}
