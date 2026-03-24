# ADR-002: JSON Structured Output for All Helper Tools

**Status:** Decided  
**Date:** 2026-03-24  
**Relates to:** [ADR-001](001-project-goals.md) (CI-driven source
verification — `qt-sources-gen` is the first tool affected)

## Context

The project includes Rust helper tools (currently `qt-sources-gen`,
with more expected) that run in GitHub Actions CI. Their output is
consumed by workflow steps — detecting changes, committing updated
files, and surfacing diagnostics in CI logs.

Plain-text output is fragile to parse: downstream workflow steps resort
to `grep`, `awk`, or regex matching to extract status, filenames, or
error messages. Log lines interleaved with program output make this
worse. As more tools are added, maintaining ad-hoc text parsers across
workflows becomes an ongoing maintenance burden.

Structured JSON output is a common convention for CLI tools designed for
machine consumption (`cargo --message-format=json`, `rustfmt
--emit=json`, `jq`, etc.). GitHub Actions natively supports `jq` and
`fromJson()` in expressions, making JSON trivial to consume in workflow
files.

## Decision

All helper tools in this repository **must** emit JSON as their sole
output format:

1. **Program output (stdout):** A single JSON object (or
   newline-delimited JSON objects) written to stdout. Workflow steps
   parse this with `jq` or `fromJson()`. No plain-text banners,
   progress bars, or human-readable summaries on stdout.

2. **Logs (stderr):** Use `tracing-subscriber` with the
   `tracing_subscriber::fmt::format::Json` formatter, writing
   structured JSON log events to stderr. The `RUST_LOG` env var
   controls verbosity as usual. CI can forward stderr to log sinks or
   parse individual events if needed.

3. **Exit codes:** Remain numeric (0 = success, non-zero = failure).
   Exit codes carry coarse status; the JSON output carries details.

### Example: `qt-sources-gen`

**stdout** on sentinel mismatch (exit 1):
```json
{"status":"changed","sentinel":"SHA512 (md5sums.txt) = abc123...","modules":38}
```

**stdout** when up to date (exit 0):
```json
{"status":"up_to_date"}
```

**stderr** (structured log events):
```json
{"timestamp":"...","level":"INFO","message":"fetching md5sums.txt","url":"https://..."}
{"timestamp":"...","level":"INFO","message":"sentinel unchanged — sources are up to date"}
```

### Implementation rule

When adding a new helper tool, configure `tracing-subscriber` with
`.json()` on stderr, and emit program results as JSON on stdout.
Human-readable output may be added behind an explicit `--human` flag
if needed for local debugging, but JSON is the default.

## Consequences

- CI workflow steps can use `jq` to extract fields reliably, without
  fragile text parsing.
- Logs and program output are cleanly separated (stderr vs stdout),
  both in structured format.
- Human readability of raw CI logs is reduced — structured JSON log
  lines are verbose. Mitigated by CI log viewers that can render JSON
  and by the optional `--human` escape hatch.
- All existing and future tools must be updated/written to conform
  before merging to main.
