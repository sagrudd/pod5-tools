Development
===========

Language and Packaging
----------------------

``pod5-tools`` is implemented in Rust. The command-line interface uses
``clap`` and the repository exposes both a binary and a reusable library.

Rust Build and Test
-------------------

From the repository root:

.. code-block:: sh

   cargo fmt --check
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo run -- --help

The current implementation includes read-only discovery, fast verification,
official-POD5-backed metadata summaries, manifests, comparison, subdivision
planning, whole-file subdivision materialization, and playback schedule
inspection. Read-level POD5 rewriting still depends on a future concrete writer
backend.

Versioning
----------

Releases follow semantic versioning. Before the first stable release, versions
will use the ``0.y.z`` series while command and library contracts are still
being established.

Documentation
-------------

User-facing documentation is maintained with Sphinx and the Read the Docs theme.
Public Rust structs, enums, traits, and functions should include documentation
comments when introduced.

POD5 Reader Boundary
--------------------

POD5 metadata access is isolated behind the Rust ``Pod5MetadataReader`` trait.
Command code should depend on this trait rather than directly coupling itself to
a concrete POD5 parser. This keeps ``fileinfo``, ``folderinfo``, manifests, and
future subdivision planning testable without committing large binary fixtures to
the repository.

The command-line default is ``OfficialPod5MetadataReader``. It invokes Oxford
Nanopore's official Python ``pod5.Reader`` in a read-only subprocess and selects
the executable from ``POD5_TOOLS_PYTHON`` or ``python3``. ``fileinfo`` and
``folderinfo`` therefore require a Python environment where ``import pod5``
succeeds.

Reader adapters return typed errors with separate categories for path, format,
schema, and integrity failures. Commands should preserve those categories in
machine-readable output and use them to choose meaningful exit statuses in later
slices.

Backend Evaluation
------------------

The project now uses the official Python ``pod5`` reader for semantic
compatibility with ONT files and schema changes. A Rust-native Arrow reader can
still be evaluated later if packaging the Python dependency becomes a deployment
problem, but it must match the official implementation before it can replace the
default backend.

Local Documentation Build
-------------------------

From the repository root:

.. code-block:: sh

   python -m venv .venv
   . .venv/bin/activate
   pip install -r docs/requirements.txt
   sphinx-build -b html docs/source docs/build/html

Continuous Integration
----------------------

GitHub Actions runs the baseline project checks on pushes and pull requests:

.. code-block:: sh

   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   python -m sphinx -W -b html docs/source docs/build/html

Automation
----------

Automated development runs should read ``AGENTS.md``, ``roadmap.md``, and
``todo.md`` before editing. They should complete the first unchecked todo slice,
run relevant checks, update documentation, commit, push, and stop when all todo
items are complete.
