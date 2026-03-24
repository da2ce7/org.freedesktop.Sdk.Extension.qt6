# ADR-002: Manifest Linting with flatpak-builder-lint

**Status:** Proposed  
**Date:** 2026-03-24  
**Relates to:** [ADR-001 (draft)](001-manifest-design.md) (manifest
structure and build strategy)

## Context

Flathub requires all submitted manifests to pass
[flatpak-builder-lint](https://github.com/flathub-infra/flatpak-builder-lint),
the official linter maintained by `flathub-infra`. The linter validates
manifests at three levels:

1. **Manifest** — JSON schema conformance, top-level properties, module
   source hygiene (no deprecated hash algorithms, no bare git branches,
   no network access in build args, etc.).
2. **Build output** — metainfo/appstream validation, ELF architecture
   checks, desktop file presence (run after `flatpak-builder`
   completes).
3. **Repository** — OSTree ref structure, catalogue metadata (run on
   the published repo; only relevant on the Flathub build
   infrastructure).

The manifest-level checks can be run offline with no special
dependencies beyond Python and the linter package. Build-output and
repo-level checks require OSTree GObject introspection bindings and a
completed `flatpak-builder` build, making them impractical to run in a
lightweight CI job.

### Current linter findings (manifest level)

Running the linter's manifest checks against the draft produces:

| Check | Result |
|---|---|
| JSON schema validation | Passed |
| Top-level structure (`build-extension`, `branch`, `modules`) | Passed |
| Source hash algorithms (no md5/sha1) | Passed |
| `flathub.json` arch constraints | Passed |
| Source hashes populated | 39 modules have placeholder values — expected in draft stage |

No errors or warnings. The manifest is structurally ready for
submission once source hashes are populated.

### Checks that cannot run offline

| Check | Requires |
|---|---|
| Metainfo / appstream validation | `appstream-util` or `appstreamcli` |
| ELF architecture verification | Completed build artifacts |
| Desktop file checks | Not applicable (SDK extension, not an app) |
| OSTree repo checks | Published repo |

## Decision

### 1. Run manifest-level linting in CI

Add a GitHub Actions job that runs `flatpak-builder-lint manifest` on
every push and pull request targeting the draft manifest. This catches
regressions in manifest structure without needing a full
`flatpak-builder` build.

Use the official container image
`ghcr.io/flathub-infra/flatpak-builder-lint` to avoid dependency
issues (the linter requires PyGObject and OSTree GI bindings even for
the manifest-level entry point).

```yaml
lint-manifest:
  runs-on: ubuntu-latest
  container:
    image: ghcr.io/flathub-infra/flatpak-builder-lint
  steps:
    - uses: actions/checkout@v4
    - run: flatpak-builder-lint manifest draft/org.freedesktop.Sdk.Extension.qt611.json
```

### 2. Use SHA-512 hashes in the manifest

The flatpak-builder manifest schema supports `sha512` as a source hash
property alongside `sha256`. The `qt-sources-gen` tool already computes
SHA-512 hashes for all Qt archives (stored in the `sources` file). Using
`sha512` directly in the manifest eliminates the need to compute or
maintain a separate SHA-256 for each archive.

Switch the draft manifest's source entries from:

```json
{ "type": "archive", "url": "…", "sha256": "FIXME" }
```

to:

```json
{ "type": "archive", "url": "…", "sha512": "FIXME" }
```

This is validated by the linter's JSON schema (which accepts `sha512`
for `archive` and `file` source types) and by `flatpak-builder` itself.

**Note:** The linter flags `md5` and `sha1` as deprecated but accepts
both `sha256` and `sha512` without warnings.

### 3. Validate metainfo XML separately

The metainfo file
(`org.freedesktop.Sdk.Extension.qt611.metainfo.xml`) can be validated
with `appstreamcli validate` independently of a full build. Add a
second CI step using a container or distro package that provides
`appstreamcli`:

```yaml
- run: appstreamcli validate --no-net draft/org.freedesktop.Sdk.Extension.qt611.metainfo.xml
```

### 4. Accept that build-output and repo checks run only on Flathub

Build-output checks (`flatpak-builder-lint builddir`) and repo checks
(`flatpak-builder-lint repo`) require a completed `flatpak-builder`
build or OSTree repo respectively. These run automatically on the
Flathub build infrastructure during submission review. There is no
value in replicating them locally — the full Qt build takes hours and
multiple gigabytes.

## Consequences

- CI catches manifest schema regressions, deprecated hash usage, and
  structural issues on every PR — before the lengthy Flathub build.
- Using `sha512` in the manifest aligns source hashes end-to-end:
  `qt-sources-gen` computes SHA-512, the `sources` file stores SHA-512,
  and the manifest consumes SHA-512. No hash conversion step is needed.
- Metainfo validation catches appstream issues early (missing fields,
  invalid XML) without a full build.
- Build-output and repo-level lint failures will only surface during
  the Flathub submission build. This is acceptable because those checks
  validate build artifacts, not manifest authoring.
