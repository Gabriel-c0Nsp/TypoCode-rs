//! Julia spec.
//!
//! Line comments are `#`. Block comments are `#= =#` (nestable).

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

pub const JULIA: LanguageSpec = LanguageSpec {
    names: &["julia"],
    extensions: &["jl"],
    line_comments: &["#"],
    block_comments: &[BlockComment {
        start: "#=",
        end: "=#",
        nestable: true,
        long_bracket: false,
    }],
    strings: &[
        StringLiteral::Triple { delim: '"' },
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
        assert_strip(&JULIA, "# header\nx = 1\n", "x = 1\n");
    }

    #[test]
    fn block_comment_drops_line() {
        assert_strip(&JULIA, "code\n#= doc =#\nmore\n", "code\nmore\n");
    }

    #[test]
    fn nested_block_balances() {
        let input = "#= a #= b =# c =#\nx = 1\n";
        assert_strip(&JULIA, input, "x = 1\n");
    }

    #[test]
    fn triple_quote_preserves_hash() {
        let input = "doc = \"\"\"\n# not a comment\n\"\"\"\n";
        assert_strip(&JULIA, input, input);
    }
}
