# AGENTS.md

This repository contains `pod5-tools`, a Rust command-line and library toolbox
for working with Oxford Nanopore POD5 datasets.

## Project Direction

- Implement the application in Rust.
- Use `clap` for the command-line interface.
- Expose both a binary crate and reusable library APIs.
- Follow semantic versioning for all releases.
- Keep user-facing documentation in Sphinx/Read the Docs format.
- Maintain Rust documentation comments for public structs, enums, traits, and
  functions as code is introduced.
- Treat the current Mnematikon playback implementation in `../mnematikon` as
  source material for migration, not as an API boundary to preserve blindly.

## Sponsor Context

Mnemosyne Biosciences sponsors this work. The project should serve
bioinformatics service providers and sequencing core facilities that need
reliable local-infrastructure tooling around long-read sequencing data.

The core product value is operational trust in raw POD5 data: discovery,
integrity checks, provenance, metadata recovery, temporal subdivision, and
development playback.

## Development Discipline

- Before editing, inspect `git status --short` and avoid overwriting unrelated
  user work.
- Keep changes scoped to the current development slice in `todo.md`.
- Prefer small, reviewable commits with clear messages.
- After each prompt that changes files, review the diff, commit, and push.
- Do not commit secrets, credentials, generated caches, large sequencing data,
  local virtual environments, or machine-specific paths.
- Add or update focused tests with each code-bearing slice.
- Update Sphinx documentation in the same change set as user-facing behavior.

## Versioning

Use semantic versioning:

- Patch versions for bug fixes, documentation corrections, metadata updates,
  and backwards-compatible implementation refinements.
- Minor versions for new commands, new library APIs, new output contracts, or
  backwards-compatible behavior that materially changes how operators use the
  tool.
- Major versions only after explicit discussion and approval from a human
  reviewer or the project owner.

Until the first release, use `0.y.z` versions and treat public contracts as
stabilizing but still subject to documented change.

## Documentation Expectations

- Sphinx sources live in `docs/source`.
- Keep the documentation practical: purpose, inputs, outputs, examples,
  caveats, and expected failure modes.
- When adding public Rust types, include concise doc comments that describe the
  contract and units of measurement.
- Prefer stable table-oriented examples for CLI output so sequencing facilities
  can integrate the tool with shell, workflow, and LIMS contexts.

## Command Design Principles

- Make defaults safe for production sequencing data.
- Read from source data by default; require explicit output paths for generated
  files.
- Prefer deterministic tabular or JSON outputs over human-only prose.
- Preserve source metadata and record provenance whenever writing manifests or
  derived POD5 files.
- Keep upstream ONT `pod5` behavior in mind and avoid cloning commands already
  well served by the official package unless `pod5-tools` adds operational
  value.

## Initial Command Scope

Planned commands include:

- `find`: locate directories containing POD5 files.
- `verify`: confirm a candidate file has a POD5 extension and specification-
  adherent POD5 content.
- `fileinfo`: inspect one POD5 file for QC, provenance, and integrity.
- `folderinfo`: aggregate file information across a POD5 folder or run tree.
- `playback`: emit existing POD5 data over time to simulate acquisition.
- `manifest`: write stable JSON/TSV descriptions for downstream tools.
- `subdivide`: create temporal or structural subsets for development and tests.
- `compare`: compare POD5 collections for completeness and drift.

## Automation Guidance

When an automation is working through `todo.md`:

- Read `AGENTS.md`, `roadmap.md`, `todo.md`, and the relevant Sphinx page before
  editing.
- Select the first incomplete development slice.
- Complete the smallest useful increment that can be built, tested, documented,
  committed, and pushed.
- Mark completed todo items as done in `todo.md`.
- Stop and mark the automation complete once every todo item is done.

## Kanon identity contract

- This repository's Mnemosyne product or component identity must be registered
  in the authoritative Kanon registry at
  `https://github.com/sagrudd/kanon`.
- Changes to the stable identifier, display name, repository location, crate,
  package, container, binary, product-manifest or schema coordinates, supported
  host modes, dependencies, compatibility, lifecycle, aliases, deprecation, or
  replacement must include the corresponding Kanon change in the same delivery
  transaction or an explicitly linked Kanon pull request.
- Before a release, verify that this repository's Kanon identity and dependency
  declarations match the release artefacts. Once Kanon channels and locksets
  are operational, releases and maintained product branches must use the
  applicable supported channel and pin the resolved lockset identifier and
  digest.
- Do not invent, rename, or reuse Mnemosyne product identifiers locally. Do not
  treat registration in Kanon as proof that a component is installed,
  entitled, healthy, or supported by every host profile.
- If live Kanon services are unavailable, use a verified pinned Kanon snapshot
  or lockset. Do not bypass identity or compatibility validation to make a
  release proceed.
