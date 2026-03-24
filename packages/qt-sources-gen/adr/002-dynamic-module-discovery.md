# ADR-002: Dynamic Module Discovery from md5sums.txt

**Status:** Decided  
**Date:** 2026-03-24  
**Relates to:** [ADR-001 (qt-sources-gen)](001-complete-refactoring.md)
(module structure), [ADR-001 (project)](../../adr/001-project-goals.md)
(CI-driven source verification)

## Context

The tool previously maintained a hardcoded `MODULES` constant in
`modules.rs` — a static list of 37 Qt submodule names. Every time Qt
upstream added or removed a submodule in a new release, this list had
to be updated manually. This created two problems:

1. **Silent omissions.** If a new submodule appeared in `md5sums.txt`
   but was not added to `MODULES`, the tool would silently skip it.
   The `sources` file would be missing that module's hash, and the
   Flatpak manifest would have no verified hash to consume. This
   already happened with `qtquickeffectmaker` and `qtwebview`, which
   were present in the Qt 6.11.0 release but absent from the hardcoded
   list.

2. **Stale entries.** If Qt removed a submodule, the hardcoded list
   would still attempt to download it, causing a 404 error and a
   failed run.

The tool already fetches and parses `md5sums.txt` on every run (for
the sentinel check and MD5 verification). The filenames of all
released archives are right there in the parsed data — there is no
need to duplicate that information in a static list.

## Decision

### 1. Derive the module list from md5sums.txt

Replace the hardcoded `MODULES` constant with `archive_filenames()`, a
function that extracts the list of archive filenames from the parsed
`md5sums.txt` entries. The function:

- Filters to files matching `*-everywhere-src-*.tar.xz`
- Excludes the `qt-everywhere-src` mega-bundle (not a submodule)
- Returns filenames in sorted order for deterministic output

### 2. Detect module-set mismatches

Add `modules_match()`, which compares the set of archive filenames in
the existing `sources` file against the set derived from `md5sums.txt`.
If they differ — modules added or removed upstream — the tool treats
this as a change that requires regeneration, even if the sentinel hash
is unchanged.

This closes a gap: previously, the sentinel hash was the only trigger
for regeneration. If Qt re-published `md5sums.txt` with the same
content hash but the `sources` file had been manually edited or was
generated from an older module list, the mismatch would go undetected.

### 3. Update verify.rs to iterate dynamically

`download_and_verify` now receives its module list from
`archive_filenames()` (called with the `expected_md5s` from the
sentinel) instead of iterating `MODULES`. A helper `module_name()`
extracts the module name from a filename for logging purposes.

## Consequences

- **Automatic coverage.** New Qt submodules are picked up on the next
  run with no code changes. The `sources` file and Flatpak manifest
  stay in sync with upstream.
- **Module-set drift detection.** A mismatch between the `sources` file
  and `md5sums.txt` triggers a full re-download, preventing stale or
  incomplete hash sets from persisting.
- **Sorted output.** Archive filenames are sorted lexicographically,
  making the `sources` file output deterministic regardless of
  `HashMap` iteration order. This differs from the previous behaviour
  where output order matched the hardcoded list order.
- **No manual maintenance.** The `MODULES` constant is gone. Adding
  or removing Qt submodules upstream requires zero changes to the tool.
