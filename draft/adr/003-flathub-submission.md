# ADR-003: Flathub Submission Strategy

**Status:** Proposed  
**Date:** 2026-03-24  
**Relates to:** [ADR-001 (draft)](001-manifest-design.md) (manifest
structure), [ADR-002 (draft)](002-manifest-linting.md) (manifest
linting), [ADR-003 (project)](../../adr/003-manifest-ci.md) (CI
strategy)

## Context

The Qt 6.11 SDK extension is ready to be submitted to Flathub. The
Flathub CONTRIBUTING.md and the
[requirements](https://docs.flathub.org/docs/for-app-authors/requirements)
and
[submission](https://docs.flathub.org/docs/for-app-authors/submission)
documentation define the process, policies, and required files for new
submissions.

### Submission mechanism

All new submissions to Flathub are made as pull requests against the
**`new-pr`** branch of `flathub/flathub`. The PR must never target
`master`. The workflow is:

1. Fork `flathub/flathub` (with all branches, not just `master`).
2. Create a branch from `new-pr`.
3. Add the required files, commit, push.
4. Open the PR against the `new-pr` base branch.

A fork at `git@github.com:da2ce7/flathub.git` has been added as a
submodule at `draft/flathub`, tracking the `new-pr` branch. This
submodule serves as the staging area where the submission files are
assembled and tested before pushing the PR.

### Applicable Flathub requirements for SDK extensions

SDK extensions have a distinct acceptance profile compared to
applications. Key policies and how they apply:

| Requirement | Applicability | Status |
|---|---|---|
| **Extensions or BaseApps** — must have "a clear use case defined and prospective users" | Applies. Qt 6.11 SDK extension enables Qt apps on freedesktop runtime without bundling KDE Frameworks. | Addressed in metainfo description |
| **Extension ID** — must prefix with the extension point ID from the runtime | Applies. `org.freedesktop.Sdk.Extension.qt611` extends `org.freedesktop.Sdk`. Exempt from domain control rules. | Compliant |
| **Manifest at top level** — named after the ID with `.json`/`.yml`/`.yaml` | Applies. `org.freedesktop.Sdk.Extension.qt611.json` | Compliant |
| **flathub.json** — architecture constraints and build flags | Applies. Must include `only-arches`, `skip-appstream-check`, `skip-icons-check` (SDK extensions have no desktop icons or strict appstream requirements). | Compliant |
| **Metainfo file (optional for extensions)** — but recommended to appear on Flathub website | Optional. We provide `org.freedesktop.Sdk.Extension.qt611.metainfo.xml` for visibility. | Included |
| **No network access during build** — all sources must have publicly accessible URLs | Applies. All 38 Qt submodule tarballs are fetched from `download.qt.io` with SHA-512 hashes. | Compliant |
| **Building from source** — all components built from source code | Applies. All modules built with `cmake-ninja` from source archives. | Compliant |
| **License** — must be correctly specified in metainfo and allow redistribution | Applies. Qt is `LGPL-3.0-only AND GPL-3.0-only AND GPL-2.0-only WITH Qt-GPL-exception-1.0`. | Declared in metainfo |
| **License of contents in Flathub repository** — permissive license recommended for manifests | Applies. The submission repo should include a license file (e.g. MIT) for the manifest and build scripts. | To be added |
| **Installing license files** — license files for each module installed to `$FLATPAK_DEST/share/licenses/$FLATPAK_ID` | Applies. Qt source archives include license files; flatpak-builder auto-installs common license filenames. | Verify during build |
| **Stable releases** — only stable software in the stable repo | Applies. Qt 6.11.0 is a stable release (2026-03-23). | Compliant |
| **Linter** — `flatpak-builder-lint manifest` must pass | Applies. Validated in [ADR-002](002-manifest-linting.md). | Passing |
| **End-of-life dependency policy** — must not use EOL runtimes/extensions | Applies. Using `org.freedesktop.Sdk` 25.08 and `org.freedesktop.Sdk.Extension.llvm22`. | Current |
| **Generative AI policy** — PRs must not be generated/automated by AI tools; code must have meaningful human review | Applies. The submission PR must be authored and reviewed by a human. AI-assisted code must be moderated and justified. | Acknowledged |

### Required files in the submission PR

Based on the Flathub requirements, the submission branch must contain:

| File | Description |
|---|---|
| `org.freedesktop.Sdk.Extension.qt611.json` | The manifest (top-level, named after the ID) |
| `flathub.json` | Architecture and build flags (`only-arches`, `skip-appstream-check`, `skip-icons-check`) |
| `org.freedesktop.Sdk.Extension.qt611.metainfo.xml` | Metainfo for Flathub website listing (optional for extensions, but recommended) |

No dependency manifests are needed since all Qt sources are declared
directly in the main manifest as `type: archive` entries.

## Decision

### 1. Stage the submission in the `draft/flathub` submodule

Use the `draft/flathub` submodule (tracking the `new-pr` branch of the
`da2ce7/flathub` fork) as the staging area. Once the manifest sources
are finalized and hashes populated, copy the required files from
`draft/` into the submodule:

```
draft/flathub/
├── flathub.json
├── org.freedesktop.Sdk.Extension.qt611.json
└── org.freedesktop.Sdk.Extension.qt611.metainfo.xml
```

### 2. Create a submission branch from `new-pr`

Inside the submodule, create a topic branch from `new-pr` for the
submission PR:

```sh
cd draft/flathub
git checkout -b add-qt611-extension new-pr
```

The PR title should follow the convention: **"Add
org.freedesktop.Sdk.Extension.qt611"**.

### 3. Pre-submission checklist

Before pushing, verify:

- [ ] `flatpak-builder-lint manifest` passes on the manifest
- [ ] All source archive hashes are populated (no `FIXME` placeholders)
- [ ] `flathub.json` contains `only-arches`, `skip-appstream-check`,
      and `skip-icons-check`
- [ ] Metainfo XML validates with `appstreamcli validate --no-net`
- [ ] The PR targets the `new-pr` base branch (never `master`)
- [ ] No source code or build artifacts are included in the submission
- [ ] AI-generated content policy is respected: the PR is
      human-authored with meaningful review

### 4. Open the PR against `flathub/flathub:new-pr`

Push the submission branch to the fork and open the PR via the GitHub
web interface against `flathub/flathub`'s `new-pr` branch. After
review, a reviewer will comment `bot, build` to trigger a test build
on Flathub's infrastructure.

### 5. Post-approval

Once approved, the submission is merged into a new repository under the
Flathub GitHub organisation (`flathub/org.freedesktop.Sdk.Extension.qt611`).
The submitter receives write access. Future updates go directly to that
repository — updates never go through the submission process again.

## Consequences

- The `draft/flathub` submodule provides a clean staging area that
  mirrors the exact file layout the Flathub PR expects, without
  polluting the main repository with submission logistics.
- The submodule pin in the parent repo tracks which commit was
  submitted, providing an audit trail.
- The pre-submission checklist consolidates Flathub requirements into
  actionable verification steps specific to an SDK extension, reducing
  the risk of review rejection.
- After the extension is accepted on Flathub, the `draft/flathub`
  submodule can be removed — ongoing maintenance happens in the
  Flathub-hosted repository directly.
