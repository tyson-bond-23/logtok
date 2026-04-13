<!-- GSD:project-start source:PROJECT.md -->
## Project

**Logs Tokeniser**

A high-performance, cross-platform CLI tool that tokenizes sensitive data out of application logs — credentials, infrastructure details, business logic, and PII — so they can be safely analyzed by Claude Code for error diagnosis. Claude's results are then de-tokenized back into meaningful, readable output without ever exposing the original secrets.

**Core Value:** Engineers can diagnose production log errors through Claude Code without revealing any sensitive information — secrets, internal architecture, business logic, or PII never leave the local environment.

### Constraints

- **Performance**: Must handle large log files (GBs) without excessive memory usage — block/stream processing required
- **Security**: Token mappings never transmitted — encrypted at rest, decrypted only locally
- **Portability**: Single binary, zero runtime dependencies, cross-platform (Windows, macOS, Linux, ARM)
- **Privacy**: No sensitive data should appear in any output, intermediate file, or network request
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Language Decision: Rust
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
- `memmap2` provides zero-copy file access
- `tokio-stream` manages block boundaries and backpressure
- `rayon` parallelizes regex matching within each block
- This architecture directly maps to future Kafka/RabbitMQ streaming (replace file source with message consumer)
### Encryption Pipeline
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
## Installation
# Create project
# Core dependencies
# Dev dependencies
## Cargo.toml Profile Configuration
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
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
