//! Erlang spec. Uses `%` as the line comment marker.

use crate::comments::spec::{LanguageSpec, StringLiteral};

pub const ERLANG: LanguageSpec = LanguageSpec {
    names: &["erlang"],
    extensions: &["erl", "hrl"],
    line_comments: &["%"],
    block_comments: &[],
    strings: &[
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
        assert_strip(&ERLANG, "% header\nfoo() -> 1.\n", "foo() -> 1.\n");
    }

    #[test]
    fn line_comment_after_code_preserves_code() {
        assert_strip(&ERLANG, "X = 1. % tail\n", "X = 1.\n");
    }

    #[test]
    fn percent_inside_string_preserved() {
        let input = "S = \"% not a comment\".\n";
        assert_strip(&ERLANG, input, input);
    }
}
