use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "gendoc-md",
    version,
    about = "Generate nested Markdown API maps for local Python projects."
)]
pub struct Options {
    #[arg(short = 'i', long = "input", value_name = "DIR")]
    pub inputs: Vec<PathBuf>,

    #[arg(short = 'p', long = "package", value_name = "IMPORT_NAME")]
    pub packages: Vec<String>,

    #[arg(short = 'm', long = "module", value_name = "IMPORT_NAME")]
    pub modules: Vec<String>,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "DIR",
        default_value = "docs/api-md"
    )]
    pub output: PathBuf,

    #[arg(long = "render-toc")]
    pub render_toc: bool,

    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<NonZeroUsize>,
}

impl Options {
    pub fn render_toc(&self) -> bool {
        self.render_toc
    }

    pub fn effective_jobs(&self) -> usize {
        self.jobs
            .map(NonZeroUsize::get)
            .unwrap_or_else(num_cpus::get)
    }
}

pub fn parse() -> Options {
    Options::parse()
}
