//! Binary entry point for `gendoc-md`.
//!
//! The binary delegates all behavior to the library crate so tests and other
//! callers can exercise the same pipeline without spawning a process.

/// Run the CLI and print a contextual error before exiting on failure.
fn main() {
    if let Err(error) = gendoc_md::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
