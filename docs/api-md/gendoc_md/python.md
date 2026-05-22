**gendoc_md > python**

# Module: python

## Contents

**Functions**

- [`parse_source`](#parse_source) - Parse Python source text into a renderable [`Document`].
- [`parse_target`](#parse_target) - Parse one discovered Python source target from disk.

---

## gendoc_md::python::parse_source

*Function*

Parse Python source text into a renderable [`Document`].

The extractor records module docstrings, static `__all__` values, public
functions, public classes, and public methods directly defined in classes.

# Errors

Returns an error when the source is not valid Python for the parser.

```rust
fn parse_source(target: &crate::model::SourceTarget, source: &str) -> anyhow::Result<crate::model::Document>
```



## gendoc_md::python::parse_target

*Function*

Parse one discovered Python source target from disk.

# Errors

Returns an error when the file cannot be read or parsed.

```rust
fn parse_target(target: &crate::model::SourceTarget) -> anyhow::Result<crate::model::Document>
```
