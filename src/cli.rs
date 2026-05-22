//! Command line argument parsing for `gendoc-md`.
//!
//! This module only translates user input into an [`Options`] value. It does
//! not perform discovery, parsing, rendering, or filesystem mutations.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser};

/// Runtime options accepted by the documentation generator.
///
/// Values come from Clap when running the binary, but tests and library callers
/// may also construct this type directly for deterministic execution.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "gendoc-md",
    version,
    about = "Generate nested Markdown API maps for local Python projects."
)]
pub struct Options {
    /// Search roots used to resolve Python packages and modules.
    #[arg(short = 'i', long = "input", value_name = "DIR")]
    pub inputs: Vec<PathBuf>,

    /// Explicit Python package import names to document recursively.
    #[arg(short = 'p', long = "package", value_name = "IMPORT_NAME")]
    pub packages: Vec<String>,

    /// Explicit Python module import names to document as individual files.
    #[arg(short = 'm', long = "module", value_name = "IMPORT_NAME")]
    pub modules: Vec<String>,

    /// Directory where generated Markdown files are written.
    #[arg(
        short = 'o',
        long = "output",
        value_name = "DIR",
        default_value = "docs/api-md"
    )]
    pub output: PathBuf,

    /// Whether rendered Markdown pages include table-of-contents sections.
    #[arg(long = "render-toc")]
    pub render_toc: bool,

    /// Optional Rayon worker count for parallel parsing and rendering.
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<NonZeroUsize>,
}

impl Options {
    /// Return whether table-of-contents sections should be rendered.
    pub fn render_toc(&self) -> bool {
        self.render_toc
    }

    /// Return the configured worker count, defaulting to available CPUs.
    pub fn effective_jobs(&self) -> usize {
        self.jobs
            .map(NonZeroUsize::get)
            .unwrap_or_else(num_cpus::get)
    }
}

/// Parse command line arguments into [`Options`].
pub fn parse() -> Options {
    let matches = Options::command()
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(ArgAction::HelpShort)
                .help("Print help (see more with '--help')"),
        )
        .get_matches();

    Options::from_arg_matches(&matches).unwrap_or_else(|error| error.exit())
}
