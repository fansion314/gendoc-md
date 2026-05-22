use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::cli::Options;
use crate::diagnostics::GendocError;
use crate::model::{DiscoveredProject, SourceTarget, SourceTree, TargetKind};

pub fn discover_targets(options: &Options) -> Result<DiscoveredProject> {
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

#[derive(Debug, Default)]
struct DiscoveryBuilder {
    targets: BTreeMap<String, SourceTarget>,
    top_level: BTreeSet<String>,
    children_by_parent: BTreeMap<String, BTreeSet<String>>,
}

impl DiscoveryBuilder {
    fn add_target(&mut self, target: SourceTarget) {
        self.add_import_name(&target.import_name);
        self.targets
            .entry(target.import_name.clone())
            .or_insert(target);
    }

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

fn absolute_path(path: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    })
}

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

fn expand_package(package: &SourceTarget, output: &Path) -> Result<DiscoveryBuilder> {
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

fn same_or_inside(path: &Path, parent: &Path) -> bool {
    if parent.as_os_str().is_empty() {
        return false;
    }
    path == parent || path.starts_with(parent)
}

fn validate_import_name(import_name: &str) -> Result<()> {
    if import_name.split('.').all(is_identifier) {
        Ok(())
    } else {
        Err(GendocError::InvalidImportName(import_name.to_string()).into())
    }
}

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
    use super::*;
    use std::fs;

    use crate::cli::Options;

    #[test]
    fn validates_import_names() {
        assert!(validate_import_name("pkg.mod").is_ok());
        assert!(validate_import_name("pkg.1bad").is_err());
    }

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
