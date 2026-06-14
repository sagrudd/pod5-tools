# Changelog

## 0.1.0 - Unreleased

Initial development release candidate.

### Added

- Rust binary and library crate using `clap` for command-line parsing.
- `find`, `verify`, `fileinfo`, `folderinfo`, `manifest`, `compare`,
  `subdivide plan`, `subdivide write`, `playback plan`, and `playback emit`
  command surfaces.
- Dockerized official `pod5.Reader` backend for `fileinfo` and `folderinfo`
  metadata, including flow cell ID, sequencing kit, read count, acquisition
  start time, duration, file version, and parser integrity status.
- Versioned manifest, subdivision, playback, and provenance output contracts.
- Sphinx documentation, Read the Docs configuration, CI, and automation-facing
  development guidance.

### Known Limitations

- Deep POD5 integrity checks beyond the fixed signatures are reserved but not
  implemented.
- `fileinfo` and `folderinfo` require a Docker-compatible runtime and a backend
  image containing Oxford Nanopore's `pod5` package.
- `subdivide write` currently copies whole POD5 files rather than rewriting
  read-level or elapsed-time subsets.
- No release tag has been created; tagging requires explicit release approval.
