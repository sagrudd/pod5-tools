Command Plan
============

The command-line interface will be implemented in Rust using ``clap``. The
project has not yet implemented these commands; this page records the intended
surface for the first development milestones.

``find``
--------

Find directories containing one or more POD5 files.

Planned output fields include:

* directory path;
* POD5 file count;
* total bytes;
* oldest modification time;
* newest modification time.

``fileinfo``
------------

Inspect a single POD5 file.

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

Planned checks include mixed flow cells, mixed sequencing kits, duplicate file
names, acquisition-time gaps, unreadable files, and integrity failures.

``manifest``
------------

Write stable JSON or TSV inventories for downstream tools, workflow engines, or
LIMS-style systems.

``compare``
-----------

Compare two POD5 collections or manifests and report missing files, added files,
duplicates, metadata drift, and integrity changes.

``subdivide``
-------------

Plan or write temporal and structural POD5 subdivisions for development and
test fixtures. Early work should implement read-only planning before writing
new POD5 files.

``playback``
------------

Replay an existing POD5 collection in sequencing-like order. This behavior will
be migrated from the Mnematikon implementation into standalone library and CLI
contracts.
