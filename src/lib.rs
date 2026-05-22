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

pub fn run() -> Result<()> {
    let options = cli::parse();
    run_with_options(options)
}

pub fn run_with_options(options: Options) -> Result<()> {
    if let Some(jobs) = options.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.get())
            .build_global()
            .context("failed to configure rayon thread pool")?;
    }

    validate_output_dir(&options.output)?;

    let targets = discover::discover_targets(&options)?;
    let mut documents = targets
        .par_iter()
        .map(python::parse_target)
        .collect::<Result<Vec<_>>>()?;

    documents.sort_by(|left, right| left.import_name.cmp(&right.import_name));

    let rendered = render::render_project(&documents, &options)?;
    write_rendered_files(&options.output, rendered)?;

    Ok(())
}

fn validate_output_dir(output: &Path) -> Result<()> {
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

fn write_rendered_files(output: &Path, files: Vec<RenderedFile>) -> Result<()> {
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
