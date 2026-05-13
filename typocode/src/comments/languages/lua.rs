//! Lua spec.
//!
//! Lua's block comment shares syntax with its long-bracket string
//! literal: `--[=*[ ... ]=*]`. The stripper's `long_bracket` flag on
//! both [`BlockComment`] and the [`StringLiteral::LongBracket`]
//! variant uses the same level-counting scanner under the hood.

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

pub const LUA: LanguageSpec = LanguageSpec {
    names: &["lua"],
    extensions: &["lua"],
    line_comments: &["--"],
    block_comments: &[BlockComment {
        start: "--",
        end: "",
        nestable: false,
        long_bracket: true,
    }],
    strings: &[
        StringLiteral::LongBracket,
        StringLiteral::Simple {
            delim: '"',
            escape: Some('\\'),
        },
        StringLiteral::Simple {
            delim: '\'',
            escape: Some('\\'),
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn line_comment_drops_line() {
        assert_strip(&LUA, "-- header\nlocal x = 1\n", "local x = 1\n");
    }

    #[test]
    fn line_comment_after_code_preserves_code() {
        assert_strip(&LUA, "local x = 1 -- tail\n", "local x = 1\n");
    }

    #[test]
    fn long_bracket_block_drops_lines() {
        assert_strip(&LUA, "code\n--[[\n block\n]]\nmore\n", "code\nmore\n");
    }

    #[test]
    fn long_bracket_block_with_levels() {
        let input = "code\n--[==[ inner ]] still ]==]\nmore\n";
        assert_strip(&LUA, input, "code\nmore\n");
    }

    #[test]
    fn long_bracket_string_preserves_dash_dash() {
        let input = "s = [==[has -- inside]==]\nx = 1\n";
        assert_strip(&LUA, input, input);
    }

    #[test]
    fn double_quote_string_preserves_dash_dash() {
        let input = "s = \"-- not a comment\"\n";
        assert_strip(&LUA, input, input);
    }
}
