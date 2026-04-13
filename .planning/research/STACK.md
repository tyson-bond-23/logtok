# Technology Stack

**Project:** Logs Tokeniser
**Researched:** 2026-04-13

## Language Decision: Rust

**Confidence: HIGH**

Rust is the clear choice over Go for this project. The deciding factors:

1. **Memory efficiency for GB-scale files**: Rust's zero-cost abstractions and lack of GC mean predictable memory usage during block-based log processing. Go's GC causes unpredictable latency spikes when processing large buffers -- exactly the workload this tool handles.
2. **Smaller binaries**: Rust with musl produces ~3MB binaries vs Go's 15MB+. For a tool that ships to developer laptops, CI/CD, Docker, and K8s pods, binary size matters.
3. **Performance**: Rust is consistently 2x faster than Go in text processing benchmarks (regex, JSON parsing, string manipulation). Since this tool's core loop is regex-heavy sensitive data detection across GBs of text, this compounds.
4. **Correctness for security-critical code**: The token store handles encryption. Rust's type system and ownership model prevent classes of bugs (use-after-free, buffer overflows) that matter in security-sensitive code. Go's runtime safety is weaker here.
5. **Cross-platform single binary**: Both languages produce single binaries, but Rust's `cross` tooling and cargo-dist provide a mature pipeline for multi-target release builds.

**Why not Go**: Go would be faster to develop initially but the performance ceiling is lower, binaries are larger, and GC pauses during large file processing are unacceptable. The tool's core is a CPU-bound text processing pipeline -- Rust's sweet spot.

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Rust (stable) | 1.85+ | Language | Zero-cost abstractions, no GC, single binary, memory safety | HIGH |
| clap | 4.6.0 | CLI argument parsing | De facto standard for Rust CLIs. Derive macros auto-generate help text, subcommands, and validation. 4.x is mature and stable. | HIGH |
| tokio | 1.51.1 | Async runtime | Required for async file I/O, streaming, and future Kafka/RabbitMQ integration. The only production-grade async runtime in Rust. | HIGH |
| tokio-stream | 0.1.18 | Stream combinators | Bridges tokio async with the Stream trait for building block-based processing pipelines | HIGH |

### Text Processing & Pattern Matching

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| regex | 1.12.3 | Sensitive data pattern matching | Guarantees O(m*n) worst-case (no catastrophic backtracking). Uses SIMD acceleration. The standard for Rust text matching. | HIGH |
| serde | 1.0.228 | Serialization framework | Universal Rust serialization. Needed for JSON log parsing, config files, token store serialization. | HIGH |
| serde_json | 1.0.149 | JSON parsing | Handles structured JSON logs. Supports streaming deserialization via `from_reader` for large files. | HIGH |
| memmap2 | 0.9.10 | Memory-mapped file I/O | Zero-copy access to large log files. Maps file directly into virtual memory -- no buffer allocation needed. 157M+ downloads, battle-tested. | HIGH |

### Encryption & Security

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| aes-gcm | 0.10.3 | AES-256-GCM encryption | Encrypts token store at rest. Authenticated encryption (AEAD) prevents tampering. Hardware-accelerated on x86 via AES-NI. Security-audited by NCC Group. | HIGH |
| argon2 | 0.5.3 | Key derivation from passphrase | Derives encryption key from user passphrase. Argon2id is the winner of the Password Hashing Competition -- memory-hard, resistant to GPU attacks. Pure Rust. | HIGH |
| rand | 0.9+ | Cryptographic random generation | Nonce generation for AES-GCM. Uses OS entropy source. | HIGH |

### HTTP & API Integration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| reqwest | 0.13.2 | HTTP client | Send tokenized logs to Claude API. Async, supports streaming responses, TLS built-in. Industry standard Rust HTTP client. | HIGH |
| anthropic-sdk-rust | 0.1.1 | Claude API typed client | Provides typed request/response structs and streaming support for Claude Messages API. Wraps reqwest. | LOW |

**Note on Claude API integration**: The `anthropic-sdk-rust` crate is at 0.1.1 and community-maintained. There is no official Anthropic Rust SDK. Two strategies:

- **Recommended (v1)**: Build a thin API client module directly using `reqwest`. The Anthropic Messages API is a single POST endpoint (`/v1/messages`) with well-documented JSON schema. A hand-rolled client of ~200 lines gives full control over streaming, retries, and error handling without depending on an immature third-party crate.
- **Future**: Adopt an SDK crate if one matures to 1.0+ or if Anthropic releases an official Rust SDK.

### Parallel Processing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| rayon | 1.11.0 | Data parallelism | Parallel regex matching across log blocks. `par_iter()` makes parallelization trivial. 266M+ downloads. | HIGH |

### Observability & UX

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| tracing | 0.1.44 | Structured logging/diagnostics | Internal logging for the tool itself. Async-aware, structured, composable. | HIGH |
| tracing-subscriber | 0.3+ | Log output formatting | Console output for tracing. Pairs with tracing. | HIGH |
| indicatif | 0.18.4 | Progress bars | Visual feedback during large file processing. Supports file-size display, ETA, throughput. | HIGH |
| anyhow | 1.0.102 | Error handling (application) | Ergonomic error handling with context chains. For the binary, not library code. | HIGH |
| thiserror | 2.0+ | Error handling (library) | Derive macros for structured error types in core library modules (tokenizer, store, API client). | HIGH |

### Cross-Platform Build & Distribution

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| cross | latest | Cross-compilation | Docker-based cross-compilation to Linux/Windows/ARM from any host. Single command: `cross build --target aarch64-unknown-linux-musl` | HIGH |
| cargo-dist | latest | Release binary packaging | Generates GitHub release artifacts with installers for all platforms. Handles signing, checksums, shell/PowerShell installers. | MEDIUM |

### Testing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| assert_cmd | 2.0+ | CLI integration tests | Test the compiled binary end-to-end with argument parsing and output assertions. | HIGH |
| predicates | 3.0+ | Test assertions | Composable predicates for asserting on command output. Pairs with assert_cmd. | HIGH |
| tempfile | 3.0+ | Temporary test files | Create temporary log files and token stores for testing. Cleans up automatically. | HIGH |
| insta | 1.40+ | Snapshot testing | Snapshot test tokenization output to catch regressions in pattern matching. | MEDIUM |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Language | Rust | Go | GC pauses on large files, larger binaries, weaker type safety for crypto code |
| Language | Rust | Node.js/Bun | Not viable for single binary distribution, poor performance for GB-scale text processing |
| CLI framework | clap | structopt | Merged into clap 3+. structopt is deprecated. |
| CLI framework | clap | argh | Smaller but less featured. clap's ecosystem (completions, man pages) is worth it. |
| Async runtime | tokio | async-std | Smaller ecosystem, fewer maintained integrations, less battle-tested |
| HTTP client | reqwest | hyper (direct) | reqwest wraps hyper with sane defaults. No need for low-level HTTP control. |
| HTTP client | reqwest | ureq | ureq is sync-only. We need async for streaming Claude responses. |
| Encryption | aes-gcm | ring | ring is C-backed (harder to cross-compile), aes-gcm is pure Rust with HW acceleration |
| Encryption | aes-gcm | sodiumoxide | Wraps libsodium (C library). Cross-compilation friction. RustCrypto is pure Rust. |
| JSON | serde_json | simd-json | simd-json is faster but requires unsafe code and nightly features. Marginal gain for our use case. |
| Progress | indicatif | pbr | indicatif is more actively maintained, better API, supports multi-bar |
| Parallelism | rayon | manual threadpool | Rayon handles work-stealing and load balancing automatically. No reason to hand-roll. |
| File I/O | memmap2 | std::fs::read | read() loads entire file into memory. memmap2 uses OS virtual memory for zero-copy access. Critical for GB-scale files. |
| Claude API | reqwest (direct) | anthropic-sdk-rust | SDK is 0.1.1, immature, could break. The API is one endpoint -- direct reqwest is more reliable. |

## Architecture-Relevant Stack Notes

### Block Processing Pipeline
The stack enables a block-based streaming architecture:
```
File (memmap2) -> Block Iterator (tokio-stream) -> Parallel Regex (rayon) -> Tokenized Output
```
- `memmap2` provides zero-copy file access
- `tokio-stream` manages block boundaries and backpressure
- `rayon` parallelizes regex matching within each block
- This architecture directly maps to future Kafka/RabbitMQ streaming (replace file source with message consumer)

### Encryption Pipeline
```
Passphrase -> argon2 (key derivation) -> AES-256-GCM (encrypt/decrypt token store)
```
- Key derived per-session from passphrase, never stored
- Token store is a serde-serialized data structure, encrypted as a single blob
- Nonces generated per-encryption via `rand`

### Cross-Platform Targets
| Target Triple | Platform | Notes |
|---------------|----------|-------|
| x86_64-unknown-linux-musl | Linux x64 | Static linking, no glibc dependency |
| aarch64-unknown-linux-musl | Linux ARM64 | K8s on ARM, Graviton |
| x86_64-apple-darwin | macOS Intel | Requires macOS SDK for cross-compile |
| aarch64-apple-darwin | macOS Apple Silicon | Primary dev target |
| x86_64-pc-windows-msvc | Windows x64 | Requires MSVC linker or cross w/ mingw |

**macOS cross-compilation caveat**: Apple's licensing prevents Docker-based cross-compilation. macOS builds should run on macOS CI (GitHub Actions `macos-latest` runner). All other targets can cross-compile from any host via `cross`.

## Installation

```bash
# Create project
cargo init logs-tokeniser
cd logs-tokeniser

# Core dependencies
cargo add clap --features derive
cargo add tokio --features full
cargo add tokio-stream
cargo add serde --features derive
cargo add serde_json
cargo add regex
cargo add memmap2
cargo add rayon
cargo add reqwest --features json,stream,rustls-tls
cargo add aes-gcm
cargo add argon2
cargo add rand
cargo add tracing
cargo add tracing-subscriber --features env-filter
cargo add indicatif
cargo add anyhow
cargo add thiserror

# Dev dependencies
cargo add --dev assert_cmd
cargo add --dev predicates
cargo add --dev tempfile
cargo add --dev insta
```

## Cargo.toml Profile Configuration

```toml
[profile.release]
opt-level = 3
lto = true        # Link-time optimization for smaller, faster binary
strip = true      # Strip debug symbols
codegen-units = 1 # Single codegen unit for maximum optimization
panic = "abort"   # Smaller binary, no unwinding overhead
```

## Sources

- [clap 4.6.0 docs](https://docs.rs/clap/latest/clap/) - Verified version via docs.rs
- [tokio 1.51.1 docs](https://docs.rs/tokio/latest/tokio/) - Verified version via docs.rs
- [regex 1.12.3 docs](https://docs.rs/regex/latest/regex/) - Verified version via docs.rs
- [serde 1.0.228 docs](https://docs.rs/serde/latest/serde/) - Verified version via docs.rs
- [serde_json 1.0.149 docs](https://docs.rs/serde_json/latest/serde_json/) - Verified version via docs.rs
- [aes-gcm 0.10.3 docs](https://docs.rs/aes-gcm/latest/aes_gcm/) - Verified version via docs.rs, NCC Group audited
- [argon2 0.5.3 docs](https://docs.rs/argon2/latest/argon2/) - Verified version via docs.rs
- [memmap2 0.9.10 docs](https://docs.rs/memmap2/latest/memmap2/) - Verified version via docs.rs
- [reqwest 0.13.2 docs](https://docs.rs/reqwest/latest/reqwest/) - Verified version via docs.rs
- [rayon 1.11.0 docs](https://docs.rs/rayon/latest/rayon/) - Verified version via docs.rs
- [indicatif 0.18.4 docs](https://docs.rs/indicatif/latest/indicatif/) - Verified version via docs.rs
- [tracing 0.1.44 docs](https://docs.rs/tracing/latest/tracing/) - Verified version via docs.rs
- [anyhow 1.0.102 docs](https://docs.rs/anyhow/latest/anyhow/) - Verified version via docs.rs
- [anthropic-sdk-rust 0.1.1 docs](https://docs.rs/anthropic-sdk-rust/latest/anthropic_sdk/) - Verified version via docs.rs
- [Rust vs Go for AI Tooling comparison](https://dasroot.net/posts/2026/03/rust-vs-go-ai-tooling-comparison/)
- [RustCrypto AEADs (aes-gcm)](https://github.com/RustCrypto/AEADs)
- [cargo cross](https://github.com/cross-rs/cross)
- [Cross-platform Rust CI/CD pipeline](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/)
