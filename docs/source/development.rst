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
Docker-backed official POD5 metadata summaries, manifests, comparison,
subdivision planning, whole-file subdivision materialization, and playback
schedule inspection. Read-level POD5 rewriting still depends on a future
concrete writer backend.

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

The command-line default is ``DockerPod5MetadataReader``. It invokes
``docker run --rm --network none`` with the input file's parent directory
mounted read-only at ``/pod5-input``. The container image runs Oxford
Nanopore's official Python ``pod5.Reader`` and returns a small JSON metadata
record to the Rust command layer.

The Docker executable is selected from ``POD5_TOOLS_DOCKER`` or ``docker``.
The backend image is selected from ``POD5_TOOLS_POD5_IMAGE`` or
``pod5-tools-pod5:0.1.0``. If the selected image is absent, the reader builds
it from an embedded Dockerfile before running the parser.

The repository also includes the backend Dockerfile for manual pre-seeding or
package pinning:

.. code-block:: sh

   docker build -t pod5-tools-pod5:0.1.0 docker/pod5-backend
   docker build --build-arg POD5_PACKAGE=pod5==0.3.39 -t pod5-tools-pod5:0.1.0 docker/pod5-backend

Reader adapters return typed errors with separate categories for path, format,
schema, and integrity failures. Commands should preserve those categories in
machine-readable output and use them to choose meaningful exit statuses in later
slices.

Backend Evaluation
------------------

The project now uses a Dockerized official Python ``pod5`` reader for semantic
compatibility with ONT files and schema changes while avoiding host-level Python
environment coupling. Replicating the Python module directly in Rust is a
future optimization, not the current production path. A Rust-native Arrow/POD5
reader must be validated against the container backend on realistic fixtures
before it can replace the default parser.

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
