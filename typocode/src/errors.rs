//! Error and panic reporting.
//!
//! Wires up `color-eyre` for pretty error reports and replaces the default
//! panic hook with one that first restores the terminal — otherwise a
//! panic during a Ratatui frame leaves the user staring at a garbled
//! alternate screen with no visible message.

use color_eyre::config::HookBuilder;

/// Installs the `color-eyre` eyre hook and a panic hook that restores the
/// terminal before printing the panic.
///
/// Must be called once, early in `main`, before any code enables raw mode
/// or enters the alternate screen.
pub fn install() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    let panic_hook = panic_hook.into_panic_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        panic_hook(info);
    }));

    Ok(())
}
