# pod5-tools

[![CI](https://github.com/sagrudd/pod5-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/sagrudd/pod5-tools/actions/workflows/ci.yml)
[![Documentation Status](https://readthedocs.org/projects/pod5-tools/badge/?version=latest)](https://pod5-tools.readthedocs.io/en/latest/?badge=latest)

`pod5-tools` is a planned Rust toolbox for auditing, discovering, describing,
and replaying Oxford Nanopore POD5 datasets.

The project is sponsored by Mnemosyne Biosciences, which develops enterprise
software for multi-tenant bioinformatics analyses on local infrastructure.
Mnemosyne Biosciences serves bioinformatics service providers and DNA sequencing
core facilities, with a particular focus on long-read DNA sequencing and the
additional epigenetic information that can be recovered from raw signal data.

## Purpose

POD5 is the main vehicle through which Oxford Nanopore Technologies presents
raw sequencing signal. Many workflows only need basecalled BAM or FASTQ output,
but the raw POD5 layer can be valuable for data maintenance, provenance,
reanalysis, and epigenetic signal-aware development. In practice, metadata can
be lost or distorted by control software, transfers, and downstream workflow
staging. `pod5-tools` is intended to make raw-data collections easier to trust,
describe, subdivide, and reuse.

The ambition is to provide a maintained and supported command-line and library
toolbox that can:

- verify POD5 data integrity before expensive analysis begins;
- enumerate files, reads, run metadata, and folder-level structure;
- identify what is present across local storage trees;
- expose temporal and structural subdivision primitives for workflow
  development;
- migrate the playback methods currently implemented in `../mnematikon` into a
  standalone, reusable project.

## Planned Interface

The project will expose both a binary and a Rust library. The binary will use
`clap` for command-line parsing, and all versioned releases will follow semantic
versioning.

Initial command concepts:

```text
pod5-tools find /path/to/search
pod5-tools fileinfo /path/to/file.pod5
pod5-tools verify /path/to/file.pod5
pod5-tools folderinfo /path/to/folder
pod5-tools subdivide write /path/to/folder --out /tmp/subdivided --files-per-chunk 4
pod5-tools playback plan --manifest playback_manifest.json --sample sample-a
pod5-tools playback emit --manifest playback_manifest.json --sample sample-a --speedup 5x
```

Candidate command groups:

- `find`: locate folders containing one or more POD5 files and report tabular
  paths, counts, and byte totals.
- `fileinfo`: report QC and provenance information for a single POD5 file,
  including flow cell ID, sequencing kit, read count, acquisition start time,
  signal duration, file size, schema/version information, and integrity status.
- `verify`: quickly confirm that a candidate file is actually POD5 by checking
  the extension and then validating content against the ONT POD5
  specification, including signature, layout, table, schema, and metadata
  expectations.
- `folderinfo`: aggregate `fileinfo`-style metrics across a directory tree,
  detecting mixed flow cells, interrupted runs, duplicated file names, and
  unexpected gaps in acquisition time.
- `playback`: reproduce sequencing-like file arrival from existing POD5 files,
  migrated from Mnematikon and exposed as both CLI and library behavior.
- `split` or `subdivide`: produce structural or temporal POD5 subsets suitable
  for test fixtures and workflow development.
- `manifest`: write stable JSON/TSV manifests for downstream systems, including
  read counts, source paths, checksums, timestamps, and optional sample labels.
- `compare`: compare two POD5 collections by metadata, read IDs, file sizes,
  acquisition windows, and integrity state.
- `sample`: extract representative subsets for regression tests without
  accidentally moving whole production datasets.

The upstream ONT `pod5` package already provides important general-purpose
operations such as view, inspect, merge, filter, subset, repack, schema update,
and FAST5 conversion. `pod5-tools` should complement that ecosystem by focusing
on operational inspection, storage-tree discovery, reproducible playback, and
workflow-oriented subdivision rather than replacing the official toolchain.

## Documentation

Sphinx documentation lives under `docs/source` and is intended for Read the
Docs. Public structs and library APIs should have Rust documentation comments
that can be carried into generated reference documentation as the codebase
matures.

The development roadmap is in `roadmap.md`, and the implementation slices are
tracked in `todo.md`.
