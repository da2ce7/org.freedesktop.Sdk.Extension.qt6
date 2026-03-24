# ADR-003: CI Strategy for the Draft Manifest

**Status:** Accepted  
**Date:** 2026-03-24  
**Relates to:** [ADR-001](001-project-goals.md) (project goals),
[ADR-001 (draft)](../draft/adr/001-manifest-design.md) (manifest
design), [ADR-002 (draft)](../draft/adr/002-manifest-linting.md)
(manifest linting)

## Context

The draft manifest (`draft/org.freedesktop.Sdk.Extension.qt611.json`)
defines 39 Qt submodules plus `scripts` and `appdata` modules — 41
modules total. Before submitting to Flathub, we need CI that catches
regressions in manifest structure, source integrity, and basic build
viability.

Flathub's own build infrastructure runs the full build on dedicated
machines with ample disk and CPU when a manifest is submitted for
review. Replicating that full build in our CI is expensive:

| Constraint | Value |
|---|---|
| Qt full-build disk usage | 30–60 GB intermediates |
| Qt full-build time (4 cores) | 2–4+ hours |
| GitHub Actions standard runner | 4 cores, ~14 GB RAM, ~14 GB free disk |
| GitHub Actions large runner | 8+ cores, ~84 GB disk (paid) |
| Runtime + SDK + llvm22 download | ~2–3 GB |
| Source tarballs (38 archives) | ~1 GB |

A full build does not fit on standard runners and consumes excessive CI
minutes even on large runners. The CI strategy must balance coverage
against cost.

### Validation tiers

Three tiers of validation exist, each with different cost/value
profiles:

**Tier 1 — Lightweight checks (seconds, standard runners).** Manifest
schema linting, metainfo XML validation, JSON well-formedness, and
hash consistency between the `sources` file and the manifest.

**Tier 2 — Partial build (20–40 minutes, standard runners).** Build
`qtbase` only using `flatpak-builder --stop-at=qtshadertools`. This
validates that the runtime/SDK/extension dependencies resolve, the
`build-options` block is correct, CMake finds system libraries (OpenSSL,
etc.), and the install-prefix layout works. `qtbase` is the foundation
module — if it builds, the remaining modules' build recipes are
structurally identical and unlikely to fail for manifest reasons.

**Tier 3 — Full build (hours, Flathub infrastructure).** Builds all 41
modules. Runs `flatpak-builder-lint builddir` and `repo`-level checks.
This happens automatically on Flathub's buildbot during submission
review. There is no value in replicating it locally.

### Reference extension CI patterns

Active Flathub SDK extensions (llvm22, rust-stable) do not run
`flatpak-builder` in their own CI — they rely on Flathub's buildbot for
the full build. Their repositories contain only the manifest and
metadata files; CI, if any, is limited to automated dependency update
PRs via `flatpak-external-data-checker`.

Our repository is different: it contains both the Rust tool
(`qt-sources-gen`) and the draft manifest, so we can add manifest
checks alongside the existing Rust CI.

### `flathub.json` flags

The llvm22 extension's `flathub.json` sets `skip-appstream-check` and
`skip-icons-check`. SDK extensions have no desktop icons and have
relaxed appstream requirements. Our current `flathub.json` only sets
`only-arches` — it should gain these skip flags as well.

## Decision

### 1. Three-job workflow triggered on draft/sources changes

Add `.github/workflows/draft-manifest.yml` with three jobs, triggered
on pushes and pull requests that touch `draft/**` or `sources`:

| Job | Runner | Container | Purpose |
|---|---|---|---|
| `lint-manifest` | `ubuntu-latest` | `ghcr.io/flathub-infra/flatpak-builder-lint` | `flatpak-builder-lint manifest` |
| `validate-metainfo` | `ubuntu-latest` | — (install `appstream` via apt) | `appstreamcli validate --no-net` |
| `check-hashes` | `ubuntu-latest` | — | Cross-check manifest `sha512` values against `sources` file |

These three jobs run in parallel and complete in under a minute.

### 2. Optional qtbase smoke-test build

Add a fourth job `build-qtbase` gated behind the three lint jobs. It
installs Flatpak, fetches the 25.08 SDK + llvm22 extension, and runs:

```
flatpak-builder --user --force-clean \
  --install-deps-from=flathub \
  --stop-at=qtshadertools \
  builddir \
  draft/org.freedesktop.Sdk.Extension.qt611.json
```

This builds only `qtbase` (~20–40 minutes, ~5–8 GB disk) and catches
build-option or CMake configuration errors. It runs on standard runners.

This job should be optional (not required for merge) because it is slow
and consumes CI minutes. It can be triggered:
- On pull requests touching `draft/**` (to validate before merge)
- Via `workflow_dispatch` for manual testing
- **Not** on every push to reduce CI cost

### 3. No full build in our CI

The full 41-module build runs only on Flathub's buildbot. We do not
attempt to replicate it. Tier 2 (qtbase only) gives sufficient
confidence that the manifest structure and build-options are correct.

### 4. Hash consistency check

The `check-hashes` job extracts every `sha512` value from the manifest
with `jq` and verifies it matches the corresponding line in the
`sources` file. This catches copy-paste errors or stale hashes after a
`qt-sources-gen` run that updated `sources` but not the manifest.

The `scripts` and `appdata` modules use `type: script` and `type: file`
sources (no archive hashes) — the check skips those.

### 5. Separate workflow file

The manifest CI lives in `.github/workflows/draft-manifest.yml`,
separate from the Rust tool CI in `testing.yml` and the source
generation CI in `generate-sources.yml`. This keeps concerns separated:
each workflow triggers on its own path set and has independent failure
semantics.

### 6. Update `flathub.json`

Add `skip-appstream-check` and `skip-icons-check` to
`draft/flathub.json`, matching the llvm22 extension convention for SDK
extensions:

```json
{
  "only-arches": ["x86_64", "aarch64"],
  "skip-appstream-check": true,
  "skip-icons-check": true
}
```

## Consequences

- Manifest structure regressions are caught in seconds on every
  PR — before committing to a multi-hour Flathub build.
- The hash consistency check ensures the `sources` file and manifest
  stay synchronised, closing the gap where `qt-sources-gen` updates one
  but not the other.
- The qtbase smoke test validates the most complex module (custom
  CMake flags, OpenSSL linking, install-path overrides) without
  building all 39 Qt modules.
- Full-build validation is deferred to Flathub's buildbot. Build-output
  lint failures (`flatpak-builder-lint builddir`) will only surface
  during Flathub submission review.
- CI minutes are kept low: the lint jobs are free (seconds), and the
  qtbase build (~30 min) runs only on PRs or manual dispatch.
- Adding `skip-appstream-check` and `skip-icons-check` to
  `flathub.json` prevents false failures on Flathub's side for checks
  that do not apply to SDK extensions.
