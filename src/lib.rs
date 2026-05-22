//! Orchestration for the `gendoc-md` command line workflow.
//!
//! This crate keeps IO, discovery, parsing, and rendering in separate modules so
//! that Python source is never imported or executed. The top-level runner
//! validates destructive output choices before replacing generated Markdown
//! files.

pub mod cli;
pub mod diagnostics;
pub mod discover;
pub mod model;
pub mod python;
pub mod render;

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use rayon::prelude::*;

use crate::cli::Options;
use crate::model::RenderedFile;

/// Parse CLI arguments and run the documentation generator.
///
/// # Errors
///
/// Returns an error when arguments lead to invalid paths, discovery fails,
/// Python parsing fails, rendering fails, or generated files cannot be written.
pub fn run() -> Result<()> {
    let options = cli::parse();
    run_with_options(options)
}

/// Run the generator with already parsed runtime options.
///
/// The pipeline discovers Python targets, parses them in parallel, renders all
/// Markdown in memory, and writes the output only after those earlier stages
/// succeed.
///
/// # Errors
///
/// Returns an error for invalid output directories, Rayon configuration
/// failures, discovery failures, parse failures, render failures, or filesystem
/// write failures.
pub fn run_with_options(options: Options) -> Result<()> {
    // Implementation note: output validation happens before discovery so an
    // unsafe destination is rejected even if no Python targets are found.

    if let Some(jobs) = options.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.get())
            .build_global()
            .context("failed to configure rayon thread pool")?;
    }

    validate_output_dir(&options.output)?;

    let discovered = discover::discover_targets(&options)?;
    let mut documents = discovered
        .targets
        .par_iter()
        .map(python::parse_target)
        .collect::<Result<Vec<_>>>()?;

    documents.sort_by(|left, right| left.import_name.cmp(&right.import_name));

    let rendered = render::render_project(&documents, &discovered.tree, &options)?;
    write_rendered_files(&options.output, rendered)?;

    Ok(())
}

/// Reject output directories that could erase the project or filesystem root.
///
/// # Errors
///
/// Returns an error when the output path is empty, resolves to a filesystem
/// root, equals the current project directory, or is a parent of it.
fn validate_output_dir(output: &Path) -> Result<()> {
    // Implementation note: this is syntactic normalization rather than
    // canonicalization so callers can point to output directories that do not
    // exist yet.

    if output.as_os_str().is_empty() {
        bail!("output directory must not be empty");
    }

    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let absolute = if output.is_absolute() {
        output.to_path_buf()
    } else {
        cwd.join(output)
    };

    if absolute.parent().is_none() {
        bail!("refusing to use filesystem root as output directory");
    }

    let output = normalize(&absolute);
    let cwd = normalize(&cwd);

    if output == cwd {
        bail!("refusing to use the current project directory as output directory");
    }

    if cwd.starts_with(&output) {
        bail!("refusing to use a parent of the current project directory as output directory");
    }

    Ok(())
}

/// Replace the output directory with the rendered Markdown files.
///
/// # Errors
///
/// Returns an error when clearing, creating, or writing any output path fails.
fn write_rendered_files(output: &Path, files: Vec<RenderedFile>) -> Result<()> {
    // Implementation note: rendering is completed by the caller before this
    // function mutates the output directory, preserving old docs on parse
    // failures.

    if output.exists() {
        fs::remove_dir_all(output)
            .with_context(|| format!("failed to clear output directory {}", output.display()))?;
    }
    fs::create_dir_all(output)
        .with_context(|| format!("failed to create output directory {}", output.display()))?;

    for file in files {
        let path = output.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        fs::write(&path, file.content)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    Ok(())
}

/// Normalize `.` and `..` components without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
