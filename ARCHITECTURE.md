# ARCHITECTURE.md

## 1. Project Overview

`gendoc-md` is a Rust command line tool that statically reads a local Python project and generates a nested Markdown API map for LLMs and humans.

- One-line description: generate a stable `docs/api-md` directory that mirrors Python packages and modules as Markdown documentation.
- Core use case: give an LLM a fast, navigable index of a project's public Python API without importing or executing the project.
- Non-goals: runtime introspection, dependency package documentation, namespace package support, and complete Python semantic analysis.
- Main tradeoff: prefer fast, deterministic static extraction over perfectly reflecting dynamic Python behavior.

## 2. High-Level Architecture

The tool has a narrow pipeline:

```text
CLI args
  -> target discovery
  -> parallel Python parsing
  -> documentation model
  -> Markdown rendering
  -> atomic-ish output directory replacement
```

The code is split into small layers:

- `cli`: argument parsing and conversion into runtime options.
- `discover`: resolves import names and finds local packages/modules.
- `python`: parses Python source and extracts documentation facts.
- `model`: shared data structures for files, APIs, packages, and render input.
- `render`: turns the model into deterministic Markdown files.
- `diagnostics`: user-facing error types and reporting helpers.

The core path is discovery, parsing, model assembly, and rendering. Packaging files such as `pyproject.toml` only exist to ship the binary through PyPI.

## 3. Code Map

- To change CLI parameters, edit `src/cli.rs`.
- To change default search behavior, package/module resolution, or ignored directories, edit `src/discover.rs`.
- To change Python AST extraction rules, public/private filtering, docstring extraction, or signature extraction, edit `src/python.rs`.
- To change the internal documentation representation, edit `src/model.rs`.
- To change Markdown shape, headings, links, optional TOC generation, or generated warnings, edit `src/render.rs`.
- To change error messages or exit behavior, edit `src/diagnostics.rs` and the orchestration in `src/lib.rs`.
- To change Python package publishing metadata, edit `pyproject.toml`.

There is no network logic in v1. If network behavior is added later, it should be isolated in a new module and kept out of the parser and renderer.

## 4. Main Execution Flow

1. `main` calls `gendoc_md::run()`.
2. `cli` parses arguments into `Options`.
3. `discover` builds search roots, resolves explicit `-p`/`-m` targets or discovers top-level targets when none are provided.
4. `discover` expands package targets into Python files, assigns each file an import name, and builds the package/module tree used by rendering.
5. `python` parses files in parallel and extracts module docstrings, `__all__`, public functions, classes, and public class members.
6. `render` builds all Markdown text in memory, including the top-level `index.md`.
7. Only after successful discovery, parsing, and rendering does the writer clear the output directory and write the new files.

## 5. Architectural Boundaries

- `render` must not parse Python source or inspect the filesystem beyond output paths it is given.
- `python` must not know CLI defaults, output directories, or Markdown formatting.
- `discover` must not inspect Python AST details beyond locating files and import names.
- `cli` must not perform filesystem mutations.
- `model` must stay free of IO and parser dependencies.

These boundaries keep feature changes local and make it hard for rendering concerns to leak into parsing or discovery.

## 6. Architectural Invariants

- The target Python project is never imported or executed.
- All generated output is deterministic for the same input tree.
- The output directory is cleared only after discovery, parsing, and Markdown rendering have all succeeded.
- Dangerous output directories such as `/`, the repository root, or an empty path are rejected.
- Public API means names that do not start with `_`, except static `__all__` is still reported as export metadata.
- Package documentation for `__init__.py` always renders to `index.md`.
- A top-level generated `index.md` is always created.

## 7. Cross-Cutting Concerns

Performance is handled by parallel file parsing and rendering with Rayon. Determinism is handled after parallel work by stable sorting before rendering and writing.

Error handling should prefer clear, actionable messages over partial success. Parse failures should prevent output deletion and writing.

Path handling should consistently use import names for Python identity and relative output paths for Markdown identity.

## 8. Extension Points

- Additional Python constructs should be added in `python` and represented in `model` before rendering changes are made.
- New output formats should add a renderer beside `render`, using the same documentation model.
- Future configuration files should be parsed before discovery and normalized into the same `Options` shape used by CLI args.
- Future language support should add a language-specific parser that emits the same high-level model, rather than changing Markdown rendering first.

## 9. Important Types and Concepts

- Search root: a directory used to resolve Python import names.
- Import name: dotted Python name such as `pkg.sub.module`.
- Package target: a directory with `__init__.py`, recursively documented.
- Module target: a single `.py` file documented as one Markdown page.
- Documented file: one parsed Python file plus derived metadata and APIs.
- Rendered file: one Markdown output path plus final content.

## 10. External Dependencies

- `clap`: command line argument parsing.
- `rayon`: multi-core parsing and rendering.
- `ignore`: fast recursive walking with common ignore behavior.
- Ruff/RustPython parser crates: static Python parsing and AST access.
- `maturin`: Python packaging for publishing the Rust binary as a PyPI package.

## 11. Repository Layout

```text
.
├── ARCHITECTURE.md
├── Cargo.toml
├── pyproject.toml
├── README.md
├── src/
│   ├── cli.rs
│   ├── diagnostics.rs
│   ├── discover.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── model.rs
│   ├── python.rs
│   └── render.rs
└── tests/
    └── cli.rs
```

## 12. Design Decisions

- Static parsing only: avoids user-code side effects and keeps the tool safe for arbitrary projects.
- Import-name `-m`: module targets are resolved like Python imports rather than raw paths.
- Clear output directory before writing: makes stale pages impossible, while the preflight pipeline protects existing docs on failure.
- Top-level `index.md`: gives both LLMs and humans a single stable entry point.
- Multi-core by default: large repositories should benefit from available CPU without extra flags.

## 13. Things That Are Intentionally Not Documented Here

This file does not document every AST node, every Markdown line, all CLI help text, or all tests. Those details belong in code and focused tests.

## 14. Maintenance Notes

Update this file when module boundaries change, a new language or output format is added, CLI semantics change, or a major invariant is relaxed.

Keep it short enough that a future maintainer or coding agent can read it before editing the repository.
