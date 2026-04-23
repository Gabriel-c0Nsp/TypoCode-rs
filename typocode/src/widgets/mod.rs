//! Render primitives for the typing UI.
//!
//! Each submodule owns one region of the frame: the typing body and its
//! extras overlay, the line-number gutter, the status footer, and the
//! end-of-run summary panel. `view::render` wires them together; none of
//! these modules know about [`crate::app::App`] directly.

pub mod body;
pub mod footer;
pub mod gutter;
pub mod summary;
