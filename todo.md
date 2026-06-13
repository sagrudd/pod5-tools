# Todo

Work through this file from top to bottom. Each slice should be independently
buildable or documentable, tested where applicable, committed, and pushed.

## Slice 1: Repository Foundation

- [x] Expand `README.md` with sponsor context, project purpose, planned CLI, and
  ecosystem positioning.
- [x] Add `AGENTS.md` with Rust, clap, semantic versioning, documentation, and
  automation guidance.
- [x] Add `roadmap.md`.
- [x] Add `todo.md`.
- [x] Scaffold Sphinx documentation under `docs/source`.

## Slice 2: Buildable Rust Skeleton

- [x] Add `Cargo.toml` for package `pod5-tools` at version `0.1.0`.
- [x] Add `src/lib.rs` with documented public metadata structs for planned
  command outputs.
- [x] Add `src/main.rs` using `clap` with subcommand stubs.
- [x] Add baseline tests for CLI parsing and library types.
- [x] Document build, test, and CLI preview usage in Sphinx.

## Slice 3: Documentation Build and CI

- [x] Add Sphinx requirements and Read the Docs configuration.
- [x] Add local documentation build instructions.
- [x] Add GitHub Actions or equivalent CI for `cargo fmt`, `cargo clippy`,
  `cargo test`, and Sphinx build.
- [x] Add badges or status notes to the README only after CI exists.

## Slice 4: `find` Command

- [x] Implement recursive POD5 folder discovery.
- [x] Emit TSV by default and JSON with an explicit option.
- [x] Include path, POD5 file count, byte total, newest mtime, and oldest mtime.
- [x] Add tests using temporary directory fixtures.
- [x] Document output fields and examples.

## Slice 5: POD5 Reader Abstraction

- [x] Define a reader trait or adapter boundary for POD5 metadata access.
- [x] Evaluate Rust-native Arrow access versus binding to the official POD5
  implementation.
- [x] Add error types that separate path, format, schema, and integrity errors.
- [x] Add mocked tests that allow command development before large fixtures are
  committed.

## Slice 6: `fileinfo`

- [x] Implement file-level metadata extraction.
- [x] Report flow cell ID, sequencing kit, read count, acquisition start,
  duration, file size, schema/version, and integrity status when available.
- [x] Support TSV and JSON output.
- [x] Add tests for success and failure paths.
- [x] Document operational caveats and field definitions.

## Slice 7: `verify`

- [x] Add a `verify` subcommand to the CLI contract.
- [x] Check the input path exists, is a file, and has a `.pod5` extension.
- [x] Add fast content checks for the fixed ONT POD5 signature at the start and
  end of the file.
- [x] Define deeper specification checks for combined-file layout, footer magic,
  footer length, required Reads/Signal/Run Info tables, and required schema
  metadata.
- [x] Emit TSV by default and JSON with an explicit option.
- [x] Add tests for extension, signature, truncated-file, and success/failure
  reporting paths.
- [x] Document how `verify` maps failures to ONT POD5 specification concepts.

## Slice 8: `folderinfo`

- [x] Aggregate `fileinfo` records across a folder.
- [x] Detect mixed flow cells, mixed sequencing kits, duplicate names, suspicious
  temporal gaps, unreadable files, and partial/integrity failures.
- [x] Provide machine-readable summary output.
- [x] Add tests for mixed and clean folder scenarios.
- [x] Document examples for sequencing core handoff checks.

## Slice 9: Manifest and Compare Planning

- [x] Implement versioned manifest data structures.
- [x] Add `manifest` command for JSON and TSV inventories.
- [x] Add `compare` command for manifest-to-manifest and folder-to-folder
  comparisons.
- [x] Define exit codes for automation use.
- [x] Document manifest schema versioning.

## Slice 10: Subdivision Planning

- [x] Implement `subdivide plan` for elapsed-time, file-count, read-count, and
  sample-label strategies.
- [x] Keep planning read-only in this slice.
- [x] Add deterministic tests for generated plans.
- [x] Document how plans can feed workflow development.

## Slice 11: Playback Migration

- [x] Extract Mnematikon playback concepts into standalone library types.
- [x] Migrate speedup, cutoff, bucket scheduling, and manifest tests.
- [x] Remove Mnematikon API/session assumptions from the core playback planner.
- [x] Add `playback plan` and `playback emit` CLI surfaces.
- [x] Document compatibility notes for existing Mnematikon behavior.

## Slice 12: Write-Capable Subdivision

- [x] Implement source-preserving output writing for selected subdivision modes.
- [x] Verify generated POD5 output integrity.
- [x] Record provenance in sidecar manifests.
- [x] Add tests or documented fixture generation steps.
- [x] Document storage and performance caveats.

## Slice 13: Release Preparation

- [x] Audit all public structs and functions for Rust documentation comments.
- [x] Ensure README and Sphinx docs match implemented behavior.
- [x] Confirm semantic version and changelog/release notes.
- [x] Run full test and documentation build.
- [x] Defer tagging until explicit release approval.
