Command Plan
============

The command-line interface is implemented in Rust using ``clap``. ``find``, the
filesystem-backed part of ``fileinfo``, fast ``verify`` extension/signature
checks, filesystem-backed ``folderinfo``, versioned ``manifest`` output, and
basic ``compare`` are implemented. Other command behavior provides parser and
help coverage until its development slices are completed.

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
test fixtures. Early work should implement read-only planning before writing
new POD5 files.

Preview:

.. code-block:: sh

   cargo run -- subdivide /path/to/folder

``playback``
------------

Replay an existing POD5 collection in sequencing-like order. This behavior will
be migrated from the Mnematikon implementation into standalone library and CLI
contracts.

Preview:

.. code-block:: sh

   cargo run -- playback --input /path/to/source --out /path/to/playback
