# Roadmap

This roadmap defines the early development path for `pod5-tools`. It is written
for a Rust implementation with a `clap` CLI, reusable library APIs, semantic
versioning, and Sphinx/Read the Docs documentation.

## Milestone 0: Project Foundation

Goal: establish the repository shape, documentation system, command vocabulary,
and contribution rules before moving code.

Deliverables:

- Repository guidance in `AGENTS.md`.
- Expanded README explaining sponsor context, project purpose, initial command
  set, and relationship to the ONT POD5 ecosystem.
- Sphinx documentation scaffold under `docs/source`.
- Roadmap and todo files that can drive automated incremental development.
- Initial package plan for a Rust binary plus library crate.

Acceptance criteria:

- Documentation builds with Sphinx.
- The README and docs describe the intended CLI surface without claiming
  implemented behavior.
- The first implementation slice is small enough to complete in one automation
  pass.

## Milestone 1: Rust Workspace and CLI Skeleton

Goal: create the first buildable Rust package without implementing POD5 parsing.

Deliverables:

- `Cargo.toml` with semantic version `0.1.0`.
- Library crate for shared types and command planning.
- Binary crate using `clap`.
- Subcommand stubs for `find`, `verify`, `fileinfo`, `folderinfo`, `playback`,
  `manifest`, `subdivide`, and `compare`.
- CI or local test instructions for `cargo fmt`, `cargo clippy`, `cargo test`,
  and Sphinx docs.

Acceptance criteria:

- `cargo test` passes.
- `pod5-tools --help` and each subcommand help render correctly.
- Public structs introduced in the library have Rust doc comments.
- Sphinx command reference documents the CLI skeleton as planned or preview
  functionality.

## Milestone 2: Discovery and Metadata Inspection

Goal: deliver read-only operational inspection commands that are useful before
any POD5 rewriting or playback work.

Deliverables:

- `find` recursively identifies directories containing `.pod5` files and emits
  TSV and JSON.
- `verify` checks one file by extension and by ONT POD5 content expectations,
  including fixed leading and trailing signatures, combined-file layout sanity,
  required table presence, and schema metadata needed to treat the file as
  specification-adherent POD5.
- `fileinfo` reports file path, size, schema/version where available, read
  count, flow cell ID, sequencing kit, start time, duration, and integrity
  status.
- `folderinfo` aggregates `fileinfo` results across one folder and identifies
  mixed metadata, acquisition windows, byte totals, and suspicious gaps.
- Integration tests with small fixtures or mocked reader traits.

Acceptance criteria:

- Commands are read-only.
- Output contracts are documented and tested.
- Errors distinguish unreadable paths, non-POD5 files, parse failures, and
  integrity failures.
- `verify` has a fast path for extension/signature failures and a deeper path
  for schema/layout adherence once the concrete reader backend is connected.

## Milestone 3: Manifests, Comparison, and Subdivision Planning

Goal: provide stable descriptions and planning primitives for workflow
developers.

Deliverables:

- `manifest` writes JSON and TSV inventories with checksums, file metadata,
  optional sample labels, and acquisition windows.
- `compare` reports added, missing, duplicated, or metadata-drifted POD5 files
  between collections.
- `subdivide plan` creates temporal or structural subdivision plans without
  writing new POD5 output.
- Documentation for downstream workflow and LIMS integration.

Acceptance criteria:

- Manifest formats are versioned.
- Comparison exits with useful status codes for automation.
- Subdivision plans are deterministic and test-covered.

## Milestone 4: Playback Migration from Mnematikon

Goal: move the POD5 playback concept out of `../mnematikon` into `pod5-tools`
as standalone library and binary behavior.

Source material:

- `../mnematikon/crates/mnematikon-cli/src/playback.rs`
- related Mnematikon benchmarking and playback tests
- Mnematikon API-specific behavior that must be separated from generic POD5
  playback planning

Deliverables:

- Library types for playback manifests, batch schedules, speedup parsing,
  cutoff parsing, and temporal bucket planning.
- CLI command for generating playback batches and optionally emitting them over
  wall-clock time.
- Mnematikon integration reduced to a consumer of `pod5-tools` behavior or
  documented as future downstream work.
- Tests migrated or rewritten to cover the standalone contracts.

Acceptance criteria:

- Playback planning does not depend on Mnematikon storage or API sessions.
- Wall-clock waiting is isolated behind testable abstractions.
- Existing Mnematikon playback semantics are either preserved or explicitly
  changed in the documentation.

## Milestone 5: POD5 Writing and Advanced Workflows

Goal: add write-capable operations once read-only and planning behavior is
stable.

Candidate deliverables:

- `subdivide write` for temporal or read-count based POD5 subsets.
- Sampling fixtures for test-data generation.
- Integrity verification after writing.
- Optional signal export or plotting support if it complements, rather than
  duplicates, GUI tools such as pod5Viewer.

Acceptance criteria:

- Write operations never modify source data in place.
- Output provenance records source files, selection criteria, and tool version.
- Large-data behavior is benchmarked before release.

## Ecosystem Positioning

The official ONT `pod5` package already provides schema update, repack, merge,
filter, subset, conversion, and high-speed tabular read views. pod5Viewer
provides GUI inspection, filtering, plotting, and export of reads and signal
measurements. `pod5-tools` should sit beside these tools as an operational
toolbox for facilities: discovery across storage trees, integrity and metadata
audits, folder-level summaries, reproducible manifests, comparison, subdivision
planning, and playback for workflow development.
