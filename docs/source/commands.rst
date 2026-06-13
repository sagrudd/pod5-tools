Command Plan
============

The command-line interface is implemented in Rust using ``clap``. The current
binary provides parser and help coverage for the planned subcommands, but the
commands are implementation stubs until their development slices are completed.

Preview help:

.. code-block:: sh

   cargo run -- --help
   cargo run -- find --help

``find``
--------

Find directories containing one or more POD5 files.

Preview:

.. code-block:: sh

   cargo run -- find /path/to/search

Planned output fields include:

* directory path;
* POD5 file count;
* total bytes;
* oldest modification time;
* newest modification time.

``fileinfo``
------------

Inspect a single POD5 file.

Preview:

.. code-block:: sh

   cargo run -- fileinfo /path/to/file.pod5 --format json

Planned output fields include:

* path and file size;
* flow cell ID;
* sequencing kit;
* read count;
* acquisition start date and time;
* duration of data;
* POD5 schema/version information;
* integrity status.

``folderinfo``
--------------

Aggregate file-level metadata across a folder or run tree.

Preview:

.. code-block:: sh

   cargo run -- folderinfo /path/to/folder

Planned checks include mixed flow cells, mixed sequencing kits, duplicate file
names, acquisition-time gaps, unreadable files, and integrity failures.

``manifest``
------------

Write stable JSON or TSV inventories for downstream tools, workflow engines, or
LIMS-style systems.

Preview:

.. code-block:: sh

   cargo run -- manifest /path/to/folder --format json

``compare``
-----------

Compare two POD5 collections or manifests and report missing files, added files,
duplicates, metadata drift, and integrity changes.

Preview:

.. code-block:: sh

   cargo run -- compare /path/to/left /path/to/right

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
