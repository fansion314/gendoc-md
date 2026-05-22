//! Shared documentation model used between discovery, parsing, and rendering.
//!
//! Types in this module intentionally avoid IO and parser-specific details so
//! architecture boundaries stay clear. Discovery fills source targets, the
//! Python parser fills documents, and the renderer consumes those documents.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// Kind of Python source target discovered on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetKind {
    /// A regular Python package represented by an `__init__.py` file.
    Package,
    /// A single Python module represented by a `.py` file.
    Module,
}

/// One Python file selected for parsing.
#[derive(Debug, Clone)]
pub struct SourceTarget {
    /// Dotted Python import name, such as `pkg.sub.module`.
    pub import_name: String,
    /// Filesystem path to the Python source file.
    pub path: PathBuf,
    /// Search root used to compute this target's import name.
    pub root: PathBuf,
    /// Whether this target is a package initializer or a standalone module.
    pub kind: TargetKind,
}

/// Complete discovery result for a run.
#[derive(Debug, Clone, Default)]
pub struct DiscoveredProject {
    /// Source files that should be parsed.
    pub targets: Vec<SourceTarget>,
    /// Import-name tree used by the renderer for navigation links.
    pub tree: SourceTree,
}

/// Deterministic import tree for packages and modules.
#[derive(Debug, Clone, Default)]
pub struct SourceTree {
    /// Top-level import names in sorted order.
    pub top_level: Vec<String>,
    /// Direct children keyed by parent import name.
    pub children_by_parent: BTreeMap<String, Vec<String>>,
}

impl SourceTree {
    /// Return direct child import names for `parent`.
    pub fn children(&self, parent: &str) -> &[String] {
        self.children_by_parent
            .get(parent)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Kind of generated documentation page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentKind {
    /// Documentation for a Python package initializer.
    Package,
    /// Documentation for a Python module.
    Module,
}

/// Parsed documentation facts for one Python source file.
#[derive(Debug, Clone)]
pub struct Document {
    /// Dotted Python import name for the documented file.
    pub import_name: String,
    /// Filesystem path to the source file that was parsed.
    pub source_path: PathBuf,
    /// Relative Markdown output path for this document.
    pub output_path: PathBuf,
    /// Whether this document represents a package or module.
    pub kind: DocumentKind,
    /// Module-level docstring, if present.
    pub docstring: Option<String>,
    /// First non-empty line of the module docstring.
    pub summary: Option<String>,
    /// Static string values declared in `__all__`.
    pub exports: Vec<String>,
    /// Public functions and classes defined at module scope.
    pub apis: Vec<ApiItem>,
}

/// Public API item extracted from a Python module.
#[derive(Debug, Clone)]
pub enum ApiItem {
    /// A public function defined at module scope.
    Function(FunctionDoc),
    /// A public class defined at module scope.
    Class(ClassDoc),
}

/// Documentation facts for a Python function or method.
#[derive(Debug, Clone)]
pub struct FunctionDoc {
    /// Function or method name without a containing class prefix.
    pub name: String,
    /// Renderable Python signature header ending at the declaration colon.
    pub signature: String,
    /// Function docstring, if present.
    pub docstring: Option<String>,
    /// Decorator expressions without the leading `@`.
    pub decorators: Vec<String>,
    /// Whether the function was declared with `async def`.
    pub is_async: bool,
}

/// Documentation facts for a Python class.
#[derive(Debug, Clone)]
pub struct ClassDoc {
    /// Class name.
    pub name: String,
    /// Renderable Python class header ending at the declaration colon.
    pub signature: String,
    /// Class docstring, if present.
    pub docstring: Option<String>,
    /// Public methods defined directly on the class.
    pub methods: Vec<FunctionDoc>,
}

/// Fully rendered Markdown file ready to write under the output directory.
#[derive(Debug, Clone)]
pub struct RenderedFile {
    /// Relative output path.
    pub path: PathBuf,
    /// Markdown file contents.
    pub content: String,
}
