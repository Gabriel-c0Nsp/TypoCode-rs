//! Language description used by the comment stripper.
//!
//! Each supported language is a `const LanguageSpec` declared somewhere
//! under [`crate::comments::languages`]. The stripper engine never
//! grows when a new language is added; only the registry slice does.
//! That design also leaves room for a future config-file loader that
//! deserialises user-defined specs into the same shape.

/// Complete description of a language's comment and string syntax.
///
/// Lookups go through `extensions`; `names` is reserved for a future
/// manual override flag and config-file references.
#[derive(Debug)]
pub struct LanguageSpec {
    pub names: &'static [&'static str],
    pub extensions: &'static [&'static str],
    pub line_comments: &'static [&'static str],
    pub block_comments: &'static [BlockComment],
    pub strings: &'static [StringLiteral],
}

/// One block-comment shape supported by a language.
///
/// `nestable` lets Rust / Haskell / OCaml-family balance `/* /* */ */`.
/// `long_bracket` switches the matcher to Lua's `--[=*[ ... ]=*]` form,
/// where the open and close must agree on the count of `=` characters.
#[derive(Debug, Clone, Copy)]
pub struct BlockComment {
    pub start: &'static str,
    pub end: &'static str,
    pub nestable: bool,
    pub long_bracket: bool,
}

/// One string-literal shape the stripper must skip past so it doesn't
/// trip over a comment marker that lives inside source-code data.
#[derive(Debug, Clone, Copy)]
pub enum StringLiteral {
    /// `"..."` or `'...'`, optionally honouring a backslash escape.
    Simple {
        delim: char,
        escape: Option<char>,
    },
    /// Python-style triple-quoted string (`"""..."""` or `'''...'''`).
    Triple { delim: char },
    /// JavaScript template literal — `` `...` `` with backslash escapes.
    Backtick,
    /// Rust raw string: `r"..."`, `r#"..."#`, `r##"..."##`, ... Closing
    /// requires the same number of `#` as the opening sequence.
    RawHash {
        prefix: &'static str,
        delim: char,
    },
    /// Lua long bracket string: `[=*[ ... ]=*]`. Open and close levels
    /// must agree, identical to `BlockComment { long_bracket: true }`.
    LongBracket,
}
