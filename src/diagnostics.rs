//! User-facing diagnostic errors for `gendoc-md`.
//!
//! Keep messages in this module actionable because they are printed directly by
//! the binary with context from `anyhow`.

use std::path::PathBuf;

use thiserror::Error;

/// Domain errors reported by discovery and validation.
#[derive(Debug, Error)]
pub enum GendocError {
    /// Import names must be dotted Python identifiers.
    #[error("invalid import name `{0}`")]
    InvalidImportName(String),

    /// A requested package could not be resolved from any search root.
    #[error("package `{import_name}` was not found in search roots: {roots:?}")]
    PackageNotFound {
        /// Dotted Python package import name requested by the user.
        import_name: String,
        /// Search roots that were checked.
        roots: Vec<PathBuf>,
    },

    /// A requested module could not be resolved from any search root.
    #[error("module `{import_name}` was not found in search roots: {roots:?}")]
    ModuleNotFound {
        /// Dotted Python module import name requested by the user.
        import_name: String,
        /// Search roots that were checked.
        roots: Vec<PathBuf>,
    },
}
