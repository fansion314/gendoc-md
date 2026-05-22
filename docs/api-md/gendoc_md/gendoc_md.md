**gendoc_md**

# Module: gendoc_md

## Contents

**Modules**

- [`cli`](#cli) - Command line argument parsing for `gendoc-md`.
- [`diagnostics`](#diagnostics) - User-facing diagnostic errors for `gendoc-md`.
- [`discover`](#discover) - Python target discovery and import-tree construction.
- [`model`](#model) - Shared documentation model used between discovery, parsing, and rendering.
- [`python`](#python) - Static Python parsing and API extraction.
- [`render`](#render) - Markdown rendering for parsed documentation models.

**Functions**

- [`run`](#run) - Parse CLI arguments and run the documentation generator.
- [`run_with_options`](#run_with_options) - Run the generator with already parsed runtime options.

---

## Module: cli

Command line argument parsing for `gendoc-md`.

This module only translates user input into an [`Options`] value. It does
not perform discovery, parsing, rendering, or filesystem mutations.



## Module: diagnostics

User-facing diagnostic errors for `gendoc-md`.

Keep messages in this module actionable because they are printed directly by
the binary with context from `anyhow`.



## Module: discover

Python target discovery and import-tree construction.

Discovery maps local regular Python packages and modules to dotted import
names without importing target code. It also excludes generated output and
common cache/build directories so the renderer never documents its own
Markdown or tool artifacts.



## Module: model

Shared documentation model used between discovery, parsing, and rendering.

Types in this module intentionally avoid IO and parser-specific details so
architecture boundaries stay clear. Discovery fills source targets, the
Python parser fills documents, and the renderer consumes those documents.



## Module: python

Static Python parsing and API extraction.

This module uses RustPython/Ruff parser crates to inspect syntax without
importing or executing target code. It extracts stable documentation facts
and leaves Markdown formatting to the renderer.



## Module: render

Markdown rendering for parsed documentation models.

Rendering is deterministic and side-effect free: this module turns in-memory
[`Document`] values into [`RenderedFile`] values and does not read source
files or write output directories.



## gendoc_md::run

*Function*

Parse CLI arguments and run the documentation generator.

# Errors

Returns an error when arguments lead to invalid paths, discovery fails,
Python parsing fails, rendering fails, or generated files cannot be written.

```rust
fn run() -> anyhow::Result<()>
```



## gendoc_md::run_with_options

*Function*

Run the generator with already parsed runtime options.

The pipeline discovers Python targets, parses them in parallel, renders all
Markdown in memory, and writes the output only after those earlier stages
succeed.

# Errors

Returns an error for invalid output directories, Rayon configuration
failures, discovery failures, parse failures, render failures, or filesystem
write failures.

```rust
fn run_with_options(options: crate::cli::Options) -> anyhow::Result<()>
```
