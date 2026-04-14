use logtok::compactor::Compactor;

#[test]
fn compactor_three_identical_lines_outputs_x3() {
    let mut compactor = Compactor::new();
    let r1 = compactor.feed("same line".to_string());
    let r2 = compactor.feed("same line".to_string());
    let r3 = compactor.feed("same line".to_string());
    // First feed starts a new streak, no output
    assert_eq!(r1, None);
    // Subsequent identical feeds continue streak, no output
    assert_eq!(r2, None);
    assert_eq!(r3, None);
    // Flush emits the compacted line
    let flushed = compactor.flush();
    assert_eq!(flushed, Some("[x3] same line".to_string()));
}

#[test]
fn compactor_single_line_no_x1_prefix() {
    let mut compactor = Compactor::new();
    let r = compactor.feed("only line".to_string());
    assert_eq!(r, None);
    let flushed = compactor.flush();
    assert_eq!(flushed, Some("only line".to_string()));
}

#[test]
fn compactor_alternating_lines_output_individually() {
    let mut compactor = Compactor::new();
    let r1 = compactor.feed("line A".to_string());
    assert_eq!(r1, None); // First line, no output yet
    let r2 = compactor.feed("line B".to_string());
    assert_eq!(r2, Some("line A".to_string())); // Streak broke, emit A
    let r3 = compactor.feed("line C".to_string());
    assert_eq!(r3, Some("line B".to_string())); // Streak broke, emit B
    let flushed = compactor.flush();
    assert_eq!(flushed, Some("line C".to_string())); // Final flush emits C
}

#[test]
fn compactor_flush_emits_last_tracked_line() {
    let mut compactor = Compactor::new();
    compactor.feed("tracked".to_string());
    let flushed = compactor.flush();
    assert_eq!(flushed, Some("tracked".to_string()));
    // Second flush should return None (nothing tracked)
    let flushed2 = compactor.flush();
    assert_eq!(flushed2, None);
}

#[test]
fn compactor_mixed_runs() {
    // A, A, A, B, B, C -> [x3] A, [x2] B, C
    let mut compactor = Compactor::new();
    let mut output: Vec<String> = Vec::new();

    if let Some(line) = compactor.feed("A".to_string()) { output.push(line); }
    if let Some(line) = compactor.feed("A".to_string()) { output.push(line); }
    if let Some(line) = compactor.feed("A".to_string()) { output.push(line); }
    if let Some(line) = compactor.feed("B".to_string()) { output.push(line); }
    if let Some(line) = compactor.feed("B".to_string()) { output.push(line); }
    if let Some(line) = compactor.feed("C".to_string()) { output.push(line); }
    if let Some(line) = compactor.flush() { output.push(line); }

    assert_eq!(output, vec![
        "[x3] A".to_string(),
        "[x2] B".to_string(),
        "C".to_string(),
    ]);
}
