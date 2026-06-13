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

- [ ] Define a reader trait or adapter boundary for POD5 metadata access.
- [ ] Evaluate Rust-native Arrow access versus binding to the official POD5
  implementation.
- [ ] Add error types that separate path, format, schema, and integrity errors.
- [ ] Add mocked tests that allow command development before large fixtures are
  committed.

## Slice 6: `fileinfo`

- [ ] Implement file-level metadata extraction.
- [ ] Report flow cell ID, sequencing kit, read count, acquisition start,
  duration, file size, schema/version, and integrity status when available.
- [ ] Support TSV and JSON output.
- [ ] Add tests for success and failure paths.
- [ ] Document operational caveats and field definitions.

## Slice 7: `folderinfo`

- [ ] Aggregate `fileinfo` records across a folder.
- [ ] Detect mixed flow cells, mixed sequencing kits, duplicate names, suspicious
  temporal gaps, unreadable files, and partial/integrity failures.
- [ ] Provide machine-readable summary output.
- [ ] Add tests for mixed and clean folder scenarios.
- [ ] Document examples for sequencing core handoff checks.

## Slice 8: Manifest and Compare Planning

- [ ] Implement versioned manifest data structures.
- [ ] Add `manifest` command for JSON and TSV inventories.
- [ ] Add `compare` command for manifest-to-manifest and folder-to-folder
  comparisons.
- [ ] Define exit codes for automation use.
- [ ] Document manifest schema versioning.

## Slice 9: Subdivision Planning

- [ ] Implement `subdivide plan` for elapsed-time, file-count, read-count, and
  sample-label strategies.
- [ ] Keep planning read-only in this slice.
- [ ] Add deterministic tests for generated plans.
- [ ] Document how plans can feed workflow development.

## Slice 10: Playback Migration

- [ ] Extract Mnematikon playback concepts into standalone library types.
- [ ] Migrate speedup, cutoff, bucket scheduling, and manifest tests.
- [ ] Remove Mnematikon API/session assumptions from the core playback planner.
- [ ] Add `playback plan` and `playback emit` CLI surfaces.
- [ ] Document compatibility notes for existing Mnematikon behavior.

## Slice 11: Write-Capable Subdivision

- [ ] Implement source-preserving output writing for selected subdivision modes.
- [ ] Verify generated POD5 output integrity.
- [ ] Record provenance in sidecar manifests.
- [ ] Add tests or documented fixture generation steps.
- [ ] Document storage and performance caveats.

## Slice 12: Release Preparation

- [ ] Audit all public structs and functions for Rust documentation comments.
- [ ] Ensure README and Sphinx docs match implemented behavior.
- [ ] Confirm semantic version and changelog/release notes.
- [ ] Run full test and documentation build.
- [ ] Tag only after explicit release approval.
