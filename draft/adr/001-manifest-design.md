# ADR-001: Manifest Structure and Build Strategy

**Status:** Proposed  
**Date:** 2026-03-24  
**Relates to:** [ADR-001 (project)](../../adr/001-project-goals.md)
(project goals and scope)

## Context

The draft manifest
(`org.freedesktop.Sdk.Extension.qt611.json`) must define how 38 Qt
submodules are fetched, configured, built, and installed inside the
`flatpak-builder` sandbox. Several design choices differ from the only
prior art — the stale
[4c0n/org.freedesktop.Sdk.Extension.qt6](https://github.com/4c0n/org.freedesktop.Sdk.Extension.qt6)
extension (Qt 6.3, runtime 21.08, YAML, git sources, docs + examples
included).

Active Flathub SDK extensions provide the structural reference:

| Extension | Format | Source type | Docs / examples |
|---|---|---|---|
| `org.freedesktop.Sdk.Extension.llvm22` | JSON | archive (tarball) | Disabled at configure (`-DLLVM_INCLUDE_DOCS:BOOL=OFF`) |
| `org.freedesktop.Sdk.Extension.rust-stable` | JSON | archive (pre-built) | Excluded at install (`--without=rust-docs`), cleanup strips `/share/info`, `/share/man` |
| `4c0n/org.freedesktop.Sdk.Extension.qt6` | YAML | git (tag + commit) | Built in separate `*-docs` modules with `ninja docs` and example copying |

The choices below aim to minimise build time, align with current Flathub
conventions, and integrate cleanly with the `qt-sources-gen` source
verification tool.

## Decision

### 1. JSON manifest format

Use a single JSON file (`org.freedesktop.Sdk.Extension.qt611.json`)
rather than YAML. JSON is the canonical format for Flathub manifests,
matches the llvm22 and rust-stable extensions, avoids YAML parsing
ambiguities (bare strings vs numbers, multiline scalars), and is
directly consumable by `flatpak-builder` without a YAML front-end.

### 2. Release tar archives over git sources

Fetch each Qt submodule as a `type: archive` tarball from
`download.qt.io/official_releases/` instead of cloning from
`code.qt.io` with `type: git`.

**Rationale:**

- **Build speed.** Archive extraction is significantly faster than
  cloning the full git history of 38 repositories.
- **Determinism.** A tarball URL + cryptographic hash is a single
  immutable reference. Git sources require both a tag and a pinned
  commit hash to be reproducible.
- **Flathub convention.** Both llvm22 and rust-stable use archive
  sources.
- **Tooling integration.** The `qt-sources-gen` tool is designed around
  archive URLs and SHA-512 hashes. It monitors the upstream
  `md5sums.txt` sentinel to detect when tarballs are re-spun, and
  regenerates hashes automatically — a workflow that has no git
  equivalent.

### 3. SHA-512 source hashes

Use `sha512` rather than `sha256` as the hash property in each archive
source entry:

```json
{ "type": "archive", "url": "…", "sha512": "<hash>" }
```

The flatpak-builder manifest schema accepts `sha512` for `archive` and
`file` source types. The `flatpak-builder-lint` JSON schema validates
it without warnings (only `md5` and `sha1` are flagged as deprecated).

**Rationale:**

- **End-to-end alignment.** The `qt-sources-gen` tool computes SHA-512
  hashes for every Qt archive and stores them in the `sources` file.
  Using `sha512` in the manifest means the same hash flows from tool
  output to manifest without a conversion step.
- **Upstream verification.** Qt publishes `md5sums.txt` for its
  releases. The tool verifies downloaded archives against both the
  upstream MD5 and its own SHA-512. Using SHA-512 in the manifest
  provides stronger integrity than SHA-256 with no additional
  computation.
- **No ecosystem conflict.** While most Flathub manifests use `sha256`,
  `flatpak-builder` treats `sha512` identically in terms of
  verification behaviour. There is no tooling or infrastructure reason
  to prefer `sha256`.

### 4. No documentation or examples

Disable docs and examples for every module:

```
-DQT_BUILD_EXAMPLES=OFF
-DQT_BUILD_TESTS=OFF
```

No separate `*-docs` build modules are included.

**Rationale:**

- **Flathub precedent.** Neither the llvm22 nor rust-stable extension
  ships documentation. LLVM disables docs at configure time; Rust
  excludes them at install time and cleans up residual man/info pages.
- **Build cost.** The old qt6 extension dedicated 4 extra modules
  (`qt-base-docs`, `qt-declarative-docs`, `qt-tools-docs`, `qt-doc`)
  plus `qt-translations` to building and installing documentation and
  examples. Each required a separate CMake configure, `ninja docs`, and
  manual `cp -rn` of example trees — substantial added build time for
  content most Flatpak application consumers never use.
- **Extension size.** Qt documentation and examples add hundreds of
  megabytes to the installed extension. SDK extensions should be as
  lean as possible since every consumer downloads the full extension.
- **Availability elsewhere.** Qt documentation is freely available at
  `doc.qt.io` and through distribution packages. Bundling it inside
  a Flatpak SDK extension adds no unique value.

### 5. Uniform cmake-ninja build recipe

Every module uses an identical build pattern:

```json
{
  "buildsystem": "cmake-ninja",
  "builddir": true,
  "config-opts": [
    "-DCMAKE_BUILD_TYPE=Release",
    "-DQT_BUILD_EXAMPLES=OFF",
    "-DQT_BUILD_TESTS=OFF"
  ]
}
```

Module-specific flags (e.g. `qtbase`'s OpenSSL and install-path
overrides) are added on top of this baseline.

**Rationale:**

- **Predictability.** A uniform recipe means every module builds the
  same way. Deviations are explicit and reviewable.
- **Tooling.** Uniformity makes it feasible to validate or partially
  generate the manifest with tooling — a script can verify that every
  module has the required baseline flags.
- **Maintenance.** Adding a new Qt submodule requires copying the
  template and adding the archive URL. No per-module build-system
  research is needed.

### 6. Dependency-ordered module list

The 38 modules appear in topological dependency order in the manifest:

1. `qtbase` (no Qt dependencies)
2. `qtshadertools` (depends on qtbase)
3. `qtsvg`, `qtimageformats`, `qtlanguageserver` (depend on qtbase)
4. `qtdeclarative` (depends on qtbase, qtshadertools,
   qtlanguageserver)
5. All remaining modules (depend on some subset of the above)
6. `qttranslations` (depends on qttools, late in the order)

`flatpak-builder` builds modules sequentially in manifest order, so
incorrect ordering causes configure-time failures when a module cannot
find its dependencies.

### 7. enable.sh activation script

A `scripts` module installs `enable.sh` which sets:

- `PATH` — find Qt binaries (`qmake`, `moc`, `rcc`, etc.)
- `LD_LIBRARY_PATH` — find Qt shared libraries at runtime
- `PKG_CONFIG_PATH` — find `.pc` files for pkg-config consumers
- `CMAKE_PREFIX_PATH` — find Qt's CMake config packages
- `QT_PLUGIN_PATH` — find Qt plugins (platform, image formats, etc.)
- `QML2_IMPORT_PATH` — find QML modules

This follows the same pattern as llvm22's `enable.sh` (which sets only
`PATH`) but is necessarily more extensive because Qt has more integration
points.

### 8. Metainfo and Flathub metadata

- An `appdata` module installs
  `org.freedesktop.Sdk.Extension.qt611.metainfo.xml` to
  `${FLATPAK_DEST}/share/metainfo/`.
- `flathub.json` restricts builds to `x86_64` and `aarch64`.
- The metainfo explicitly documents that QtWebEngine is excluded,
  setting expectations for consumers.

## Consequences

- The manifest is structurally consistent with active Flathub SDK
  extensions, easing review and acceptance.
- Archive sources with SHA-512 hashes provide end-to-end integrity:
  `qt-sources-gen` computes the hash, the `sources` file stores it,
  and the manifest consumes it — no conversion or second hash algorithm
  needed.
- Omitting docs and examples keeps the extension lean and the build
  fast, at the cost of not providing offline documentation — consumers
  are directed to `doc.qt.io`.
- The uniform build recipe simplifies adding future Qt submodules but
  may need per-module overrides as Qt evolves (e.g. modules requiring
  additional system dependencies or non-standard CMake variables).
- The dependency ordering must be manually maintained; misordering
  will surface as build failures, not warnings.
