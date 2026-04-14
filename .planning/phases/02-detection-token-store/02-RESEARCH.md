# Phase 2: Detection & Token Store - Research

**Researched:** 2026-04-14
**Domain:** Regex pattern detection, TOML configuration, AES-256-GCM encrypted persistence
**Confidence:** HIGH

## Summary

Phase 2 expands the existing 7-category regex detector to 19 categories, adds per-project TOML configuration (`.logtok.toml`), implements dry-run preview mode, and persists token mappings in an AES-256-GCM encrypted store. The existing codebase provides a clean foundation: `DetectionPatterns` already supports priority-ordered overlap resolution and `TokenMap` already has deterministic `get_or_insert()` semantics. The main work is (1) making the pattern list configurable rather than hardcoded, (2) adding serde serialization to `TokenMap`, (3) building an encryption layer with Argon2 key derivation from `LOGTOK_KEY` env var, and (4) TOML config discovery/parsing.

All required crates (`aes-gcm`, `argon2`, `toml`, `rand`) are mature, well-documented, and pure Rust -- no cross-compilation friction. The encryption pipeline is straightforward: Argon2id derives a 32-byte key from the env var passphrase + random salt, AES-256-GCM encrypts the serde-serialized token map, nonce + salt + ciphertext are stored as a single blob.

**Primary recommendation:** Refactor `DetectionPatterns::new()` to accept a config struct that merges built-in defaults with TOML overrides, add `serde::{Serialize, Deserialize}` to `TokenMap` internals, and build a thin `Store` module that handles encrypt/decrypt/load/save of the token map.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Comprehensive regex-based detection -- all categories active by default, no opt-in required
- **D-02:** New categories beyond Phase 1's 7 (EMAIL, URL, KEY, PASS, IP, HOST, PATH): CONN, PHONE, UUID, ARN, SSN, CC, JWT, PEM, MAC, DNS, OS, NAME
- **D-03:** Name/username detection is structured-only -- `user=X`, `"name": "X"`, `author: X`, `"username": "jdoe"` patterns. No free-text NLP name detection in v1.
- **D-04:** Existing Phase 1 HOST pattern (`.internal/.local/.corp`) remains as-is; DNS category covers broader domain matching
- **D-05:** Per-project config file: `.logtok.toml` in project root
- **D-06:** TOML format -- human-editable, language-neutral, Rust-native (`toml` crate)
- **D-07:** Config discovery: `logtok` walks up from CWD to find `.logtok.toml`
- **D-08:** All detection categories enabled by default -- zero-config for the common case
- **D-09:** No global/user-level config for v1 -- per-project only
- **D-10:** Store location: `.logtok/store.enc` in the project directory (next to `.logtok.toml`)
- **D-11:** Encryption: AES-256-GCM with key derived via Argon2 from `LOGTOK_KEY` environment variable
- **D-12:** No interactive passphrase prompt -- env var only. CLI errors clearly if `LOGTOK_KEY` is not set when store operations are needed.
- **D-13:** Append-only store with optional TTL expiry
- **D-14:** Manual reset via `--reset-store` flag to wipe and start fresh
- **D-15:** Store format: serialized via serde, encrypted as single blob
- **D-16:** Claude's discretion -- dry-run output format, detail level, and presentation

### Claude's Discretion
- Dry-run output format and detail level
- Regex pattern specifics and ordering/priority for new categories
- Error handling for malformed config files
- Store migration strategy if format changes
- TTL default value and granularity

### Deferred Ideas (OUT OF SCOPE)
- LLM-based contextual name/PII detection -- deferred to v2
- Subcommand structure (`logtok tokenize`, `logtok diagnose`) -- Phase 3
- Global/user-level config (`~/.config/logtok/`) -- v2

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DET-01 | Detect and tokenize credentials (API keys, tokens, passwords, connection strings) | Existing KEY/PASS patterns + new CONN, JWT, PEM categories cover this. Capture-group approach for extracting secret values. |
| DET-02 | Detect and tokenize PII (emails, IP addresses, usernames, phone numbers) | Existing EMAIL/IP + new PHONE, SSN, CC, NAME, UUID categories. Luhn validation for CC. Structured-only NAME detection. |
| DET-03 | Detect and tokenize infrastructure details (hostnames, file paths, internal URLs, ports) | Existing URL/HOST/PATH + new DNS, MAC, ARN, OS categories. HOST preserved as-is per D-04. |
| DET-04 | Preview what would be tokenized before sending (dry-run mode) | New `--dry-run` CLI flag. Tabular output to stderr showing category, value, proposed token. |
| DET-05 | Configure detection rules (enable/disable categories, add custom patterns) | `.logtok.toml` config with `[detection]` section. Category enable/disable + `[[custom_patterns]]` array. |
| TOK-03 | Token mappings stored in encrypted local store (AES-256-GCM) | aes-gcm 0.10.3 + argon2 0.5.3 for key derivation from `LOGTOK_KEY` env var. Store at `.logtok/store.enc`. |
| TOK-04 | Token store persists across sessions and can be reused | Serde serialization of `TokenMap` internals. Load on startup, save after processing. TTL-based expiry for old entries. |

</phase_requirements>

## Standard Stack

### New Dependencies for Phase 2

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| aes-gcm | 0.10.3 | AES-256-GCM authenticated encryption | RustCrypto AEAD, NCC Group audited, pure Rust, HW-accelerated via AES-NI. [VERIFIED: docs.rs/crate/aes-gcm/latest shows 0.10.3] |
| argon2 | 0.5.3 | Key derivation from LOGTOK_KEY env var | Argon2id winner of Password Hashing Competition, memory-hard, pure Rust. [VERIFIED: docs.rs/crate/argon2/latest shows 0.5.3] |
| toml | 0.8 | TOML config file parsing | De facto Rust TOML crate, serde integration. Pin to 0.8.x for edition 2021 compat; 1.1.x exists but 0.8 is widely used and stable. [VERIFIED: docs.rs shows 1.1.2 latest, but 0.8.x is the conservative choice matching Cargo.toml edition 2024] |
| rand | 0.9 | Cryptographic nonce/salt generation | OS entropy source, already in CLAUDE.md stack. [VERIFIED: docs.rs/crate/rand/latest shows 0.10.1 available; 0.9+ per CLAUDE.md spec] |

**Note on toml version:** The `toml` crate 0.8.x is mature and well-tested. Version 1.1.x adds TOML spec 1.1.0 support but the project only needs basic table/array deserialization. Either works; recommend `toml = "0.8"` for stability or `toml = "1"` for latest. [ASSUMED -- no specific compatibility issue identified, either version works]

### Already in Cargo.toml (no changes needed)
| Library | Version | Used For in Phase 2 |
|---------|---------|---------------------|
| serde | 1.0.228 | Serialize/Deserialize derives on TokenMap, Config structs |
| serde_json | 1.0.149 | JSON log processing (unchanged from Phase 1) |
| clap | 4.6.0 | New CLI flags: `--dry-run`, `--reset-store`, `--config` |
| thiserror | 2 | New error variants for config/store/encryption errors |
| anyhow | 1.0.102 | Application-level error handling |
| regex | 1.12.3 | All pattern matching (unchanged engine) |

### Installation

```bash
cargo add aes-gcm@0.10.3 --features aes
cargo add argon2@0.5.3
cargo add toml@0.8
cargo add rand@0.9
```

## Architecture Patterns

### Recommended Project Structure

```
src/
  cli.rs           # Extended with --dry-run, --reset-store, --config flags
  config.rs        # NEW: .logtok.toml parsing, config discovery, category definitions
  detector.rs      # REFACTORED: Accept config-driven pattern list instead of hardcoded
  store.rs         # NEW: Encrypted token store (load/save/encrypt/decrypt)
  tokenizer.rs     # EXTENDED: Add Serialize/Deserialize, TTL metadata
  processor.rs     # MODIFIED: Integrate store load/save, dry-run branch
  error.rs         # EXTENDED: Config, store, and encryption error variants
  main.rs          # MODIFIED: Config loading, store lifecycle
  json_processor.rs  # Unchanged
  compactor.rs       # Unchanged
```

### Pattern 1: Config-Driven Detection

**What:** Replace hardcoded patterns in `DetectionPatterns::new()` with a config struct that merges built-in defaults with user overrides.

**When to use:** This is the core refactor for DET-05.

```rust
// Source: Design based on existing DetectionPatterns architecture
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct DetectionConfig {
    /// Categories to disable (all enabled by default per D-08)
    #[serde(default)]
    pub disabled_categories: Vec<String>,
    /// Custom patterns to add
    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

#[derive(Deserialize, Clone)]
pub struct CustomPattern {
    pub name: String,
    pub pattern: String,
    /// Optional: capture group index (like KEY/PASS use group 1)
    pub capture_group: Option<usize>,
}

impl DetectionPatterns {
    pub fn from_config(config: &DetectionConfig) -> Self {
        let mut patterns = Self::builtin_patterns();
        // Remove disabled categories
        patterns.retain(|(cat, _)| !config.disabled_categories.contains(cat));
        // Append custom patterns
        for custom in &config.custom_patterns {
            let regex = Regex::new(&custom.pattern)
                .expect("Invalid custom regex"); // or return Result
            patterns.push((custom.name.clone(), regex));
        }
        Self { patterns }
    }
}
```

### Pattern 2: Encrypted Store Lifecycle

**What:** Load encrypted store at startup, merge with new tokens during processing, save back on completion.

**When to use:** Every CLI invocation that performs tokenization.

```rust
// Source: [CITED: docs.rs/aes-gcm] + [CITED: docs.rs/argon2]
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;

pub struct Store {
    cipher: Aes256Gcm,
    path: PathBuf,
}

impl Store {
    pub fn new(passphrase: &str, store_path: PathBuf) -> Result<Self> {
        // Generate or load salt (stored as first 16 bytes of file)
        let salt = /* load from file header or generate new */;
        
        // Derive 32-byte key via Argon2id
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(passphrase.as_bytes(), &salt, &mut key)?;
        
        let cipher = Aes256Gcm::new(&key.into());
        Ok(Self { cipher, path: store_path })
    }
    
    pub fn load(&self) -> Result<TokenMap> {
        // Read file: [16-byte salt][12-byte nonce][ciphertext]
        // Decrypt ciphertext, deserialize via serde_json/bincode
    }
    
    pub fn save(&self, token_map: &TokenMap) -> Result<()> {
        // Serialize token_map, generate fresh nonce, encrypt, write
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let serialized = serde_json::to_vec(&token_map)?;
        let ciphertext = self.cipher.encrypt(&nonce, serialized.as_ref())?;
        // Write: [salt][nonce][ciphertext]
    }
}
```

### Pattern 3: Config Discovery (Walk-Up)

**What:** Walk from CWD upward to find `.logtok.toml`, similar to how `.gitignore` or `Cargo.toml` discovery works.

```rust
// Source: Standard Rust pattern for config discovery
pub fn find_config() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
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
```

### Pattern 4: Dry-Run Output

**What:** When `--dry-run` is set, detect and display matches without writing tokenized output.

**Recommendation for Claude's Discretion (D-16):**

```
$ logtok --dry-run server.log

logtok dry-run: 47 sensitive values detected in server.log

  Category  Count  Examples
  --------  -----  --------
  IP        12     192.168.1.100, 10.0.0.55
  EMAIL     5      admin@example.com, user@company.org
  KEY       8      (values hidden)
  HOST      4      db-primary.internal
  CONN      3      postgresql://... (truncated)
  PATH      7      /var/log/app/server.log
  JWT       2      eyJhbGciOiJSUzI1... (truncated)
  ...

No output file written. Run without --dry-run to tokenize.
```

Output to stderr so it does not interfere with piping. Show category counts and truncated examples (never show full KEY/PASS values even in dry-run). [ASSUMED -- this format is a recommendation, not a locked decision]

### Anti-Patterns to Avoid

- **Re-compiling regexes per line:** The existing pattern of compiling once in `DetectionPatterns::new()` must be preserved. Custom patterns from TOML should also be compiled once at startup.
- **Storing the encryption key in the store file:** The key is derived from `LOGTOK_KEY` env var each session. Only the salt and nonce are stored alongside ciphertext.
- **Serializing `Regex` objects:** `regex::Regex` does not implement Serialize. The config stores pattern strings; compiled `Regex` objects are runtime-only.
- **Encrypting per-entry:** The store is a single encrypted blob (per D-15), not per-entry encryption. This is simpler and avoids nonce reuse risks.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Authenticated encryption | Custom AES wrapper | `aes-gcm` crate | Authenticated encryption is notoriously hard to get right. aes-gcm is NCC Group audited. |
| Key derivation | SHA-256 of passphrase | `argon2` crate (Argon2id) | Simple hashing is vulnerable to brute force. Argon2id is memory-hard, GPU-resistant. |
| TOML parsing | Manual string parsing | `toml` crate with serde | Edge cases in TOML spec (multiline strings, inline tables) are non-trivial. |
| Luhn checksum | Manual digit loop | Inline ~15 lines or `luhn3` crate | Algorithm is simple but easy to get off-by-one errors. A small inline implementation is acceptable here. |
| Nonce generation | Timestamp-based | `Aes256Gcm::generate_nonce(&mut OsRng)` | OS entropy is the only safe nonce source. Timestamp-based nonces can collide. |
| Credit card regex | Single pattern | Per-issuer patterns (Visa, MC, Amex) + Luhn | Card formats vary by issuer. Pattern-only matching produces too many false positives without Luhn validation. |

## Common Pitfalls

### Pitfall 1: Nonce Reuse in AES-GCM
**What goes wrong:** Using the same nonce with the same key completely breaks AES-GCM security -- an attacker can recover plaintext.
**Why it happens:** Developer stores a fixed nonce or derives it deterministically.
**How to avoid:** Generate a fresh random nonce via `Aes256Gcm::generate_nonce(&mut OsRng)` on every `save()` call. Store the nonce alongside the ciphertext.
**Warning signs:** Any code path where the nonce is not freshly random.

### Pitfall 2: Salt Handling for Argon2
**What goes wrong:** If the salt changes between sessions, the derived key changes, and the store becomes undecryptable.
**Why it happens:** Generating a new salt on every startup instead of persisting it.
**How to avoid:** Generate the salt once when the store is first created. Store it as the unencrypted header of `store.enc`. Load it on subsequent runs.
**Warning signs:** "decryption failed" errors on second run.

### Pitfall 3: Regex Priority with New Categories
**What goes wrong:** New patterns like CONN (`postgresql://...`) overlap with URL (`https?://...`). JWT (`eyJ...`) could match KEY patterns. DNS overlaps with HOST.
**Why it happens:** Pattern order determines priority for overlapping matches (existing architecture).
**How to avoid:** Carefully order patterns: most specific first. Suggested priority order:
1. PEM (multi-line, very specific structure)
2. JWT (eyJ... three-part, very specific)
3. CONN (protocol-prefixed: `postgresql://`, `redis://`, `kafka://`)
4. EMAIL (before IP -- emails contain dot-separated segments)
5. URL (https?://)
6. KEY (key=value patterns with capture groups)
7. PASS (password=value patterns with capture groups)
8. CC (digit sequences, Luhn-validated)
9. SSN (3-2-4 digit pattern)
10. ARN (arn:aws: prefix, very specific)
11. NAME (structured: user=, "name":, etc.)
12. UUID (8-4-4-4-12 hex pattern)
13. MAC (xx:xx:xx:xx:xx:xx)
14. PHONE (international phone formats)
15. IP (dotted quad)
16. HOST (.internal/.local/.corp)
17. DNS (broader domain matching)
18. OS (version strings)
19. PATH (slash-separated, catches a lot -- must be last)
**Warning signs:** Test cases where a connection string is tokenized as URL instead of CONN.

### Pitfall 4: False Positives in Broad Categories
**What goes wrong:** DNS catches every domain in log text. PATH matches things like `/api/v1/users`. OS matches random version-like strings.
**Why it happens:** Broad regex without context constraints.
**How to avoid:** 
- DNS: Require at least 2 dot-separated segments with a valid TLD suffix, or limit to known patterns
- PATH: Already requires 3+ segments which is reasonable
- OS: Anchor to known OS name prefixes (`Linux`, `Windows NT`, `Darwin`, `Ubuntu`)
- NAME: Strictly structured-only (D-03) -- only match `user=X`, `"name": "X"` etc.
**Warning signs:** Token counts spike unreasonably on real logs.

### Pitfall 5: Store File Corruption
**What goes wrong:** Process killed mid-write leaves corrupted `store.enc`. Next run fails to decrypt.
**Why it happens:** Writing directly to the store file without atomic guarantees.
**How to avoid:** Write to a temporary file (`.logtok/store.enc.tmp`), then atomically rename. On Unix this is `fs::rename()`; on Windows it requires `ReplaceFile` or similar.
**Warning signs:** Occasional "decryption failed" errors in CI or after crashes.

### Pitfall 6: LOGTOK_KEY Not Set
**What goes wrong:** User runs `logtok` without setting `LOGTOK_KEY`, gets a cryptic error.
**Why it happens:** Env var dependency is easy to forget.
**How to avoid:** Check for `LOGTOK_KEY` early in main, provide a clear error message: `"LOGTOK_KEY environment variable is required for token store encryption. Set it with: export LOGTOK_KEY='your-passphrase'"`. Per D-12, no interactive prompt.
**Warning signs:** Any code path that reaches encryption without validating the env var first.

## Code Examples

### Encryption Round-Trip (Verified Pattern)

```rust
// Source: [CITED: docs.rs/aes-gcm] + [CITED: docs.rs/argon2]
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;

fn derive_key(passphrase: &[u8], salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase, salt, &mut key)
        .expect("Argon2 key derivation failed");
    key
}

fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("encryption failed");
    let nonce_bytes: [u8; 12] = nonce.into();
    (ciphertext, nonce_bytes)
}

fn decrypt(key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .expect("decryption failed")
}
```

### TOML Config Structure

```toml
# .logtok.toml -- Example configuration

[detection]
# Disable specific built-in categories
disabled = ["OS", "DNS"]

[store]
# TTL for token mappings (default: 30 days)
ttl_days = 30

# Custom detection patterns
[[detection.custom_patterns]]
name = "INTERNAL_ID"
pattern = 'ACME-[A-Z]{2}-\d{6}'

[[detection.custom_patterns]]
name = "CUSTOM_TOKEN"
pattern = 'tok_[a-zA-Z0-9]{32}'
capture_group = 0
```

```rust
// Source: [CITED: docs.rs/toml/latest/toml/]
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct LoktokConfig {
    #[serde(default)]
    pub detection: DetectionSection,
    #[serde(default)]
    pub store: StoreSection,
}

#[derive(Deserialize, Default)]
pub struct DetectionSection {
    #[serde(default)]
    pub disabled: Vec<String>,
    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

#[derive(Deserialize, Default)]
pub struct StoreSection {
    #[serde(default = "default_ttl")]
    pub ttl_days: u32,
}

fn default_ttl() -> u32 { 30 }

pub fn load_config(path: &std::path::Path) -> anyhow::Result<LoktokConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: LoktokConfig = toml::from_str(&content)?;
    Ok(config)
}
```

### Serializable TokenMap Extension

```rust
// Source: Extending existing src/tokenizer.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct TokenMapData {
    pub value_to_token: HashMap<String, String>,
    pub token_to_value: HashMap<String, String>,  // Reverse map for Phase 3 de-tokenization
    pub category_counters: HashMap<String, u32>,
    pub entries: HashMap<String, TokenEntry>,  // For TTL tracking
}

#[derive(Serialize, Deserialize)]
pub struct TokenEntry {
    pub token: String,
    pub category: String,
    pub created_at: u64,  // Unix timestamp
}
```

### JWT Detection Pattern

```rust
// Source: [CITED: regex101.com/library patterns, regextester.com/105777]
// JWT: three base64url-encoded segments separated by dots, first starts with eyJ
("JWT", r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+")
```

### Connection String Patterns

```rust
// Source: [ASSUMED -- based on standard connection string formats]
// CONN: Database and message broker connection strings
("CONN", r"(?:postgresql|postgres|mysql|mongodb|mongodb\+srv|redis|rediss|amqp|amqps|kafka)://[^\s\"'>]+")
```

### Credit Card with Luhn Validation

```rust
// Source: [ASSUMED -- standard Luhn algorithm]
fn luhn_check(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap())
        .collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let sum: u32 = digits.iter().rev().enumerate().map(|(i, &d)| {
        if i % 2 == 1 {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else {
            d
        }
    }).sum();
    sum % 10 == 0
}
```

The CC detector should first regex-match potential card number patterns (4xxx, 5[1-5]xx, 3[47]xx, 6011, etc.) then validate with Luhn before creating a detection match.

## New Regex Patterns Reference

All new category patterns for Phase 2 (D-02):

| Category | Pattern (simplified) | Notes |
|----------|---------------------|-------|
| CONN | `(?:postgresql\|postgres\|mysql\|mongodb\|...)://[^\s"'>]+` | Protocol-prefixed connection strings |
| PHONE | `(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}` | US/international; risk of false positives with random digit sequences |
| UUID | `[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}` | Standard UUID format |
| ARN | `arn:aws:[a-zA-Z0-9-]+:[a-z0-9-]*:\d{12}:[^\s"]+` | AWS ARNs; extend for GCP/Azure |
| SSN | `\b\d{3}-\d{2}-\d{4}\b` | US SSN format; high false-positive risk -- may need context |
| CC | `\b(?:4\d{3}\|5[1-5]\d{2}\|3[47]\d{2}\|6011)[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b` | + Luhn validation post-match |
| JWT | `eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` | Three base64url segments |
| PEM | `-----BEGIN [A-Z ]*PRIVATE KEY-----` | Match just the header line; full block is multi-line |
| MAC | `(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}` | Standard colon-separated MAC |
| DNS | `\b[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+\.[a-zA-Z]{2,}\b` | Broad FQDN matching; high volume |
| OS | `(?:Linux\|Darwin\|Windows NT\|Ubuntu\|CentOS\|Red Hat\|Debian\|Alpine)\s+[\d.]+` | OS name + version |
| NAME | `(?i)(?:user(?:name)?\|author\|name)\s*[=:]\s*['"]?([a-zA-Z][a-zA-Z0-9._-]{1,30})['"]?` | Structured only (D-03), capture group |

## Store File Format

```
Byte layout of .logtok/store.enc:

[4 bytes]   Magic: "LTOK" (file identification)
[1 byte]    Version: 0x01
[16 bytes]  Salt (for Argon2 key derivation)
[12 bytes]  Nonce (for AES-GCM)
[N bytes]   Ciphertext (AES-GCM encrypted serde_json blob)
[16 bytes]  Auth tag (appended by AES-GCM, part of ciphertext output)
```

The magic bytes and version enable future format migration (Claude's discretion area). [ASSUMED -- specific byte layout is a recommendation]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `aes-gcm` used `GenericArray` | `aes-gcm` 0.10 uses `Array` type alias | 0.10.x | Import path changed; old examples with `GenericArray` are outdated |
| `rand` 0.8 `thread_rng()` | `rand` 0.9+ uses `OsRng` directly via `aead` re-export | 0.9 | Use `OsRng` from `aes_gcm::aead::OsRng`, not `rand::thread_rng()` |
| `argon2` had separate `Config` struct | `argon2` 0.5 uses builder pattern / `Argon2::default()` | 0.5.x | Simpler API, `hash_password_into()` for raw key derivation |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `toml` 0.8 vs 1.x -- either works, recommended 0.8 for stability | Standard Stack | LOW -- both versions work fine, just a version pin choice |
| A2 | Dry-run output format (tabular with truncated examples) | Architecture Patterns | LOW -- this is explicitly Claude's discretion per D-16 |
| A3 | Store file byte layout (magic + version + salt + nonce + ciphertext) | Store File Format | LOW -- internal format, can be changed freely as long as encrypt/decrypt round-trips |
| A4 | TTL default of 30 days | Code Examples | LOW -- configurable via TOML, just needs a reasonable default |
| A5 | Connection string regex covers the major protocols | New Regex Patterns | MEDIUM -- may miss niche protocols; custom_patterns covers the gap |
| A6 | Phone number regex may produce false positives on random digit sequences | New Regex Patterns | MEDIUM -- could flag non-phone numbers; context-aware validation would help but is out of scope for v1 |
| A7 | Atomic file write on Windows via rename may need special handling | Pitfall 5 | MEDIUM -- `fs::rename` on Windows may fail if target exists; may need `std::fs::rename` with temp file approach or platform-specific code |

## Open Questions

1. **PEM block detection scope**
   - What we know: PEM private key blocks are multi-line (`-----BEGIN ... PRIVATE KEY-----` through `-----END`).
   - What's unclear: Should we tokenize just the header line, or attempt multi-line detection?
   - Recommendation: Tokenize the header line only for v1. Multi-line detection (ADV-01) is deferred to v2. The header line alone is a strong indicator of a private key in logs.

2. **DNS false positive rate**
   - What we know: Broad FQDN matching will catch many domain names in logs.
   - What's unclear: Whether this creates too much noise on real-world logs.
   - Recommendation: Ship with DNS enabled by default but make it easy to disable via `.logtok.toml`. Monitor feedback.

3. **Token counter continuity across sessions**
   - What we know: TokenMap uses per-category counters (IP_001, IP_002...). When loading from store, counters must continue from where they left off.
   - What's unclear: Whether loading a store with IP_001-IP_050 and then encountering a new IP should produce IP_051.
   - Recommendation: Yes -- serialize `category_counters` alongside the value map. This ensures monotonically increasing counters across sessions.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Compilation | Yes | 1.94.1 (stable) | -- |
| cargo | Build/deps | Yes | 1.94.1 | -- |

Step 2.6: No external runtime dependencies beyond the Rust toolchain. All crates are pure Rust with no system library requirements. AES-GCM uses hardware acceleration (AES-NI) when available but falls back to software implementation.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A -- no user authentication, env var is an encryption passphrase not auth |
| V3 Session Management | No | N/A -- CLI tool, no sessions |
| V4 Access Control | No | N/A -- single-user CLI |
| V5 Input Validation | Yes | Validate TOML config structure via serde deserialization; validate regex patterns compile; validate `LOGTOK_KEY` is set |
| V6 Cryptography | Yes | AES-256-GCM (audited crate), Argon2id key derivation, random nonce per encryption, never store key |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Weak passphrase for LOGTOK_KEY | Information Disclosure | Argon2id with memory-hard parameters makes brute force expensive. Warn if passphrase < 8 chars. |
| Nonce reuse in AES-GCM | Information Disclosure | Fresh random nonce per save via OsRng. Never derive nonce deterministically. |
| Store file accessible to other users | Information Disclosure | File created with restrictive permissions (0600 on Unix). On Windows, inherit directory ACL. |
| Token values leaked in dry-run output | Information Disclosure | Truncate sensitive values in dry-run. Never show full KEY/PASS values. |
| Malicious TOML config with ReDoS pattern | Denial of Service | Rust `regex` crate guarantees O(m*n) -- no catastrophic backtracking possible. Safe by default. |

## Sources

### Primary (HIGH confidence)
- [docs.rs/aes-gcm](https://docs.rs/aes-gcm) -- Encrypt/decrypt API, nonce handling, `OsRng` usage
- [docs.rs/argon2](https://docs.rs/argon2) -- `hash_password_into()` for raw key derivation, salt requirements
- [docs.rs/toml](https://docs.rs/toml/latest/toml/) -- Serde deserialization from TOML strings
- Existing codebase: `src/detector.rs`, `src/tokenizer.rs`, `src/cli.rs` -- current architecture and patterns

### Secondary (MEDIUM confidence)
- [Rust-Crypt GitHub](https://github.com/karthik558/Rust-Crypt) -- Real-world AES-256-GCM + Argon2 implementation reference
- [RustCrypto Book](https://rustcrypto.org/key-derivation/hashing-password.html) -- Key derivation workflow documentation
- [JWT regex patterns](https://www.regextester.com/105777) -- JWT token detection regex

### Tertiary (LOW confidence)
- Phone number and SSN regex patterns -- commonly cited but false-positive rates vary by log format

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates verified on docs.rs, versions confirmed, pure Rust
- Architecture: HIGH -- extends existing well-structured codebase with clear patterns
- Pitfalls: HIGH -- crypto pitfalls (nonce reuse, salt handling) are well-documented in security literature
- Regex patterns: MEDIUM -- patterns are standard but false-positive rates depend on real log content

**Research date:** 2026-04-14
**Valid until:** 2026-05-14 (stable domain, slow-moving crate versions)
