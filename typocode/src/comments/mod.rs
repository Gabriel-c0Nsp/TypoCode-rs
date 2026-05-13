//! Optional comment stripping for source files.
//!
//! Public entry point: [`strip_by_extension`]. Given the source text
//! and a file extension, the function looks up a built-in
//! [`spec::LanguageSpec`] and runs the [`stripper`] engine. Unknown
//! extensions yield `None` so callers can fall back to the original
//! text — the silent pass-through behaviour the typing game wants.
//!
//! The module is data-driven by design: every supported language is a
//! `const` in [`languages`]. A future user-config story can append
//! deserialised specs to a runtime registry without touching the
//! engine.

pub mod languages;
pub mod registry;
pub mod spec;
pub mod stripper;

pub use registry::lookup_by_extension;
pub use spec::LanguageSpec;

/// Strips comments from `raw` when `ext` resolves to a known language.
///
/// Returns `Some(stripped)` when a language matches; `None` when the
/// extension is unknown or absent. Callers typically `.unwrap_or(raw)`
/// to keep the unmodified text in the unknown-language case.
pub fn strip_by_extension(raw: &str, ext: Option<&str>) -> Option<String> {
    let spec = lookup_by_extension(ext?)?;
    Some(stripper::strip(raw, spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_extension_yields_none() {
        assert!(strip_by_extension("anything", None).is_none());
    }

    #[test]
    fn unknown_extension_yields_none() {
        assert!(strip_by_extension("anything", Some("definitelynotreal")).is_none());
    }
}
