# org.freedesktop.Sdk.Extension.qt611

A Flatpak SDK extension providing Qt 6.11 for the
`org.freedesktop.Sdk` 25.08 runtime.

## Overview

This extension builds 39 Qt 6.11 submodules from source, giving Flatpak
applications access to Qt without depending on the full KDE runtime.
QtWebEngine and QtActiveQt are excluded.

Depends on `org.freedesktop.Sdk.Extension.llvm22` for the shader
toolchain.

## Usage

Add the extension to your Flatpak manifest:

```json
{
  "sdk-extensions": [
    "org.freedesktop.Sdk.Extension.qt611"
  ]
}
```

Activate it in your build options or at runtime via the `enable.sh`
script:

```json
{
  "build-options": {
    "append-path": "/usr/lib/sdk/qt611/bin",
    "prepend-ld-library-path": "/usr/lib/sdk/qt611/lib",
    "env": {
      "CMAKE_PREFIX_PATH": "/usr/lib/sdk/qt611"
    }
  }
}
```

## Project Structure

```
adr/                  Architecture decision records
docs/notes/           Reference materials (submodules + notes)
draft/                Draft extension manifest and metadata
packages/
  qt-sources-gen/     Rust tool — sentinel-based source hash verification
sources               SHA-512 checksums for Qt archives (CI-managed)
```

## Source Verification

The `qt-sources-gen` tool monitors Qt's upstream `md5sums.txt` file. A
SHA-512 sentinel in the `sources` file detects when archives change. A
weekly GitHub Actions workflow re-checks and regenerates hashes as
needed. All tool output is JSON (see [ADR-002](adr/002-json-structured-output.md)).

## Development

```sh
cargo build --workspace
cargo test --workspace --all-targets
```

## License

Code is licensed under [MIT-0](LICENSE).

Documentation and architecture decision records are licensed under
[CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/).
