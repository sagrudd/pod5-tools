# Changelog

All notable changes to `pod5-tools` will be documented here.

The project follows semantic versioning. Until the first stable release,
versions use the `0.y.z` series while command and library contracts are still
being established.

## 0.1.0 - Unreleased

Initial development release candidate.

### Added

- Rust binary and library crate using `clap` for command-line parsing.
- `find` for recursive POD5 directory discovery with TSV and JSON output.
- `verify` for fast POD5 extension and signature checks with reserved deeper
  specification checks.
- Filesystem-backed `fileinfo` and `folderinfo` summaries.
- Versioned manifest output and basic collection comparison.
- Read-only `subdivide plan` plus whole-file `subdivide write` materialization
  with provenance sidecars.
- Playback manifest, schedule, speedup, cutoff, and dry-run emission planning
  primitives migrated from Mnematikon without API/session coupling.
- Sphinx documentation, Read the Docs configuration, CI, and automation-facing
  development guidance.
- Mocked-reader test coverage for folder-level mixed flow-cell and sequencing
  kit aggregation.

### Known Limitations

- POD5-internal metadata parsing is not yet connected to a concrete backend.
- Deep POD5 integrity checks beyond the fixed signatures are reserved but not
  implemented.
- `subdivide write` currently copies whole POD5 files rather than rewriting
  read-level or elapsed-time subsets.
- No release tag has been created; tagging requires explicit release approval.
