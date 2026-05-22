//! Python target discovery and import-tree construction.
//!
//! Discovery maps local regular Python packages and modules to dotted import
//! names without importing target code. It also excludes generated output and
//! common cache/build directories so the renderer never documents its own
//! Markdown or tool artifacts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::cli::Options;
use crate::diagnostics::GendocError;
use crate::model::{DiscoveredProject, SourceTarget, SourceTree, TargetKind};

/// Discover Python packages and modules selected by [`Options`].
///
/// When explicit packages or modules are provided, only those targets are
/// resolved. Otherwise, discovery scans existing search roots for top-level
/// regular packages and modules.
///
/// # Errors
///
/// Returns an error when search roots cannot be canonicalized, import names are
/// invalid, requested targets cannot be found, or package walking fails.
pub fn discover_targets(options: &Options) -> Result<DiscoveredProject> {
    // Implementation note: discovery compares against the absolute output path
    // so default scans do not recursively include previously generated docs.

    let roots = search_roots(options)?;
    let output = absolute_path(&options.output)?;
    let mut discovered = DiscoveryBuilder::default();

    if options.packages.is_empty() && options.modules.is_empty() {
        discovered.merge(discover_top_level(&roots, &output)?);
    } else {
        for package in &options.packages {
            validate_import_name(package)?;
            let target = resolve_package(package, &roots)?;
            discovered.merge(expand_package(&target, &output)?);
        }

        for module in &options.modules {
            validate_import_name(module)?;
            let target = resolve_module(module, &roots)?;
            discovered.add_target(target);
        }
    }

    Ok(discovered.finish())
}

/// Mutable accumulator that preserves deterministic ordering while discovering.
#[derive(Debug, Default)]
struct DiscoveryBuilder {
    /// Targets keyed by import name to deduplicate overlapping search roots.
    targets: BTreeMap<String, SourceTarget>,
    /// Top-level import names.
    top_level: BTreeSet<String>,
    /// Direct children keyed by parent import name.
    children_by_parent: BTreeMap<String, BTreeSet<String>>,
}

impl DiscoveryBuilder {
    /// Add one target and record its import tree position.
    fn add_target(&mut self, target: SourceTarget) {
        self.add_import_name(&target.import_name);
        self.targets
            .entry(target.import_name.clone())
            .or_insert(target);
    }

    /// Merge another discovery accumulator into this one.
    fn merge(&mut self, other: DiscoveryBuilder) {
        self.top_level.extend(other.top_level);
        for (parent, children) in other.children_by_parent {
            self.children_by_parent
                .entry(parent)
                .or_default()
                .extend(children);
        }
        for target in other.targets.into_values() {
            self.targets
                .entry(target.import_name.clone())
                .or_insert(target);
        }
    }

    /// Convert the accumulator into the public discovery model.
    fn finish(self) -> DiscoveredProject {
        DiscoveredProject {
            targets: self.targets.into_values().collect(),
            tree: SourceTree {
                top_level: self.top_level.into_iter().collect(),
                children_by_parent: self
                    .children_by_parent
                    .into_iter()
                    .map(|(parent, children)| (parent, children.into_iter().collect()))
                    .collect(),
            },
        }
    }

    /// Record an import name in the source tree.
    fn add_import_name(&mut self, import_name: &str) {
        if let Some((parent, _)) = import_name.rsplit_once('.') {
            self.children_by_parent
                .entry(parent.to_string())
                .or_default()
                .insert(import_name.to_string());
        } else {
            self.top_level.insert(import_name.to_string());
        }
    }
}

/// Resolve a path against the current directory without requiring it to exist.
///
/// # Errors
///
/// Returns an error when the current directory cannot be read.
fn absolute_path(path: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    })
}

/// Return existing, canonical search roots for Python import resolution.
///
/// Defaults to `src` and `.` when no input roots are configured.
///
/// # Errors
///
/// Returns an error when an existing root cannot be canonicalized.
pub fn search_roots(options: &Options) -> Result<Vec<PathBuf>> {
    let roots = if options.inputs.is_empty() {
        vec![PathBuf::from("src"), PathBuf::from(".")]
    } else {
        options.inputs.clone()
    };

    let mut seen = BTreeSet::new();
    let mut existing = Vec::new();
    for root in roots {
        if root.is_dir() {
            let canonical = root
                .canonicalize()
                .with_context(|| format!("failed to canonicalize {}", root.display()))?;
            if seen.insert(canonical.clone()) {
                existing.push(canonical);
            }
        }
    }

    Ok(existing)
}

/// Discover top-level regular Python packages and modules under search roots.
///
/// # Errors
///
/// Returns an error when a search root cannot be read or package expansion
/// fails.
fn discover_top_level(roots: &[PathBuf], output: &Path) -> Result<DiscoveryBuilder> {
    let mut discovered = DiscoveryBuilder::default();
    for root in roots {
        for entry in std::fs::read_dir(root)
            .with_context(|| format!("failed to read search root {}", root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if should_skip_path(&path, output) {
                continue;
            }

            if path.is_dir() && path.join("__init__.py").is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if is_identifier(&name) {
                    let package = SourceTarget {
                        import_name: name,
                        path: path.join("__init__.py"),
                        root: root.clone(),
                        kind: TargetKind::Package,
                    };
                    discovered.merge(expand_package(&package, output)?);
                }
            } else if path.extension().is_some_and(|ext| ext == "py") {
                let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                    continue;
                };
                if stem == "__init__" || !is_identifier(stem) {
                    continue;
                }
                discovered.add_target(SourceTarget {
                    import_name: stem.to_string(),
                    path,
                    root: root.clone(),
                    kind: TargetKind::Module,
                });
            }
        }
    }

    Ok(discovered)
}

/// Resolve an explicit package import name to its `__init__.py`.
///
/// # Errors
///
/// Returns [`GendocError::PackageNotFound`] when no root contains the package.
fn resolve_package(import_name: &str, roots: &[PathBuf]) -> Result<SourceTarget> {
    let rel_dir = import_name.replace('.', std::path::MAIN_SEPARATOR_STR);
    for root in roots {
        let dir = root.join(&rel_dir);
        let init = dir.join("__init__.py");
        if init.is_file() {
            return Ok(SourceTarget {
                import_name: import_name.to_string(),
                path: init,
                root: root.clone(),
                kind: TargetKind::Package,
            });
        }
    }

    Err(GendocError::PackageNotFound {
        import_name: import_name.to_string(),
        roots: roots.to_vec(),
    }
    .into())
}

/// Resolve an explicit module import name to its `.py` file.
///
/// # Errors
///
/// Returns [`GendocError::ModuleNotFound`] when no root contains the module.
fn resolve_module(import_name: &str, roots: &[PathBuf]) -> Result<SourceTarget> {
    let rel_file = format!(
        "{}.py",
        import_name.replace('.', std::path::MAIN_SEPARATOR_STR)
    );
    for root in roots {
        let path = root.join(&rel_file);
        if path.is_file() {
            return Ok(SourceTarget {
                import_name: import_name.to_string(),
                path,
                root: root.clone(),
                kind: TargetKind::Module,
            });
        }
    }

    Err(GendocError::ModuleNotFound {
        import_name: import_name.to_string(),
        roots: roots.to_vec(),
    }
    .into())
}

/// Expand a regular package into all documentable Python files below it.
///
/// # Errors
///
/// Returns an error when the package path is malformed or walking fails.
fn expand_package(package: &SourceTarget, output: &Path) -> Result<DiscoveryBuilder> {
    // Implementation note: `ignore` honors common ignore files while the custom
    // filter prevents generated output and build/cache directories from leaking
    // into documentation.

    let package_dir = package
        .path
        .parent()
        .context("package __init__.py has no parent directory")?;
    let mut discovered = DiscoveryBuilder::default();

    let mut builder = WalkBuilder::new(package_dir);
    let output = output.to_path_buf();
    builder
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_exclude(true)
        .filter_entry(move |entry| !should_skip_path(entry.path(), &output));

    for entry in builder.build() {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|ext| ext != "py") {
            continue;
        }

        if let Some(target) = target_from_path(path, &package.root)? {
            discovered.add_target(target);
        }
    }

    Ok(discovered)
}

/// Build a source target for a Python path if it is inside regular packages.
///
/// # Errors
///
/// Returns an error when the path cannot be related to the search root.
fn target_from_path(path: &Path, root: &Path) -> Result<Option<SourceTarget>> {
    if !is_inside_regular_package(path, root)? {
        return Ok(None);
    }

    let rel = path.strip_prefix(root).with_context(|| {
        format!(
            "failed to compute import name for {} relative to {}",
            path.display(),
            root.display()
        )
    })?;
    let parts = rel
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return Ok(None);
    }

    let mut import_parts = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let is_last = index + 1 == parts.len();
        if is_last {
            let Some(stem) = Path::new(part).file_stem().and_then(|stem| stem.to_str()) else {
                return Ok(None);
            };
            if stem != "__init__" {
                if !is_identifier(stem) {
                    return Ok(None);
                }
                import_parts.push(stem.to_string());
            }
        } else if !is_identifier(part) {
            return Ok(None);
        } else {
            import_parts.push(part.clone());
        }
    }

    if import_parts.is_empty() {
        return Ok(None);
    }

    let kind = if path.file_name().is_some_and(|name| name == "__init__.py") {
        TargetKind::Package
    } else {
        TargetKind::Module
    };

    Ok(Some(SourceTarget {
        import_name: import_parts.join("."),
        path: path.to_path_buf(),
        root: root.to_path_buf(),
        kind,
    }))
}

/// Return whether every parent from `root` to `path` has an `__init__.py`.
///
/// # Errors
///
/// Returns an error when the package path cannot be stripped from the root.
fn is_inside_regular_package(path: &Path, root: &Path) -> Result<bool> {
    let package_dir = path.parent();
    let Some(package_dir) = package_dir else {
        return Ok(false);
    };

    let rel = package_dir.strip_prefix(root).with_context(|| {
        format!(
            "failed to validate package path {} relative to {}",
            path.display(),
            root.display()
        )
    })?;
    if rel.as_os_str().is_empty() {
        return Ok(false);
    }

    let mut cursor = root.to_path_buf();
    for component in rel.components() {
        cursor.push(component.as_os_str());
        if !cursor.join("__init__.py").is_file() {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Return whether a path should be excluded from discovery.
fn should_skip_path(path: &Path, output: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "__pycache__"
            | ".mypy_cache"
            | ".pytest_cache"
            | ".ruff_cache"
            | ".tox"
            | ".venv"
            | "venv"
            | "env"
            | "target"
            | "node_modules"
    ) || same_or_inside(path, output)
}

/// Return whether `path` is equal to or nested below `parent`.
fn same_or_inside(path: &Path, parent: &Path) -> bool {
    if parent.as_os_str().is_empty() {
        return false;
    }
    path == parent || path.starts_with(parent)
}

/// Validate a dotted Python import name.
///
/// # Errors
///
/// Returns [`GendocError::InvalidImportName`] when any component is not a
/// Python-style ASCII identifier accepted by this tool.
fn validate_import_name(import_name: &str) -> Result<()> {
    if import_name.split('.').all(is_identifier) {
        Ok(())
    } else {
        Err(GendocError::InvalidImportName(import_name.to_string()).into())
    }
}

/// Return whether a string is an ASCII Python identifier accepted by discovery.
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|char| char == '_' || char.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    //! Unit tests for discovery edge cases that do not require spawning the CLI.

    use super::*;
    use std::fs;

    use crate::cli::Options;

    /// Validate dotted import names and reject invalid components.
    #[test]
    fn validates_import_names() {
        assert!(validate_import_name("pkg.mod").is_ok());
        assert!(validate_import_name("pkg.1bad").is_err());
    }

    /// Verify package expansion records top-level and child import names.
    #[test]
    fn discovery_builds_import_tree() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("src");
        let pkg = root.join("pkg");
        let sub = pkg.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(pkg.join("__init__.py"), "").unwrap();
        fs::write(pkg.join("mod.py"), "").unwrap();
        fs::write(sub.join("__init__.py"), "").unwrap();
        fs::write(sub.join("leaf.py"), "").unwrap();

        let discovered = discover_targets(&Options {
            inputs: vec![root],
            packages: vec!["pkg".to_string()],
            modules: Vec::new(),
            output: temp.path().join("docs/api-md"),
            render_toc: false,
            jobs: None,
        })
        .unwrap();

        assert_eq!(discovered.tree.top_level, vec!["pkg"]);
        assert_eq!(discovered.tree.children("pkg"), ["pkg.mod", "pkg.sub"]);
        assert_eq!(discovered.tree.children("pkg.sub"), ["pkg.sub.leaf"]);
    }
}
