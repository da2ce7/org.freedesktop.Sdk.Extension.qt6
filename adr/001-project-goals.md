# ADR-001: Project Goals and Scope

**Status:** Decided  
**Date:** 2026-03-24

## Context

The Flatpak ecosystem provides SDK extensions as a mechanism for shipping
development libraries and tooling that augment the base
`org.freedesktop.Sdk` runtime. KDE applications built with Flatpak
currently rely on the KDE runtime (`org.kde.Sdk`), which bundles Qt
along with the full KDE Frameworks stack. Applications that need Qt but
not KDE Frameworks — or that need a newer Qt than the KDE runtime ships
— have no clean option: they must either vendor Qt into each app
manifest or adopt the full KDE runtime as dead weight.

A standalone Qt SDK extension would decouple Qt from KDE Frameworks,
letting Flatpak-packaged applications depend on exactly Qt and nothing
more. No such extension exists on Flathub today for any modern Qt 6
release.

### Prior art

| Reference | Approach | Status |
|---|---|---|
| [4c0n/org.freedesktop.Sdk.Extension.qt6](https://github.com/4c0n/org.freedesktop.Sdk.Extension.qt6) | Source-build of Qt 6.3, runtime 21.08 | Stale — unmaintained since 2022 |
| `org.freedesktop.Sdk.Extension.llvm22` (Flathub) | LLVM SDK extension, runtime 25.08 | Active — structural reference for JSON manifest format |
| `org.fedoraproject.KDE6Sdk` (Fedora) | RPM-based, packages all qt6-\* modules | Active — module list reference only (different packaging model) |

### Constraints

- **No QtWebEngine.** QtWebEngine embeds Chromium and is prohibitively
  large to build (~40 GB intermediate objects, hours of compile time).
  Applications needing WebEngine should use a separate dedicated
  extension or the system library.
- **No QtActiveQt.** Windows-only module — irrelevant for the Linux
  Flatpak target.
- **LLVM dependency.** Qt 6.11's shader toolchain (`qtshadertools`)
  requires LLVM/Clang. Rather than bundling LLVM, the extension depends
  on `org.freedesktop.Sdk.Extension.llvm22`, following Flathub
  convention for composable SDK extensions.
- **Source build.** Unlike the Fedora container approach (pre-built
  RPMs), Flathub extensions must build from source inside the
  `flatpak-builder` sandbox. All 39 Qt submodules are built with
  `cmake-ninja`.
- **CI-driven source verification.** Qt source archives change without
  URL changes when point releases are re-spun. A Rust tool
  (`qt-sources-gen`) monitors the upstream `md5sums.txt` sentinel and
  regenerates SHA-512 checksums when the sentinel hash changes, keeping
  the `sources` file current via a scheduled GitHub Actions workflow.

## Decision

Build and publish `org.freedesktop.Sdk.Extension.qt611` on Flathub with
the following characteristics:

1. **Target runtime:** `org.freedesktop.Sdk` / `org.freedesktop.Platform`
   **25.08**.
2. **SDK dependency:** `org.freedesktop.Sdk.Extension.llvm22` (provides
   clang for qtshadertools and other LLVM-dependent modules).
3. **Install prefix:** `/usr/lib/sdk/qt611` (standard SDK extension
   layout; applications activate it via `enable.sh`).
4. **Qt version:** 6.11.0 (released 2026-03-23), tracking the 6.11.x
   series for patch updates.
5. **Module set:** 39 submodules built from source in dependency order —
   `qtbase` first, then `qtshadertools`, `qtdeclarative`, and the
   remainder. QtWebEngine and QtActiveQt are excluded.
6. **Architecture:** x86\_64 and aarch64.
7. **Source integrity:** The `sources` file in the repository holds
   SHA-512 hashes for every archive, with a sentinel line
   (`SHA512 (md5sums.txt) = …`) detecting upstream changes. The
   `qt-sources-gen` Rust tool automates regeneration.
8. **Manifest format:** Single JSON file
   (`org.freedesktop.Sdk.Extension.qt611.json`) with `build-extension:
   true`, following the pattern established by the LLVM22 extension.

## Consequences

- Flatpak applications can add a single `sdk-extensions` entry to use
  Qt 6.11 without pulling in KDE Frameworks or vendoring Qt.
- Updating to Qt 6.11.x patch releases requires only re-running the
  source generator and rebuilding — no manifest structural changes.
- QtWebEngine users are explicitly out of scope and must find an
  alternative (documented in the metainfo description).
- The LLVM22 dependency means the extension cannot be used with runtimes
  older than 25.08 or without the LLVM extension installed.
- Maintenance burden: weekly CI checks for upstream archive changes, and
  manual manifest updates for new Qt minor versions (6.12, etc.).
