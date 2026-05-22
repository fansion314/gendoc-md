**gendoc_md > render**

# Module: render

## Contents

**Functions**

- [`render_project`](#render_project) - Render a complete project into Markdown files.

---

## gendoc_md::render::render_project

*Function*

Render a complete project into Markdown files.

The returned files include a top-level `index.md` plus one file per parsed
Python document.

# Errors

This function currently does not produce fallible rendering errors, but it
returns [`Result`] so callers can keep a uniform pipeline if rendering gains
validation later.

```rust
fn render_project(documents: &[crate::model::Document], tree: &crate::model::SourceTree, options: &crate::cli::Options) -> anyhow::Result<Vec<crate::model::RenderedFile>>
```
