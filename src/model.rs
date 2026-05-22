use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetKind {
    Package,
    Module,
}

#[derive(Debug, Clone)]
pub struct SourceTarget {
    pub import_name: String,
    pub path: PathBuf,
    pub root: PathBuf,
    pub kind: TargetKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentKind {
    Package,
    Module,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub import_name: String,
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub kind: DocumentKind,
    pub docstring: Option<String>,
    pub summary: Option<String>,
    pub exports: Vec<String>,
    pub apis: Vec<ApiItem>,
}

#[derive(Debug, Clone)]
pub enum ApiItem {
    Function(FunctionDoc),
    Class(ClassDoc),
}

#[derive(Debug, Clone)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub docstring: Option<String>,
    pub decorators: Vec<String>,
    pub is_async: bool,
}

#[derive(Debug, Clone)]
pub struct ClassDoc {
    pub name: String,
    pub signature: String,
    pub docstring: Option<String>,
    pub methods: Vec<FunctionDoc>,
}

#[derive(Debug, Clone)]
pub struct RenderedFile {
    pub path: PathBuf,
    pub content: String,
}
