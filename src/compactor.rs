/// Compacts consecutive identical lines into `[xN] line` format.
/// Optimized for LLM consumption (D-07): minimal tokens, full diagnostic value.
pub struct Compactor {
    last_line: Option<String>,
    count: usize,
}

impl Compactor {
    pub fn new() -> Self {
        Self {
            last_line: None,
            count: 0,
        }
    }

    /// Feed a tokenized line. Returns a completed compacted line if the streak breaks.
    pub fn feed(&mut self, line: String) -> Option<String> {
        match &self.last_line {
            Some(last) if *last == line => {
                self.count += 1;
                None
            }
            _ => {
                let output = self.flush();
                self.last_line = Some(line);
                self.count = 1;
                output
            }
        }
    }

    /// Flush the current streak. Must be called after all lines are fed.
    pub fn flush(&mut self) -> Option<String> {
        self.last_line.take().map(|line| {
            if self.count > 1 {
                format!("[x{}] {}", self.count, line)
            } else {
                line
            }
        })
    }
}

impl Default for Compactor {
    fn default() -> Self {
        Self::new()
    }
}
