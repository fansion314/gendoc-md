**gendoc_md > diagnostics**

# Module: diagnostics

## Contents

**Enums**

- [`GendocError`](#gendocerror) - Domain errors reported by discovery and validation.

---

## gendoc_md::diagnostics::GendocError

*Enum*

Domain errors reported by discovery and validation.

**Variants:**
- `InvalidImportName(String)` - Import names must be dotted Python identifiers.
- `PackageNotFound{ import_name: String, roots: Vec<std::path::PathBuf> }` - A requested package could not be resolved from any search root.
- `ModuleNotFound{ import_name: String, roots: Vec<std::path::PathBuf> }` - A requested module could not be resolved from any search root.

**Traits:** Error

**Trait Implementations:**

- **Display**
  - `fn fmt(self: &Self, __formatter: & mut ::core::fmt::Formatter) -> ::core::fmt::Result`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
