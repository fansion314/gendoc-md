**gendoc_md > discover**

# Module: discover

## Contents

**Functions**

- [`discover_targets`](#discover_targets) - Discover Python packages and modules selected by [`Options`].
- [`search_roots`](#search_roots) - Return existing, canonical search roots for Python import resolution.

---

## gendoc_md::discover::discover_targets

*Function*

Discover Python packages and modules selected by [`Options`].

When explicit packages or modules are provided, only those targets are
resolved. Otherwise, discovery scans existing search roots for top-level
regular packages and modules.

# Errors

Returns an error when search roots cannot be canonicalized, import names are
invalid, requested targets cannot be found, or package walking fails.

```rust
fn discover_targets(options: &crate::cli::Options) -> anyhow::Result<crate::model::DiscoveredProject>
```



## gendoc_md::discover::search_roots

*Function*

Return existing, canonical search roots for Python import resolution.

Defaults to `src` and `.` when no input roots are configured.

# Errors

Returns an error when an existing root cannot be canonicalized.

```rust
fn search_roots(options: &crate::cli::Options) -> anyhow::Result<Vec<std::path::PathBuf>>
```
