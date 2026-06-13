Development
===========

Language and Packaging
----------------------

``pod5-tools`` will be implemented in Rust. The command-line interface will use
``clap`` and the repository will expose both a binary and a reusable library.

Rust Build and Test
-------------------

From the repository root:

.. code-block:: sh

   cargo fmt --check
   cargo test
   cargo run -- --help

The first Rust slice establishes parser and help coverage only. Command
behavior is implemented in later todo slices.

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

Reader adapters return typed errors with separate categories for path, format,
schema, and integrity failures. Commands should preserve those categories in
machine-readable output and use them to choose meaningful exit statuses in later
slices.

Backend Evaluation
------------------

Two implementation routes remain open for the concrete POD5 reader:

* Rust-native Arrow access could keep the toolchain simpler for Rust users,
  allow direct integration with Rust data structures, and avoid shelling out to
  Python. The risk is that POD5 is more than generic Arrow tables; schema
  interpretation, compression details, and integrity behavior must match the
  official implementation closely.
* Binding to the official POD5 implementation should provide the best semantic
  compatibility with ONT files and schema changes. The tradeoff is packaging
  complexity, especially for a Rust binary expected to work cleanly on local
  infrastructure without fragile Python environment assumptions.

The next implementation step should prefer a small adapter spike before
expanding command behavior. The adapter must prove that it can read flow cell
ID, sequencing kit, read count, acquisition start, duration, version/schema
details, and integrity state from realistic POD5 files.

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
