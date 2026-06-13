Command Plan
============

The command-line interface is implemented in Rust using ``clap``. ``find``, the
filesystem-backed part of ``fileinfo``, fast ``verify`` extension/signature
checks, filesystem-backed ``folderinfo``, versioned ``manifest`` output, basic
``compare``, read-only ``subdivide plan``, and read-only playback planning
surfaces are implemented. Deeper POD5 metadata parsing and read-level POD5
rewriting remain backend-dependent future work.

Preview help:

.. code-block:: sh

   cargo run -- --help
   cargo run -- find --help

``find``
--------

Find directories containing one or more POD5 files.

Usage:

.. code-block:: sh

   cargo run -- find /path/to/search
   cargo run -- find /path/to/search --format json

The command recursively walks the search root and reports one row for each
directory that directly contains one or more files with a ``.pod5`` extension.
The extension match is case-insensitive. Source files are not modified.

TSV is emitted by default. JSON is available with ``--format json``.

Output fields:

* directory path;
* POD5 file count;
* total bytes;
* oldest modification time;
* newest modification time.

Example TSV output:

.. code-block:: text

   path	pod5_file_count	total_bytes	oldest_modified_utc	newest_modified_utc
   /data/run/pod5	24	781246272	2026-06-13T09:12:30+00:00	2026-06-13T10:45:03+00:00

``fileinfo``
------------

Inspect a single POD5 file.

Usage:

.. code-block:: sh

   cargo run -- fileinfo /path/to/file.pod5
   cargo run -- fileinfo /path/to/file.pod5 --format json

Current behavior validates that the input exists, is a file, and has a
``.pod5`` extension. It reports file size and emits the planned metadata fields.
Until a concrete POD5 reader backend is connected, POD5-internal fields are
empty or ``null`` and integrity is reported as unavailable.

TSV is emitted by default. JSON is available with ``--format json``.

Output fields:

* path and file size;
* flow cell ID;
* sequencing kit;
* read count;
* acquisition start date and time;
* duration of data;
* POD5 schema/version information;
* integrity status.

Operational caveats:

* ``fileinfo`` does not yet parse POD5 internals.
* ``fileinfo`` does not yet prove that the file is intact.
* missing files, directories, and non-POD5 files fail before output is emitted.

Example TSV output:

.. code-block:: text

   path	size_bytes	flow_cell_id	sequencing_kit	read_count	acquisition_start_utc	duration_seconds	pod5_version	integrity_status	integrity_reason
   /data/run/pod5/reads.pod5	1048576								unavailable	POD5 parser backend not configured; only filesystem metadata was inspected

``verify``
----------

Verify that a candidate file is actually POD5.

Usage:

.. code-block:: sh

   cargo run -- verify /path/to/file.pod5
   cargo run -- verify /path/to/file.pod5 --format json

Current behavior:

* check that the path exists and is a file;
* check that the extension is ``.pod5``;
* check the ONT POD5 fixed signature at both the beginning and end of the file.

Reserved checks:

* combined-file layout markers, footer magic, footer length, and padding;
* required Reads, Signal, and Run Info table presence;
* table schema metadata including POD5 version, writer software, and file
  identifier consistency.

Until a concrete POD5 parser backend is connected, reserved layout/table/schema
checks are reported as ``not_checked``. If extension and signature checks pass
but reserved checks remain, the overall status is ``incomplete`` rather than
``passed``. Any implemented check failure makes the overall status ``failed``.

Output fields:

* path;
* file size;
* overall status;
* check name;
* check category;
* check status;
* detail.

Example TSV output:

.. code-block:: text

   path	size_bytes	overall_status	check	category	status	detail
   /data/run/pod5/reads.pod5	1048576	incomplete	leading_signature	signature	passed	leading signature matches ONT POD5 signature

``folderinfo``
--------------

Aggregate file-level metadata across a folder or run tree.

Usage:

.. code-block:: sh

   cargo run -- folderinfo /path/to/folder
   cargo run -- folderinfo /path/to/folder --format json

Current behavior recursively finds ``.pod5`` files, aggregates the current
``fileinfo`` fields, and runs the implemented ``verify`` checks for each file.

TSV is emitted by default. JSON is available with ``--format json``.

Output fields:

* folder path;
* POD5 file count;
* total bytes;
* total reads when available;
* distinct flow cell IDs when available;
* distinct sequencing kits when available;
* acquisition start and end times when available;
* aggregate integrity status;
* failed file count;
* verification failed count;
* duplicate file names;
* warnings.

Operational caveats:

* mixed flow cell, mixed sequencing kit, and temporal-gap checks depend on POD5
  metadata that is not yet available from the filesystem-only reader;
* duplicate file-name detection and fast verification failure detection are
  active now;
* deep integrity still requires the future POD5 parser backend.

Example TSV output:

.. code-block:: text

   path	pod5_file_count	total_bytes	total_reads	flow_cell_ids	sequencing_kits	acquisition_start_utc	acquisition_end_utc	integrity_status	integrity_reason	failed_file_count	verification_failed_count	duplicate_file_names	warnings
   /data/run/pod5	24	781246272						unavailable	deep POD5 integrity requires the parser backend	0	0		flow cell metadata unavailable with current POD5 reader backend

``manifest``
------------

Write stable JSON or TSV inventories for downstream tools, workflow engines, or
LIMS-style systems.

Usage:

.. code-block:: sh

   cargo run -- manifest /path/to/folder
   cargo run -- manifest /path/to/folder --format json
   cargo run -- manifest /path/to/folder --format json --output manifest.json

Current behavior accepts a POD5 file or a folder tree. It writes schema version
``1`` inventories with one row per POD5 file.

Schema version 1 fields:

* ``schema_version``;
* source path;
* relative path;
* file path;
* size in bytes;
* fast verification status;
* number of failed implemented verification checks.

TSV is emitted by default. JSON is available with ``--format json``. When
``--output`` is provided, the rendered manifest is written to that file and the
command prints the output path.

Example TSV output:

.. code-block:: text

   schema_version	source	relative_path	path	size_bytes	verification_status	verification_failed_checks
   1	/data/run/pod5	reads.pod5	/data/run/pod5/reads.pod5	1048576	incomplete	0

``compare``
-----------

Compare two POD5 collections or manifests and report missing files, added files,
duplicates, metadata drift, and integrity changes.

Usage:

.. code-block:: sh

   cargo run -- compare /path/to/left /path/to/right
   cargo run -- compare left-manifest.json right-manifest.json --format json

Current behavior compares two folders, two manifest JSON files, or one of each.
The comparison key is manifest-relative path. Differences include files missing
from either side and entries with changed size or fast verification status.

Output statuses:

* ``match`` means no manifest-level differences were found;
* ``different`` means one or more missing or changed entries were found.

Automation exit-code convention:

* current CLI execution exits non-zero for command/runtime errors;
* comparison differences are represented in output as ``different`` for this
  slice;
* a later CLI layer should map ``different`` to a dedicated non-zero exit code
  when the command runner can return structured process statuses.

Example TSV output:

.. code-block:: text

   status	kind	relative_path	left_size_bytes	right_size_bytes	left_verification_status	right_verification_status
   different	missing_from_right	reads-a.pod5				

``subdivide``
-------------

Plan or write temporal and structural POD5 subdivisions for development and
test fixtures. Planning is read-only. Writing currently materializes plans by
copying whole POD5 files into chunk directories.

Usage:

.. code-block:: sh

   cargo run -- subdivide plan /path/to/folder
   cargo run -- subdivide plan /path/to/folder --files-per-chunk 4
   cargo run -- subdivide plan /path/to/folder --strategy sample-label
   cargo run -- subdivide plan /path/to/folder --strategy elapsed-time --seconds-per-chunk 900
   cargo run -- subdivide plan /path/to/folder --format json --output plan.json
   cargo run -- subdivide write /path/to/folder --out /tmp/subdivided --files-per-chunk 4

Current behavior accepts a POD5 file, folder tree, or manifest JSON file. It
builds a schema version ``1`` subdivision plan from the versioned manifest
contract. ``subdivide write`` uses the same planning contract, creates one
folder per chunk, copies source POD5 files without modifying them, verifies each
copied file with implemented checks, and writes ``subdivide_provenance.json`` in
the output directory.

Strategies:

* ``file-count`` groups manifest entries into deterministic chunks containing
  at most ``--files-per-chunk`` files. This is the default strategy.
* ``sample-label`` groups files by the first component of each
  manifest-relative path, which is useful for run trees arranged as
  ``sample/file.pod5``.
* ``elapsed-time`` records the requested ``--seconds-per-chunk`` target but
  emits one placeholder chunk until acquisition timestamps are available from
  the POD5 reader backend.
* ``read-count`` records the requested ``--reads-per-chunk`` target but emits
  one placeholder chunk until read counts are available from the POD5 reader
  backend.

Schema version 1 fields:

* ``schema_version``;
* source path;
* strategy;
* target;
* chunk index and label;
* relative paths assigned to the chunk;
* file count;
* byte total;
* read count when available;
* warnings.

TSV is emitted by default. JSON is available with ``--format json``. When
``--output`` is provided, the rendered plan is written to that file and the
command prints the output path.

``subdivide write`` TSV fields:

* source path;
* output directory;
* strategy;
* chunk index and label;
* chunk output directory;
* relative path;
* source path;
* destination path;
* copied size in bytes;
* verification status after copying;
* provenance sidecar path.

Storage and performance caveats:

* writing copies whole POD5 files; it does not yet rewrite POD5 contents by
  read, signal interval, or elapsed-time window;
* output storage can temporarily approach the full size of selected input
  files;
* existing output files with the same relative paths are overwritten by the
  copy operation;
* deep POD5 validation still depends on the future parser backend, so copied
  files currently receive the same fast extension/signature verification as
  ``verify``.

Example TSV output:

.. code-block:: text

   schema_version	source	strategy	target	chunk_index	chunk_label	file_count	total_bytes	read_count	relative_paths	warnings
   1	/data/run/pod5	file-count	2 file(s) per chunk	1	chunk-0001	2	2097152		reads-a.pod5,reads-b.pod5	

Example write TSV output:

.. code-block:: text

   source	output_dir	strategy	chunk_index	chunk_label	chunk_output_dir	relative_path	source_path	destination_path	size_bytes	verification_status	provenance_path
   /data/run/pod5	/tmp/subdivided	file-count	1	chunk-0001	/tmp/subdivided/0001-chunk-0001	reads-a.pod5	/data/run/pod5/reads-a.pod5	/tmp/subdivided/0001-chunk-0001/reads-a.pod5	1048576	incomplete	/tmp/subdivided/subdivide_provenance.json

``playback``
------------

Replay an existing POD5 collection in sequencing-like order. This behavior will
be migrated from the Mnematikon implementation into standalone library and CLI
contracts.

Usage:

.. code-block:: sh

   cargo run -- playback plan --manifest playback_manifest.json --sample sample-a
   cargo run -- playback plan --manifest a.json --sample sample-a --manifest b.json --sample sample-b
   cargo run -- playback emit --manifest playback_manifest.json --sample sample-a --speedup 5x
   cargo run -- playback plan --manifest playback_manifest.json --sample sample-a --format json --output plan.json

Current behavior:

* ``playback plan`` loads one or more playback manifest JSON files and reports
  their per-sample batches plus the merged sequencing-time schedule;
* ``playback emit`` reports a deterministic dry-run emission order and the
  wall-clock waits implied by ``--speedup``;
* migrated planning helpers are independent of Mnematikon API sessions,
  flowcell adoption, biosample creation, and upload behavior.

Compatibility notes:

* speedup values follow the existing Mnematikon convention, accepting values
  such as ``1``, ``1x``, ``2x``, and ``5X``;
* duration values currently accept seconds and minutes, for example ``300s``
  and ``5m``;
* cutoff values accept duration syntax as well as ``all``, ``full``, or
  ``none`` for all reads;
* batch emission time is calculated as ``bucket_start_seconds +
  tempo_seconds`` and schedules are merged across sample streams.

``playback plan`` TSV fields:

* sample label;
* input path, inferred from the manifest path unless ``--input`` is provided;
* manifest path;
* merged schedule seconds;
* batch index;
* batch path;
* bucket start seconds;
* emit seconds;
* read count;
* source POD5 count;
* minimum and maximum elapsed seconds when available.

``playback emit`` TSV fields:

* speedup;
* emission round index;
* sample label;
* batch index;
* batch path;
* emit seconds;
* source sequencing wait seconds;
* wall-clock wait seconds;
* read count.

Example dry-run TSV output:

.. code-block:: text

   speedup	round_index	sample	batch_index	batch_path	emit_seconds	sequence_wait_seconds	wall_wait_seconds	read_count
   5x	1	sample-a	1	/playback/a/batch-001.pod5	90	90	18	1
