Overview
========

POD5 files preserve Oxford Nanopore raw signal and associated metadata. That
layer is not required by every downstream analysis, but it is important when a
facility needs to verify data integrity, repeat basecalling, preserve epigenetic
signal context, or troubleshoot provenance after data has moved through control
software and workflow staging areas.

``pod5-tools`` will provide a maintained toolbox for operational work around
POD5 collections:

* finding POD5-containing folders across storage trees;
* reporting file-level and folder-level QC information;
* creating stable manifests for downstream systems;
* comparing collections for missing, duplicated, or drifted data;
* subdividing data temporally or structurally for software development;
* replaying existing data in sequencing-like order for workflow tests.

Relationship to Existing Tools
------------------------------

The official ONT ``pod5`` package remains the authoritative ecosystem tool for
general POD5 operations such as ``view``, ``inspect``, ``merge``, ``filter``,
``subset``, ``repack``, schema updates, and FAST5 conversion.

The pod5Viewer project demonstrates the value of accessible inspection,
filtering, plotting, and export of individual reads and raw signal.

``pod5-tools`` should complement these projects by focusing on facility
operations, reproducible metadata inventories, integrity checks, workflow
development, and playback.
