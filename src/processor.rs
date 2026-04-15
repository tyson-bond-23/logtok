use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::compactor::Compactor;
use crate::detector::{DetectionConfig, DetectionPatterns};
use crate::json_processor::{is_json_line, tokenize_json_line};
use crate::store::Store;
use crate::tokenizer::TokenMap;

/// Categories whose example values are never shown in dry-run output (T-02-12).
const HIDDEN_CATEGORIES: &[&str] = &["KEY", "PASS", "CONN", "JWT", "PEM"];

/// Backward-compatible wrapper: uses default config, no store, no dry-run.
pub fn process_file(
    input_path: &Path,
    output_path: Option<&Path>,
    block_size: usize,
    quiet: bool,
) -> Result<()> {
    let default_config = DetectionConfig::default();
    process_file_with_config(
        input_path,
        output_path,
        block_size,
        quiet,
        false,
        &default_config,
        None,
        0,
    )
}

/// Full processing pipeline with config-driven detection, store lifecycle, and dry-run support.
pub fn process_file_with_config(
    input_path: &Path,
    output_path: Option<&Path>,
    block_size: usize,
    quiet: bool,
    dry_run: bool,
    detection_config: &DetectionConfig,
    store: Option<&Store>,
    ttl_seconds: u64,
) -> Result<()> {
    // Get file size for progress bar
    let metadata = fs::metadata(input_path)
        .with_context(|| format!("Cannot read file: {}", input_path.display()))?;
    let file_size = metadata.len();

    // Set up progress bar on stderr (D-16) -- skip for dry-run
    let progress = if !quiet && !dry_run && file_size > 0 {
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Open input
    let file = File::open(input_path)
        .with_context(|| format!("Cannot open file: {}", input_path.display()))?;
    let reader = BufReader::new(file);

    // Initialize detection patterns from config (D-05, D-08)
    let patterns = DetectionPatterns::from_config(detection_config)
        .with_context(|| "Failed to compile detection patterns from config")?;

    // Initialize token map -- load from store if available
    let mut token_map = if let Some(s) = store {
        let stored_data = s.load().with_context(|| "Failed to load token store")?;
        let mut tm = TokenMap::from_data(stored_data);
        if ttl_seconds > 0 {
            tm.purge_expired(ttl_seconds);
        }
        tm
    } else {
        TokenMap::new()
    };

    // Dry-run mode: collect all detections without tokenizing
    if dry_run {
        return run_dry_run(input_path, reader, &patterns, block_size);
    }

    // Set up output writer (D-15: stdout default, --output for file)
    let mut writer: Box<dyn Write> = match output_path {
        Some(path) => {
            let f = File::create(path)
                .with_context(|| format!("Cannot create output file: {}", path.display()))?;
            Box::new(BufWriter::new(f))
        }
        None => Box::new(BufWriter::new(std::io::stdout().lock())),
    };

    let mut compactor = Compactor::new();

    // Detect format from first line
    let mut is_json_format: Option<bool> = None;

    // Block processing (D-09, D-10): accumulate lines up to block_size, process, repeat
    let mut block: Vec<String> = Vec::new();
    let mut block_bytes: usize = 0;

    for line_result in reader.lines() {
        let line = line_result.context("Error reading line from input file")?;
        let line_len = line.len() + 1; // +1 for newline

        // Auto-detect format on first non-empty line
        if is_json_format.is_none() && !line.trim().is_empty() {
            is_json_format = Some(is_json_line(&line));
        }

        block_bytes += line_len;
        block.push(line);

        // Process block when size threshold reached (D-10: default 64KB)
        if block_bytes >= block_size {
            process_block(
                &block,
                &mut token_map,
                &patterns,
                &mut compactor,
                &mut writer,
                is_json_format.unwrap_or(false),
            )?;
            if let Some(ref pb) = progress {
                pb.inc(block_bytes as u64);
            }
            block.clear();
            block_bytes = 0;
        }
    }

    // Process remaining lines in final partial block
    if !block.is_empty() {
        process_block(
            &block,
            &mut token_map,
            &patterns,
            &mut compactor,
            &mut writer,
            is_json_format.unwrap_or(false),
        )?;
        if let Some(ref pb) = progress {
            pb.inc(block_bytes as u64);
        }
    }

    // Flush compactor for the last tracked line
    if let Some(line) = compactor.flush() {
        writeln!(writer, "{}", line)?;
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("done");
    }

    writer.flush()?;

    // Print summary to stderr
    eprintln!("logtok: {} unique tokens generated", token_map.len());

    // Save token map back to store if available
    if let Some(s) = store {
        // Ensure .logtok/.gitignore exists (T-02-15)
        ensure_gitignore(input_path, s)?;
        s.save(token_map.to_data())
            .with_context(|| "Failed to save token store")?;
    }

    Ok(())
}

/// Ensure `.logtok/.gitignore` exists with `*` content to prevent accidental commit (T-02-15).
fn ensure_gitignore(_input_path: &Path, _store: &Store) -> Result<()> {
    let store_dir = std::env::current_dir()?.join(".logtok");
    let gitignore_path = store_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::create_dir_all(&store_dir)?;
        fs::write(&gitignore_path, "*\n")?;
    }
    Ok(())
}

/// Dry-run mode: scan all lines, collect detection matches, print summary table (D-16, T-02-12).
fn run_dry_run(
    input_path: &Path,
    reader: BufReader<File>,
    patterns: &DetectionPatterns,
    _block_size: usize,
) -> Result<()> {
    // Collect all detections: category -> (count, example_values)
    let mut category_stats: HashMap<String, (usize, Vec<String>)> = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.context("Error reading line from input file")?;
        let matches = patterns.detect(&line);
        for m in matches {
            let entry = category_stats
                .entry(m.category.clone())
                .or_insert_with(|| (0, Vec::new()));
            entry.0 += 1;
            // Collect up to 2 unique examples per category
            if entry.1.len() < 2 && !entry.1.contains(&m.value) {
                entry.1.push(m.value);
            }
        }
    }

    let total_count: usize = category_stats.values().map(|(c, _)| c).sum();
    let filename = input_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| input_path.display().to_string());

    eprintln!(
        "logtok dry-run: {} sensitive values detected in {}",
        total_count, filename
    );
    eprintln!();

    if total_count == 0 {
        eprintln!("  No sensitive values detected.");
        return Ok(());
    }

    // Sort categories by count descending
    let mut sorted: Vec<(String, usize, Vec<String>)> = category_stats
        .into_iter()
        .map(|(cat, (count, examples))| (cat, count, examples))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    // Print table header
    eprintln!("  {:<10} {:>5}  {}", "Category", "Count", "Examples");
    eprintln!("  {:<10} {:>5}  {}", "--------", "-----", "--------");

    for (category, count, examples) in &sorted {
        let examples_str = if HIDDEN_CATEGORIES.contains(&category.as_str()) {
            "(values hidden)".to_string()
        } else {
            examples
                .iter()
                .map(|e| truncate_example(e, 40))
                .collect::<Vec<_>>()
                .join(", ")
        };
        eprintln!("  {:<10} {:>5}  {}", category, count, examples_str);
    }

    eprintln!();
    eprintln!("No output written. Run without --dry-run to tokenize.");

    Ok(())
}

/// Truncate a string to max_len characters, appending "..." if truncated.
fn truncate_example(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

fn process_block(
    block: &[String],
    token_map: &mut TokenMap,
    patterns: &DetectionPatterns,
    compactor: &mut Compactor,
    writer: &mut dyn Write,
    is_json: bool,
) -> Result<()> {
    for line in block {
        let tokenized = if is_json && is_json_line(line) {
            match tokenize_json_line(line, token_map, patterns) {
                Ok(json_str) => json_str,
                Err(_) => {
                    // If JSON parse fails, fall back to plain text tokenization
                    token_map.tokenize_line(line, patterns)
                }
            }
        } else {
            token_map.tokenize_line(line, patterns)
        };

        // Feed to compactor; write any completed compacted line
        if let Some(compacted_line) = compactor.feed(tokenized) {
            writeln!(writer, "{}", compacted_line)?;
        }
    }
    Ok(())
}
