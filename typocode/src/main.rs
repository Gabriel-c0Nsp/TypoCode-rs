//! TypoCode binary entry point — installs error / logging hooks and hands
//! off to the library run loop.

fn main() -> color_eyre::Result<()> {
    typocode::errors::install()?;
    let _log_guard = typocode::logging::init()?;
    typocode::app::run()
}
