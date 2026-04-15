use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::detector::{CustomPatternDef, DetectionConfig};

/// Top-level config from .logtok.toml
#[derive(Deserialize, Default, Debug)]
pub struct LoktokConfig {
    #[serde(default)]
    pub detection: DetectionSection,
    #[serde(default)]
    pub store: StoreSection,
}

#[derive(Deserialize, Default, Debug)]
pub struct DetectionSection {
    /// Categories to disable (all enabled by default)
    #[serde(default)]
    pub disabled: Vec<String>,
    /// Custom detection patterns
    #[serde(default)]
    pub custom_patterns: Vec<CustomPatternToml>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomPatternToml {
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub capture_group: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct StoreSection {
    /// TTL for token mappings in days (default 30)
    #[serde(default = "default_ttl_days")]
    pub ttl_days: u32,
}

impl Default for StoreSection {
    fn default() -> Self {
        Self {
            ttl_days: default_ttl_days(),
        }
    }
}

fn default_ttl_days() -> u32 {
    30
}

/// Walk up from `start_dir` looking for `.logtok.toml`.
/// Returns `Some(path)` if found, `None` if no ancestor contains it.
pub fn find_config_from(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(".logtok.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Walk up from CWD looking for `.logtok.toml`.
pub fn find_config() -> Option<PathBuf> {
    let dir = std::env::current_dir().ok()?;
    find_config_from(&dir)
}

/// Load and parse a `.logtok.toml` config file.
pub fn load_config(path: &Path) -> Result<LoktokConfig, crate::error::TokeniserError> {
    let content =
        std::fs::read_to_string(path).map_err(|e| crate::error::TokeniserError::ConfigError {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
    toml::from_str(&content).map_err(|e| crate::error::TokeniserError::ConfigError {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

impl LoktokConfig {
    /// Convert to DetectionConfig for use with DetectionPatterns::from_config.
    pub fn to_detection_config(&self) -> DetectionConfig {
        DetectionConfig {
            disabled_categories: self.detection.disabled.clone(),
            custom_patterns: self
                .detection
                .custom_patterns
                .iter()
                .map(|p| CustomPatternDef {
                    name: p.name.clone(),
                    pattern: p.pattern.clone(),
                    capture_group: p.capture_group,
                })
                .collect(),
        }
    }

    /// Get TTL in seconds (ttl_days * 86400).
    pub fn ttl_seconds(&self) -> u64 {
        self.store.ttl_days as u64 * 86400
    }
}
