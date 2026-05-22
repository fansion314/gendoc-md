//! Integration tests for the `gendoc-md` binary.
//!
//! These tests exercise end-to-end CLI behavior with temporary Python projects
//! so discovery, parsing, rendering, and output replacement stay covered
//! together.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// Verify short and long help flags print the same compact help text.
#[test]
fn short_and_long_help_output_match() {
    let short_help = Command::cargo_bin("gendoc-md")
        .unwrap()
        .arg("-h")
        .output()
        .unwrap();
    let long_help = Command::cargo_bin("gendoc-md")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    assert!(short_help.status.success());
    assert!(long_help.status.success());
    assert_eq!(short_help.stdout, long_help.stdout);
    assert!(short_help.stderr.is_empty());
    assert!(long_help.stderr.is_empty());
}

/// Verify default discovery output for an explicit package target.
#[test]
fn generates_docs_for_default_src_project() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let pkg = root.join("src").join("sample_pkg");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        pkg.join("__init__.py"),
        r#""""Sample package."""

__all__ = ["make"]

def make(value: int) -> str:
    """Make a string."""
    return str(value)
"#,
    )
    .unwrap();
    fs::write(
        pkg.join("tools.py"),
        r#""""Tool helpers."""

class Tool:
    """A useful tool."""

    @staticmethod
    def build(name: str) -> "Tool":
        """Build a tool."""
        return Tool()
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("docs/api-md")).unwrap();
    fs::write(root.join("docs/api-md/stale.md"), "stale").unwrap();

    Command::cargo_bin("gendoc-md")
        .unwrap()
        .current_dir(root)
        .arg("-p")
        .arg("sample_pkg")
        .assert()
        .success();

    let top_index = fs::read_to_string(root.join("docs/api-md/index.md")).unwrap();
    assert!(top_index.contains("Do not modify"));
    assert!(top_index.contains("[sample_pkg](sample_pkg/index.md)"));
    assert!(!top_index.contains("## Table of Contents"));
    assert!(!root.join("docs/api-md/stale.md").exists());

    let pkg_index = fs::read_to_string(root.join("docs/api-md/sample_pkg/index.md")).unwrap();
    assert!(pkg_index.contains("# sample_pkg"));
    assert!(pkg_index.contains("[sample_pkg.tools](tools.md)"));
    assert!(pkg_index.contains("### make"));
    assert!(pkg_index.contains("def make(value: int) -> str:"));

    let module = fs::read_to_string(root.join("docs/api-md/sample_pkg/tools.md")).unwrap();
    assert!(module.contains("### Tool"));
    assert!(module.contains("#### Tool.build"));
}

/// Verify that optional table-of-contents rendering is controlled by the CLI.
#[test]
fn render_toc_flag_enables_tables_of_contents() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let pkg = root.join("src").join("sample_pkg");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(pkg.join("__init__.py"), "\"\"\"Sample package.\"\"\"\n").unwrap();

    Command::cargo_bin("gendoc-md")
        .unwrap()
        .current_dir(root)
        .arg("-p")
        .arg("sample_pkg")
        .arg("--render-toc")
        .assert()
        .success();

    let top_index = fs::read_to_string(root.join("docs/api-md/index.md")).unwrap();
    let pkg_index = fs::read_to_string(root.join("docs/api-md/sample_pkg/index.md")).unwrap();
    assert!(top_index.contains("## Table of Contents"));
    assert!(pkg_index.contains("## Table of Contents"));
}

/// Verify existing generated docs are preserved when parsing fails.
#[test]
fn parse_failure_keeps_existing_output() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let pkg = root.join("src").join("bad_pkg");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(pkg.join("__init__.py"), "def nope(:\n").unwrap();
    fs::create_dir_all(root.join("docs/api-md")).unwrap();
    fs::write(root.join("docs/api-md/keep.md"), "keep").unwrap();

    Command::cargo_bin("gendoc-md")
        .unwrap()
        .current_dir(root)
        .arg("-p")
        .arg("bad_pkg")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to parse"));

    assert_eq!(
        fs::read_to_string(root.join("docs/api-md/keep.md")).unwrap(),
        "keep"
    );
}
