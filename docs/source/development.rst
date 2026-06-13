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
