//! Haskell-family specs.
//!
//! Haskell, Elm, and PureScript all use `--` line comments and
//! nestable `{- -}` block comments. Gleam uses `//` instead and lives
//! in [`super::misc`].

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const DASH_LINE: &[&str] = &["--"];
const CURLY_DASH_BLOCK: &[BlockComment] = &[BlockComment {
    start: "{-",
    end: "-}",
    nestable: true,
    long_bracket: false,
}];
const HASKELL_STRINGS: &[StringLiteral] = &[
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

pub const HASKELL: LanguageSpec = LanguageSpec {
    names: &["haskell"],
    extensions: &["hs", "lhs"],
    line_comments: DASH_LINE,
    block_comments: CURLY_DASH_BLOCK,
    strings: HASKELL_STRINGS,
};

pub const ELM: LanguageSpec = LanguageSpec {
    names: &["elm"],
    extensions: &["elm"],
    line_comments: DASH_LINE,
    block_comments: CURLY_DASH_BLOCK,
    strings: HASKELL_STRINGS,
};

pub const PURESCRIPT: LanguageSpec = LanguageSpec {
    names: &["purescript"],
    extensions: &["purs"],
    line_comments: DASH_LINE,
    block_comments: CURLY_DASH_BLOCK,
    strings: HASKELL_STRINGS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn haskell_line_comment_drops_line() {
        assert_strip(&HASKELL, "-- header\nx = 1\n", "x = 1\n");
    }

    #[test]
    fn haskell_line_inside_string_preserved() {
        let input = "s = \"-- not a comment\"\n";
        assert_strip(&HASKELL, input, input);
    }

    #[test]
    fn haskell_nested_block_balances() {
        let input = "a {- outer {- inner -} outer -} b\n";
        let out = crate::comments::stripper::strip(input, &HASKELL);
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        assert!(!out.contains("outer"));
    }

    #[test]
    fn haskell_block_spans_lines() {
        assert_strip(
            &HASKELL,
            "code\n{- block\n  spans -}\nmore\n",
            "code\nmore\n",
        );
    }

    #[test]
    fn elm_line_comment_drops_line() {
        assert_strip(&ELM, "-- header\nx = 1\n", "x = 1\n");
    }

    #[test]
    fn purescript_block_drops_line() {
        assert_strip(
            &PURESCRIPT,
            "x = 1\n{- doc -}\ny = 2\n",
            "x = 1\ny = 2\n",
        );
    }
}
