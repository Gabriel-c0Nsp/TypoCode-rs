//! SQL spec.

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

pub const SQL: LanguageSpec = LanguageSpec {
    names: &["sql"],
    extensions: &["sql"],
    line_comments: &["--"],
    block_comments: &[BlockComment {
        start: "/*",
        end: "*/",
        nestable: false,
        long_bracket: false,
    }],
    strings: &[StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn line_comment_drops_line() {
        assert_strip(
            &SQL,
            "-- header\nSELECT 1;\n",
            "SELECT 1;\n",
        );
    }

    #[test]
    fn line_comment_after_code_preserves_code() {
        assert_strip(
            &SQL,
            "SELECT 1; -- tail\n",
            "SELECT 1;\n",
        );
    }

    #[test]
    fn block_comment_drops_line() {
        assert_strip(
            &SQL,
            "code;\n/* doc */\nmore;\n",
            "code;\nmore;\n",
        );
    }

    #[test]
    fn dash_dash_inside_string_preserved() {
        let input = "SELECT '-- not a comment' FROM t;\n";
        assert_strip(&SQL, input, input);
    }
}
