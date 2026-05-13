//! Built-in language registry.
//!
//! `ALL` is the single source of truth for which languages the
//! stripper knows about out of the box. Adding a language means
//! declaring a `const LanguageSpec` in a `languages/*.rs` file and
//! pushing a reference into the slice below.

use super::languages::{c_family, hash_line};
use super::spec::LanguageSpec;

/// Every language the built-in stripper recognises. Future user-defined
/// specs would be appended to a separate runtime registry that this
/// slice does not need to know about.
pub const ALL: &[&LanguageSpec] = &[
    &c_family::RUST,
    &c_family::C,
    &c_family::CPP,
    &c_family::JAVA,
    &c_family::JAVASCRIPT,
    &c_family::TYPESCRIPT,
    &c_family::GO,
    &c_family::SWIFT,
    &c_family::KOTLIN,
    &c_family::SCALA,
    &c_family::CSHARP,
    &c_family::DART,
    &c_family::PHP,
    &hash_line::PYTHON,
    &hash_line::RUBY,
    &hash_line::BASH,
    &hash_line::FISH,
    &hash_line::ELIXIR,
    &hash_line::R,
    &hash_line::PERL,
];

/// Looks up a language by case-insensitive file extension (without the
/// leading dot). Returns `None` when no built-in spec matches.
pub fn lookup_by_extension(ext: &str) -> Option<&'static LanguageSpec> {
    let ext_lower = ext.to_ascii_lowercase();
    ALL.iter().copied().find(|spec| {
        spec.extensions
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(&ext_lower))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_extension_returns_none() {
        assert!(lookup_by_extension("totallyunknownext").is_none());
    }
}
