# logtok

Tokenize sensitive data out of application logs so they can be safely analyzed by AI for error diagnosis. Claude's results are then de-tokenized back into meaningful, readable output -- without secrets ever leaving your machine.

## Getting Started

```bash
git clone https://github.com/your-org/logtok.git
cd logtok
cargo install --path .

# Verify installation
logtok --help
```

Or use the install script:

```bash
# Unix/macOS
./install.sh

# Windows PowerShell
.\install.ps1
```

After installation, `logtok` is available globally — no compile wait on each run.

## How It Works

logtok uses a 3-part workflow that keeps sensitive data private:

1. **Tokenize** (private) -- Run `logtok tokenize` on your log file. Credentials, IPs, emails, and 19 other categories are replaced with deterministic tokens like `[IP_001]`, `[KEY_002]`.

2. **Diagnose** (public) -- Paste the tokenized output into Claude Code (or any LLM). Add the logtok CLAUDE.md block to your project so Claude understands the token format and reasons about token relationships.

3. **De-tokenize** (private) -- Take Claude's response and run `logtok detokenize`. Tokens are replaced with real values from the encrypted local store. You get a clear, readable diagnosis.

Sensitive data never leaves your machine. The AI only sees tokens.

## Installation

### From Source

```bash
cargo install --path .
```

### From GitHub Releases

Download the binary for your platform from [Releases](../../releases):

| Platform | Binary |
|----------|--------|
| Linux x64 | `logtok-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `logtok-aarch64-unknown-linux-musl.tar.gz` |
| macOS Intel | `logtok-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `logtok-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `logtok-x86_64-pc-windows-msvc.zip` |

## Quick Start

```bash
# Set encryption key for the token store
export LOGTOK_KEY="your-secret-passphrase"

# Step 1: Tokenize your logs
logtok tokenize server.log -o tokenized.log

# Step 2: Paste tokenized.log content into Claude Code for diagnosis
# (Add the CLAUDE.md block to your project first -- see below)

# Step 3: Save Claude's response to a file, then de-tokenize
logtok detokenize response.txt
```

## Usage

### Tokenize

```bash
logtok tokenize <FILE> [OPTIONS]
```

Replace sensitive values in a log file with deterministic tokens.

| Option | Description |
|--------|-------------|
| `<FILE>` | Path to the log file to tokenize |
| `-o, --output <PATH>` | Write tokenized output to file (default: stdout) |
| `--clipboard` | Copy tokenized output to system clipboard |
| `--block-size <BYTES>` | Block size for processing (default: 65536) |
| `--dry-run` | Preview detections without tokenizing |
| `-q, --quiet` | Suppress progress bar |
| `--config <PATH>` | Path to .loktok.toml config file |

**Examples:**

```bash
# Tokenize to stdout
logtok tokenize app.log

# Tokenize to file
logtok tokenize app.log -o safe.log

# Preview what would be detected
logtok tokenize app.log --dry-run

# Copy tokenized output to clipboard
logtok tokenize app.log --clipboard
```

### De-tokenize

```bash
logtok detokenize [FILE] [OPTIONS]
```

Replace `[CATEGORY_NNN]` tokens with real values from the encrypted store.

| Option | Description |
|--------|-------------|
| `[FILE]` | File containing tokenized text (reads stdin if omitted) |
| `--detailed <PATH>` | Write full markdown report to file |
| `--store <PATH>` | Path to .loktok directory (default: CWD/.loktok) |
| `-q, --quiet` | Suppress status messages |

**Examples:**

```bash
# De-tokenize a file
logtok detokenize response.txt

# De-tokenize from stdin (pipe from clipboard, etc.)
pbpaste | logtok detokenize          # macOS
xclip -o | logtok detokenize         # Linux
powershell Get-Clipboard | logtok detokenize  # Windows

# Write detailed markdown report
logtok detokenize response.txt --detailed report.md
```

### Reset Store

```bash
logtok reset-store
```

Delete the encrypted token store. Use this to start fresh with new token mappings.

## Claude Code Integration

To enable Claude Code to understand tokenized logs, add the logtok instruction block to your project's `CLAUDE.md`:

1. Copy the `## Logtok Token-Aware Diagnosis` section from this project's CLAUDE.md
2. Paste it into your project's CLAUDE.md (or create one)
3. Claude Code will automatically read it and understand token format

The instruction block teaches Claude:
- The `[CATEGORY_NNN]` token format
- All 19 detection categories
- How to cross-reference tokens across log lines
- To preserve tokens in responses for de-tokenization

No plugins, no MCP tools, no API keys needed. Just static context.

## Configuration

Create a `.loktok.toml` file in your project root (or any parent directory):

```toml
[detection]
# Disable specific detection categories (all enabled by default)
disabled = ["DOB", "PHONE"]

# Add custom detection patterns
[[detection.custom_patterns]]
name = "INTERNAL_ID"
pattern = "ACCT-[0-9]{6,}"

[store]
# Token TTL in days (default: 30)
ttl_days = 30
```

Config discovery walks up from CWD to find `.loktok.toml`. Use `--config <path>` to override.

## Security Model

- **Token mappings never leave your machine.** The encrypted store (`.loktok/store.enc`) stays local.
- **AES-256-GCM encryption** protects the token store at rest, with Argon2id key derivation from your passphrase.
- **Tokenized output is safe to share.** Tokens like `[IP_001]` reveal nothing about the real values.
- **De-tokenized output contains real values.** Only view it locally.
- **The `.loktok/` directory is auto-gitignored** to prevent accidental commits.

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LOGTOK_KEY` | For store operations | Passphrase for encrypting/decrypting the token store |

Without `LOGTOK_KEY`, logtok operates in memory-only mode (no persistence between runs).

## Detected Categories

logtok detects 19 categories of sensitive data:

| Category | Examples |
|----------|----------|
| IP | `192.168.1.1`, `::1`, `fe80::1` |
| HOST | `db-primary.internal`, `api.example.com` |
| URL | `https://internal.corp/api/v2` |
| PATH | `/etc/secrets/key.pem`, `C:\Users\admin` |
| PORT | `:5432`, `:8080` |
| EMAIL | `admin@company.com` |
| USER | `root`, `deploy-bot` |
| PHONE | `+1-555-123-4567` |
| KEY | `AKIA...`, `sk-...`, `ghp_...` |
| PASS | `password=secret123` |
| CONN | `postgresql://user:pass@host/db` |
| JWT | `eyJhbGciOi...` |
| PEM | `-----BEGIN RSA PRIVATE KEY-----` |
| UUID | `550e8400-e29b-41d4-a716-446655440000` |
| MAC | `00:1A:2B:3C:4D:5E` |
| CC | `4111-1111-1111-1111` |
| SSN | `123-45-6789` |
| DOB | `1990-01-15` |
| CUSTOM | User-defined patterns |

## License

MIT
