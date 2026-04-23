//! Command-line argument parsing.
//!
//! Thin wrapper over `clap`'s derive API. Keeps CLI-shape concerns out of
//! `main.rs` so future flags (themes, alternate modes, etc.) have a
//! natural home.

use std::path::PathBuf;

use clap::{ArgAction, Parser};

/// TypoCode — terminal typing game that uses source code as practice text.
///
/// Pass a path to the file you want to type through. `--help` / `-h` prints
/// usage; `--version`, `-V`, and `-v` all print the crate version.
#[derive(Debug, Parser)]
#[command(name = "typocode", version, about, long_about = None, disable_version_flag = true)]
pub struct Cli {
    /// Print version information and exit.
    ///
    /// Accepts both the clap-default `-V` and the lowercase `-v` alias
    /// so users coming from the original C `TypoCode` keep muscle memory.
    #[arg(
        short = 'V',
        long = "version",
        short_alias = 'v',
        action = ArgAction::Version,
    )]
    #[allow(dead_code)]
    version: Option<bool>,

    /// Path to the source file to load as the typing challenge.
    pub path: PathBuf,
}

impl Cli {
    /// Parses the process arguments into a [`Cli`]. Exits the process via
    /// clap when the user passes `-h`, `-V`, or provides invalid args.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
