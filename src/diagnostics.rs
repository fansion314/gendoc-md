use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GendocError {
    #[error("invalid import name `{0}`")]
    InvalidImportName(String),

    #[error("package `{import_name}` was not found in search roots: {roots:?}")]
    PackageNotFound {
        import_name: String,
        roots: Vec<PathBuf>,
    },

    #[error("module `{import_name}` was not found in search roots: {roots:?}")]
    ModuleNotFound {
        import_name: String,
        roots: Vec<PathBuf>,
    },
}
