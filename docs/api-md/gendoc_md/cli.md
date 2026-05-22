**gendoc_md > cli**

# Module: cli

## Contents

**Structs**

- [`Options`](#options) - Runtime options accepted by the documentation generator.

**Functions**

- [`parse`](#parse) - Parse command line arguments into [`Options`].

---

## gendoc_md::cli::Options

*Struct*

Runtime options accepted by the documentation generator.

Values come from Clap when running the binary, but tests and library callers
may also construct this type directly for deterministic execution.

**Fields:**
- `inputs: Vec<std::path::PathBuf>` - Search roots used to resolve Python packages and modules.
- `packages: Vec<String>` - Explicit Python package import names to document recursively.
- `modules: Vec<String>` - Explicit Python module import names to document as individual files.
- `output: std::path::PathBuf` - Directory where generated Markdown files are written.
- `render_toc: bool` - Whether rendered Markdown pages include table-of-contents sections.
- `jobs: Option<std::num::NonZeroUsize>` - Optional Rayon worker count for parallel parsing and rendering.

**Methods:**

- `fn render_toc(self: &Self) -> bool` - Return whether table-of-contents sections should be rendered.
- `fn effective_jobs(self: &Self) -> usize` - Return the configured worker count, defaulting to available CPUs.

**Traits:** Parser

**Trait Implementations:**

- **FromArgMatches**
  - `fn from_arg_matches(__clap_arg_matches: &clap::ArgMatches) -> ::std::result::Result<Self, clap::Error>`
  - `fn from_arg_matches_mut(__clap_arg_matches: & mut clap::ArgMatches) -> ::std::result::Result<Self, clap::Error>`
  - `fn update_from_arg_matches(self: & mut Self, __clap_arg_matches: &clap::ArgMatches) -> ::std::result::Result<(), clap::Error>`
  - `fn update_from_arg_matches_mut(self: & mut Self, __clap_arg_matches: & mut clap::ArgMatches) -> ::std::result::Result<(), clap::Error>`
- **CommandFactory**
  - `fn command<'b>() -> clap::Command`
  - `fn command_for_update<'b>() -> clap::Command`
- **Clone**
  - `fn clone(self: &Self) -> Options`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **Args**
  - `fn group_id() -> Option<clap::Id>`
  - `fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command`
  - `fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command`



## gendoc_md::cli::parse

*Function*

Parse command line arguments into [`Options`].

```rust
fn parse() -> Options
```
