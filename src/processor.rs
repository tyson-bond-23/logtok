use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::compactor::Compactor;
use crate::detector::DetectionPatterns;
use crate::json_processor::{is_json_line, tokenize_json_line};
use crate::tokenizer::TokenMap;

pub fn process_file(
    input_path: &Path,
    output_path: Option<&Path>,
    block_size: usize,
    quiet: bool,
) -> Result<()> {
    // Get file size for progress bar
    let metadata = fs::metadata(input_path)
        .with_context(|| format!("Cannot read file: {}", input_path.display()))?;
    let file_size = metadata.len();

    // Set up progress bar on stderr (D-16)
    let progress = if !quiet && file_size > 0 {
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

    // Set up output writer (D-15: stdout default, --output for file)
    let mut writer: Box<dyn Write> = match output_path {
        Some(path) => {
            let f = File::create(path)
                .with_context(|| format!("Cannot create output file: {}", path.display()))?;
            Box::new(BufWriter::new(f))
        }
        None => Box::new(BufWriter::new(std::io::stdout().lock())),
    };

    // Initialize shared state -- CRITICAL: these live outside the block loop
    // DetectionPatterns compiled once (avoid re-compiling regexes per block)
    // TokenMap shared across ALL blocks (TOK-01 determinism)
    let patterns = DetectionPatterns::new();
    let mut token_map = TokenMap::new();
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

    Ok(())
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
