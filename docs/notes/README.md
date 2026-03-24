# Reference Materials

Prior art and reference files used during development of
`org.freedesktop.Sdk.Extension.qt611`. These are not part of the
extension itself.

## Submodules

| Directory | Source | Notes |
|---|---|---|
| `org.freedesktop.Sdk.Extension.llvm22/` | [flathub/org.freedesktop.Sdk.Extension.llvm22](https://github.com/flathub/org.freedesktop.Sdk.Extension.llvm22) | Active Flathub extension targeting runtime 25.08. Structural reference for JSON manifest format, `build-extension` layout, and `enable.sh` pattern. |
| `org.freedesktop.Sdk.Extension.qt6/` | [4c0n/org.freedesktop.Sdk.Extension.qt6](https://github.com/4c0n/org.freedesktop.Sdk.Extension.qt6) | Stale Qt 6.3 extension (runtime 21.08, unmaintained since 2022). Reference for Qt-specific cmake-ninja build options and module ordering. |

## Files

| File | Source | Notes |
|---|---|---|
| `flatpak-kde6-sdk.container.yaml` | [Fedora src.fedoraproject.org](https://src.fedoraproject.org/flatpaks/kde6-sdk/blob/rawhide/f/container.yaml) | Fedora KDE6 SDK container definition. RPM-based (not source-build), used only as a module list reference for which qt6-* packages exist. |
