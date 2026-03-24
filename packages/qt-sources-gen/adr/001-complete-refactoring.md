# ADR-001: Complete Refactoring of qt-sources-gen

**Status:** Decided  
**Date:** 2026-03-24  
**Relates to:** [ADR-001 (project)](../../adr/001-project-goals.md)
(CI-driven source verification),
[ADR-002 (project)](../../adr/002-json-structured-output.md) (JSON
structured output)

## Context

`qt-sources-gen` has grown from a simple sentinel-checker into a tool
that downloads ~39 Qt archive tarballs, computes dual hashes (MD5 +
SHA-512), parses `md5sums.txt`, and supports multiple operating modes
(`--dry-run`, `--force`). All of this logic currently lives in a single
`check_sources` function spanning ~130 lines in `lib.rs`, with several
concerns interleaved:

- **HTTP transport** — building the client, downloading bodies, streaming
  downloads with dual hashing.
- **Sentinel management** — fetching `md5sums.txt`, computing its
  SHA-512, comparing against the stored sentinel line.
- **Hash verification** — parsing upstream MD5 expectations, parsing
  stored SHA-512 values, verifying downloaded archives against both.
- **Sources file I/O** — reading the existing `sources` file, writing
  updated content.
- **Mode dispatch** — branching on `dry_run`, `force`, and
  `sentinel_changed` to decide what work to perform.

This monolithic structure makes it difficult to:

1. **Test in isolation.** Testing sentinel logic requires mocking HTTP.
   Testing hash verification requires real or faked download results.
   Everything is coupled through one function.
2. **Add new features.** Parallel downloads, retry logic, progress
   reporting, or per-module partial updates all require threading
   through the same function.
3. **Reason about correctness.** The interaction between `force`,
   `dry_run`, and `sentinel_changed` produces a matrix of behaviours
   that is hard to follow in a single flow.

## Decision

Refactor `qt-sources-gen` into clearly separated modules with
well-defined responsibilities:

### 1. Module structure

```
src/
  lib.rs          — public API re-exports
  client.rs       — HTTP client construction and download primitives
  sentinel.rs     — md5sums.txt fetching, SHA-512 sentinel computation, comparison
  sources.rs      — sources file parsing and serialization (read/write hash lines)
  verify.rs       — archive download, dual-hash (MD5 + SHA-512) computation, verification
  modules.rs      — MODULES list and filename/URL construction
  main.rs         — CLI argument parsing and orchestration
```

### 2. Responsibility boundaries

| Module | Owns | Does not touch |
|---|---|---|
| `client` | `reqwest::blocking::Client` builder, `download_body`, `download_and_hash` (streaming dual-hash) | Sentinel logic, file I/O |
| `sentinel` | Fetching `md5sums.txt`, computing its SHA-512, comparing against stored value | Archive downloads, sources file writing |
| `sources` | Parsing `SHA512 (<file>) = <hash>` lines, serializing them back, reading/writing the sources file | HTTP, hashing |
| `verify` | Orchestrating per-module download + MD5/SHA-512 verification against expected values | File I/O (receives expected hashes as input) |
| `modules` | `MODULES` list, `BASE_URL`, filename and URL formatting helpers | Everything else |
| `main` | CLI args, mode dispatch (`dry_run` / `force` / normal), exit codes, JSON output | Business logic |

### 3. Public API

Replace the current `check_sources(version, path, dry_run, force) →
CheckResult` monolith with composable building blocks:

```rust
// sentinel.rs
pub fn fetch_sentinel(client: &Client, version: &str) -> Result<Sentinel>;
pub fn compare_sentinel(current: &Sentinel, sources: &SourcesFile) -> SentinelStatus;

// sources.rs
pub fn read_sources(path: &str) -> Result<SourcesFile>;
pub fn write_sources(path: &str, content: &SourcesFile) -> Result<()>;

// verify.rs
pub fn verify_archives(client: &Client, version: &str, expected_md5s: &HashMap<String, String>, stored_sha512s: &HashMap<String, String>) -> VerifyResult;
```

`main.rs` composes these into the same three modes:

- **Normal** (`sentinel changed → download + hash + write`)
- **Force** (`sentinel unchanged → download + verify MD5 + verify SHA-512`)
- **Dry run** (`sentinel check only → early exit`)

### 4. Error handling

Replace `Box<dyn Error>` with a dedicated error enum (using `thiserror`)
so callers can match on specific failure kinds (HTTP error, hash
mismatch, I/O error) rather than inspecting error strings.

### 5. Testing

Each module gets its own unit tests:

- `sentinel` — test parsing, comparison, sentinel line formatting.
- `sources` — test round-trip read/write, handling of missing files,
  malformed lines.
- `verify` — test MD5/SHA-512 verification logic with known byte
  payloads (no network).
- Integration tests — keep the existing `dry_run.rs` tests for
  end-to-end validation against the real Qt mirror.

## Consequences

- **Easier to test:** Each module can be tested with synthetic inputs,
  without needing HTTP mocks for unrelated logic.
- **Easier to extend:** Parallel downloads, retry policies, or
  alternative hash algorithms can be added in `client.rs` / `verify.rs`
  without touching sentinel or file I/O code.
- **More files:** The single `lib.rs` splits into 5–6 files. This is
  a modest increase for a small tool, but the separation pays for
  itself in readability.
- **Migration risk:** The refactoring must preserve the existing JSON
  output contract (ADR-002) and exit code semantics. The existing
  integration tests serve as a regression safety net.
- **`check_sources` removal:** The current monolithic public function
  will be removed. Any external consumers (currently only `main.rs`
  and tests) must migrate to the new composable API.
