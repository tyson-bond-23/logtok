use clap::builder::styling::{AnsiColor, Color, Style, Styles};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const STYLES: Styles = Styles::styled()
    .header(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))).bold())
    .usage(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))).bold())
    .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))).bold())
    .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .valid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
    .invalid(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
    .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).bold());

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
#[command(name = "logtok", version, about, long_about, styles = STYLES)]
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
    #[command(after_long_help = "\
Examples:
  logtok tokenize server.log              Tokenize to stdout
  logtok tokenize server.log -o safe.log  Tokenize to file
  logtok tokenize app.log --dry-run       Preview what would change
  logtok tokenize app.log --clipboard     Tokenize and copy to clipboard")]
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
    #[command(after_long_help = "\
Examples:
  logtok detokenize response.txt                   De-tokenize a file
  echo \"...\" | logtok detokenize                    De-tokenize from stdin
  logtok detokenize response.txt --detailed report  Write detailed markdown report
  logtok detokenize response.txt --store /path      Use specific token store")]
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
    ///
    /// Removes the .logtok/store.enc file containing all token-to-value mappings.
    /// Use this when you want to start fresh or when changing the encryption key.
    /// This action is irreversible -- all stored token mappings will be lost.
    #[command(after_long_help = "\
Examples:
  logtok reset-store    Delete the token store in the current directory")]
    ResetStore,
}
