# Changelog

## 0.1.0 - Unreleased

Initial development release candidate.

### Added

- Rust binary and library crate using `clap` for command-line parsing.
- `find`, `verify`, `fileinfo`, `folderinfo`, `manifest`, `compare`,
  `subdivide plan`, `subdivide write`, `playback plan`, and `playback emit`
  command surfaces.
- Versioned manifest, subdivision, playback, and provenance output contracts.
- Sphinx documentation, Read the Docs configuration, CI, and automation-facing
  development guidance.

### Known Limitations

- POD5-internal metadata parsing is not yet connected to a concrete backend.
- Deep POD5 integrity checks beyond the fixed signatures are reserved but not
  implemented.
- `subdivide write` currently copies whole POD5 files rather than rewriting
  read-level or elapsed-time subsets.
- No release tag has been created; tagging requires explicit release approval.
