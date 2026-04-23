//! TypoCode binary entry point — parses CLI args, installs error and
//! logging hooks, loads the source file, and hands off to the library
//! run loop.

use typocode::{app, cli::Cli, errors, file, logging};

fn main() -> color_eyre::Result<()> {
    errors::install()?;
    let _log_guard = logging::init()?;
    let cli = Cli::parse_args();
    let source = file::load(&cli.path)?;
    app::run(source)
}
