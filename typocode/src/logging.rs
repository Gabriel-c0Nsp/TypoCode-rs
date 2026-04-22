//! Tracing subscriber setup.
//!
//! Logs are written to `tracing.log` in the working directory using a
//! non-blocking appender so log emission never stalls the render loop.
//! Filtering follows the `RUST_LOG` env var (default `info`).

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Installs the global tracing subscriber and returns the appender guard.
///
/// The returned [`WorkerGuard`] flushes pending log records when dropped;
/// keep it bound for the lifetime of the program (typically in `main`).
pub fn init() -> color_eyre::Result<WorkerGuard> {
    let file_appender = tracing_appender::rolling::never(".", "tracing.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Ok(guard)
}
