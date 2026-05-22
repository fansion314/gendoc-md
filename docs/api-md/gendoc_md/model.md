**gendoc_md > model**

# Module: model

## Contents

**Structs**

- [`ClassDoc`](#classdoc) - Documentation facts for a Python class.
- [`DiscoveredProject`](#discoveredproject) - Complete discovery result for a run.
- [`Document`](#document) - Parsed documentation facts for one Python source file.
- [`FunctionDoc`](#functiondoc) - Documentation facts for a Python function or method.
- [`RenderedFile`](#renderedfile) - Fully rendered Markdown file ready to write under the output directory.
- [`SourceTarget`](#sourcetarget) - One Python file selected for parsing.
- [`SourceTree`](#sourcetree) - Deterministic import tree for packages and modules.

**Enums**

- [`ApiItem`](#apiitem) - Public API item extracted from a Python module.
- [`DocumentKind`](#documentkind) - Kind of generated documentation page.
- [`TargetKind`](#targetkind) - Kind of Python source target discovered on disk.

---

## gendoc_md::model::ApiItem

*Enum*

Public API item extracted from a Python module.

**Variants:**
- `Function(FunctionDoc)` - A public function defined at module scope.
- `Class(ClassDoc)` - A public class defined at module scope.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> ApiItem`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::ClassDoc

*Struct*

Documentation facts for a Python class.

**Fields:**
- `name: String` - Class name.
- `signature: String` - Renderable Python class header ending at the declaration colon.
- `docstring: Option<String>` - Class docstring, if present.
- `methods: Vec<FunctionDoc>` - Public methods defined directly on the class.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> ClassDoc`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::DiscoveredProject

*Struct*

Complete discovery result for a run.

**Fields:**
- `targets: Vec<SourceTarget>` - Source files that should be parsed.
- `tree: SourceTree` - Import-name tree used by the renderer for navigation links.

**Trait Implementations:**

- **Default**
  - `fn default() -> DiscoveredProject`
- **Clone**
  - `fn clone(self: &Self) -> DiscoveredProject`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::Document

*Struct*

Parsed documentation facts for one Python source file.

**Fields:**
- `import_name: String` - Dotted Python import name for the documented file.
- `source_path: std::path::PathBuf` - Filesystem path to the source file that was parsed.
- `output_path: std::path::PathBuf` - Relative Markdown output path for this document.
- `kind: DocumentKind` - Whether this document represents a package or module.
- `docstring: Option<String>` - Module-level docstring, if present.
- `summary: Option<String>` - First non-empty line of the module docstring.
- `exports: Vec<String>` - Static string values declared in `__all__`.
- `apis: Vec<ApiItem>` - Public functions and classes defined at module scope.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> Document`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::DocumentKind

*Enum*

Kind of generated documentation page.

**Variants:**
- `Package` - Documentation for a Python package initializer.
- `Module` - Documentation for a Python module.

**Traits:** Eq

**Trait Implementations:**

- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
- **PartialEq**
  - `fn eq(self: &Self, other: &DocumentKind) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> DocumentKind`



## gendoc_md::model::FunctionDoc

*Struct*

Documentation facts for a Python function or method.

**Fields:**
- `name: String` - Function or method name without a containing class prefix.
- `signature: String` - Renderable Python signature header ending at the declaration colon.
- `docstring: Option<String>` - Function docstring, if present.
- `decorators: Vec<String>` - Decorator expressions without the leading `@`.
- `is_async: bool` - Whether the function was declared with `async def`.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> FunctionDoc`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::RenderedFile

*Struct*

Fully rendered Markdown file ready to write under the output directory.

**Fields:**
- `path: std::path::PathBuf` - Relative output path.
- `content: String` - Markdown file contents.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> RenderedFile`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::SourceTarget

*Struct*

One Python file selected for parsing.

**Fields:**
- `import_name: String` - Dotted Python import name, such as `pkg.sub.module`.
- `path: std::path::PathBuf` - Filesystem path to the Python source file.
- `root: std::path::PathBuf` - Search root used to compute this target's import name.
- `kind: TargetKind` - Whether this target is a package initializer or a standalone module.

**Trait Implementations:**

- **Clone**
  - `fn clone(self: &Self) -> SourceTarget`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::SourceTree

*Struct*

Deterministic import tree for packages and modules.

**Fields:**
- `top_level: Vec<String>` - Top-level import names in sorted order.
- `children_by_parent: std::collections::BTreeMap<String, Vec<String>>` - Direct children keyed by parent import name.

**Methods:**

- `fn children(self: &Self, parent: &str) -> &[String]` - Return direct child import names for `parent`.

**Trait Implementations:**

- **Default**
  - `fn default() -> SourceTree`
- **Clone**
  - `fn clone(self: &Self) -> SourceTree`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`



## gendoc_md::model::TargetKind

*Enum*

Kind of Python source target discovered on disk.

**Variants:**
- `Package` - A regular Python package represented by an `__init__.py` file.
- `Module` - A single Python module represented by a `.py` file.

**Traits:** Eq

**Trait Implementations:**

- **PartialEq**
  - `fn eq(self: &Self, other: &TargetKind) -> bool`
- **Clone**
  - `fn clone(self: &Self) -> TargetKind`
- **Debug**
  - `fn fmt(self: &Self, f: & mut $crate::fmt::Formatter) -> $crate::fmt::Result`
