# AGENTS.md

Guidance for AI agents and maintainers working in this repository.

## Project Overview

`gendoc-md` is a Rust command line tool that statically reads a local Python
project and generates a nested Markdown API map under `docs/api-md`.

Core goals:

- Generate deterministic Markdown documentation for public Python packages,
  modules, classes, functions, and declared exports.
- Avoid importing or executing the target Python project.
- Keep output navigation stable for LLMs and human maintainers.
- Preserve existing documentation on discovery or parse failure.

## Use A Staged Workflow

Use a staged workflow for code changes. Do not skip stages unless the user
explicitly asks for a different workflow.

### Stage 1: Exploration And Planning Only

- Read [ARCHITECTURE.md](ARCHITECTURE.md) first when the task requires
  understanding or changing code.
- Then read [docs/api-md/index.md](docs/api-md/index.md) if you need to
  understand public functions or module APIs.
- Read only the relevant source files after the architecture/API docs point you
  there.
- Do not edit files.
- Summarize the current implementation.
- Propose a minimal implementation plan.
- List files that need changes.
- List tests that should be added or updated.
- Identify risks, including parser, platform path, output directory, generated
  documentation, and compatibility risks.

Stop after Stage 1 and wait for user approval.

### Stage 2: Implementation

- Follow the approved minimal plan.
- Use a single worker agent if delegation is needed.
- Do not modify unrelated files.
- Do not introduce new dependencies unless necessary.
- Preserve existing architecture boundaries and invariants from
  [ARCHITECTURE.md](ARCHITECTURE.md).
- After editing, run:

```bash
./scripts/check.sh
```

- If the check script fails, inspect the logs and fix the root cause.
- Summarize changed files and verification results.

### Stage 3: Independent Review

- Review the final diff.
- Do not edit files during this stage.
- Check correctness, tests, error handling, maintainability, and platform
  issues.
- Pay special attention to static Python parsing behavior, output directory
  safety, deterministic rendering, generated API docs, and cross-platform path
  handling.
- Report findings by severity.
- If there are serious issues, propose a minimal fix plan and wait for approval
  before editing again.

## Architecture And API Docs

Read [ARCHITECTURE.md](ARCHITECTURE.md) when, and only when, you need project
architecture context or need to understand/modify code. Do not begin code tasks
by randomly reading source files; use the architecture document to build the map
first.

After reading the architecture document, prefer [docs/api-md/index.md](docs/api-md/index.md)
when you need to use public functions from project modules. The generated API
docs are hierarchical: start at `index.md`, then follow links to
module-specific Markdown files.

If the generated API docs are missing or stale, fall back to rustdoc comments
and source code, then regenerate the docs before committing.

## Code Documentation Standards

- Start every source file/module with module-level documentation that explains
  its purpose, important constraints, and project-specific context future
  maintainers need to know.
- Document every struct and function with API-facing rustdoc. Readers of
  generated Markdown docs should understand what the item does, how to use it,
  and when relevant, its `Examples` or `Errors`.
- Put implementation notes inside the function body near the top when the
  implementation depends on a project invariant, subtle ordering, or a
  non-obvious compatibility choice.
- Keep the pattern consistent:

```rust
/// API documentation extracted into Markdown docs.
pub fn example(input: Input) -> Result<Output> {
    // Implementation note for maintainers reading the source.

    todo!()
}
```

## Validation Command

After any code change, run:

```bash
./scripts/check.sh
```

The script runs:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`

Do not consider code changes complete until the script passes, or until you
clearly report why it could not be run.

## API Documentation Generation

Before committing code or API documentation changes, regenerate Markdown API
docs:

```bash
cargo doc-md --no-deps -o docs/api-md
```

Generated files under `docs/api-md` should be committed when source APIs or
their rustdoc change.

## Commit Workflow

Before committing code or API documentation changes:

1. Run the validation script:

```bash
./scripts/check.sh
```

2. Regenerate Markdown API docs:

```bash
cargo doc-md --no-deps -o docs/api-md
```

3. Use the [$git-commit](/Users/peilin/.cc-switch/skills/git-commit/SKILL.md)
   skill to create a conventional commit.

Commit rules:

- Use one logical change per commit.
- Never commit secrets, credentials, private config, or local-only artifacts.
- Do not skip hooks unless the user explicitly asks.
- Do not use destructive git commands without explicit user approval.
