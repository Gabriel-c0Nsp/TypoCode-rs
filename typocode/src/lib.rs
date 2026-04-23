//! Core library for the TypoCode terminal typing game.
//!
//! The binary in `main.rs` is a thin wrapper; integration tests and future
//! embedders drive the app through these modules directly. Modules are
//! added as the FR (functional requirement) that needs them lands.

pub mod app;
pub mod cli;
pub mod errors;
pub mod file;
pub mod logging;
pub mod text;
pub mod timer;
pub mod update;
