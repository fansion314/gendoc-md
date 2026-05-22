use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use ruff_python_ast::{Expr, Stmt, StmtClassDef, StmtFunctionDef};
use ruff_text_size::{Ranged, TextRange};

use crate::model::{
    ApiItem, ClassDoc, Document, DocumentKind, FunctionDoc, SourceTarget, TargetKind,
};

pub fn parse_target(target: &SourceTarget) -> Result<Document> {
    let source = fs::read_to_string(&target.path)
        .with_context(|| format!("failed to read {}", target.path.display()))?;
    parse_source(target, &source)
}

pub fn parse_source(target: &SourceTarget, source: &str) -> Result<Document> {
    let parsed = ruff_python_parser::parse_module(source)
        .with_context(|| format!("failed to parse {}", target.path.display()))?;
    let module = parsed.syntax();

    let docstring = suite_docstring(&module.body);
    let summary = docstring.as_deref().and_then(first_doc_line);
    let exports = extract_all(&module.body);
    let apis = extract_api_items(&module.body, source);

    Ok(Document {
        import_name: target.import_name.clone(),
        source_path: target.path.clone(),
        output_path: output_path_for(&target.import_name, &target.kind),
        kind: match target.kind {
            TargetKind::Package => DocumentKind::Package,
            TargetKind::Module => DocumentKind::Module,
        },
        docstring,
        summary,
        exports,
        apis,
    })
}

fn output_path_for(import_name: &str, kind: &TargetKind) -> PathBuf {
    let mut path = import_name.split('.').collect::<PathBuf>();
    match kind {
        TargetKind::Package => path.push("index.md"),
        TargetKind::Module => {
            path.set_extension("md");
        }
    }
    path
}

fn suite_docstring(body: &[Stmt]) -> Option<String> {
    let Some(Stmt::Expr(expr_stmt)) = body.first() else {
        return None;
    };
    string_expr_value(&expr_stmt.value)
}

fn first_doc_line(docstring: &str) -> Option<String> {
    docstring
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn extract_all(body: &[Stmt]) -> Vec<String> {
    for stmt in body {
        let Stmt::Assign(assign) = stmt else {
            continue;
        };
        if !assign.targets.iter().any(is_all_name) {
            continue;
        }
        return string_sequence(&assign.value);
    }
    Vec::new()
}

fn extract_api_items(body: &[Stmt], source: &str) -> Vec<ApiItem> {
    let mut items = Vec::new();
    for stmt in body {
        match stmt {
            Stmt::FunctionDef(function) if is_public(function.name.as_str()) => {
                items.push(ApiItem::Function(function_doc(function, source)));
            }
            Stmt::ClassDef(class) if is_public(class.name.as_str()) => {
                items.push(ApiItem::Class(class_doc(class, source)));
            }
            _ => {}
        }
    }
    items
}

fn function_doc(function: &StmtFunctionDef, source: &str) -> FunctionDoc {
    FunctionDoc {
        name: function.name.to_string(),
        signature: header_from_range(source, function.range, HeaderKind::Function),
        docstring: suite_docstring(&function.body),
        decorators: decorators(source, &function.decorator_list),
        is_async: function.is_async,
    }
}

fn class_doc(class: &StmtClassDef, source: &str) -> ClassDoc {
    let methods = class
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::FunctionDef(function) if is_public(function.name.as_str()) => {
                Some(function_doc(function, source))
            }
            _ => None,
        })
        .collect();

    ClassDoc {
        name: class.name.to_string(),
        signature: header_from_range(source, class.range, HeaderKind::Class),
        docstring: suite_docstring(&class.body),
        methods,
    }
}

fn string_expr_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(string) => Some(string.value.to_str().to_string()),
        _ => None,
    }
}

fn string_sequence(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::List(list) => list.elts.iter().filter_map(string_expr_value).collect(),
        Expr::Tuple(tuple) => tuple.elts.iter().filter_map(string_expr_value).collect(),
        _ => Vec::new(),
    }
}

fn is_all_name(expr: &Expr) -> bool {
    matches!(expr, Expr::Name(name) if name.id.as_str() == "__all__")
}

fn decorators(source: &str, decorators: &[ruff_python_ast::Decorator]) -> Vec<String> {
    decorators
        .iter()
        .map(|decorator| {
            source_for_range(source, decorator.expression.range())
                .trim()
                .to_string()
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum HeaderKind {
    Function,
    Class,
}

fn header_from_range(source: &str, range: TextRange, kind: HeaderKind) -> String {
    let slice = source_for_range(source, range);
    let markers: &[&str] = match kind {
        HeaderKind::Function => &["def ", "async def "],
        HeaderKind::Class => &["class "],
    };

    let mut started = false;
    let mut lines = Vec::new();
    for line in slice.lines() {
        let trimmed = line.trim();
        if !started && markers.iter().any(|marker| trimmed.starts_with(marker)) {
            started = true;
        }
        if started {
            lines.push(trimmed.to_string());
            if trimmed.ends_with(':') {
                break;
            }
        }
    }

    lines.join("\n")
}

fn source_for_range(source: &str, range: TextRange) -> &str {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    source.get(start..end).unwrap_or("")
}

fn is_public(name: &str) -> bool {
    !name.starts_with('_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SourceTarget, TargetKind};

    fn target() -> SourceTarget {
        SourceTarget {
            import_name: "pkg.mod".to_string(),
            path: "pkg/mod.py".into(),
            root: ".".into(),
            kind: TargetKind::Module,
        }
    }

    #[test]
    fn extracts_docstrings_exports_and_public_api() {
        let source = r#""""Module summary.

More detail.
"""

__all__ = ["make", "Thing"]

def make(value: int) -> str:
    """Make a value."""
    return str(value)

def _hidden():
    pass

class Thing:
    """A thing."""

    @property
    def name(self) -> str:
        """Thing name."""
        return "x"

    def _private(self):
        pass
"#;

        let doc = parse_source(&target(), source).unwrap();
        assert_eq!(doc.summary.as_deref(), Some("Module summary."));
        assert_eq!(doc.exports, vec!["make", "Thing"]);
        assert_eq!(doc.apis.len(), 2);
        let ApiItem::Class(class) = &doc.apis[1] else {
            panic!("expected class");
        };
        assert_eq!(class.methods.len(), 1);
        assert_eq!(class.methods[0].decorators, vec!["property"]);
    }

    #[test]
    fn extracts_multiline_all_and_signatures() {
        let source = r#""""Module."""

__all__ = (
    "factory",
)

async def factory(
    value: int,
) -> str:
    """Build."""
    return str(value)
"#;

        let doc = parse_source(&target(), source).unwrap();
        assert_eq!(doc.exports, vec!["factory"]);
        let ApiItem::Function(function) = &doc.apis[0] else {
            panic!("expected function");
        };
        assert!(function.signature.contains("async def factory("));
        assert!(function.signature.ends_with(") -> str:"));
    }
}
