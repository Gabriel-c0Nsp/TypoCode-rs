//! Markup-language specs (HTML, XML, SVG, Markdown).
//!
//! All four share the `<!-- -->` block comment and no line-comment
//! form. Markdown's stripping is intentionally simple: fenced code
//! blocks aren't protected, so a `<!--` literal inside ``` ``` would
//! be treated as a comment. Documented as a v1 limitation.

use crate::comments::spec::{BlockComment, LanguageSpec};

const HTML_BLOCK: &[BlockComment] = &[BlockComment {
    start: "<!--",
    end: "-->",
    nestable: false,
    long_bracket: false,
}];

pub const HTML: LanguageSpec = LanguageSpec {
    names: &["html"],
    extensions: &["html", "htm"],
    line_comments: &[],
    block_comments: HTML_BLOCK,
    strings: &[],
};

pub const XML: LanguageSpec = LanguageSpec {
    names: &["xml"],
    extensions: &["xml"],
    line_comments: &[],
    block_comments: HTML_BLOCK,
    strings: &[],
};

pub const SVG: LanguageSpec = LanguageSpec {
    names: &["svg"],
    extensions: &["svg"],
    line_comments: &[],
    block_comments: HTML_BLOCK,
    strings: &[],
};

pub const MARKDOWN: LanguageSpec = LanguageSpec {
    names: &["markdown", "md"],
    extensions: &["md", "markdown"],
    line_comments: &[],
    block_comments: HTML_BLOCK,
    strings: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn html_block_drops_line() {
        assert_strip(
            &HTML,
            "<p>hi</p>\n<!-- doc -->\n<p>bye</p>\n",
            "<p>hi</p>\n<p>bye</p>\n",
        );
    }

    #[test]
    fn html_inline_block_keeps_surrounding_code() {
        let input = "<a>x</a> <!-- hidden --> <b>y</b>\n";
        let out = crate::comments::stripper::strip(input, &HTML);
        assert!(out.contains("<a>x</a>"));
        assert!(out.contains("<b>y</b>"));
        assert!(!out.contains("hidden"));
    }

    #[test]
    fn html_block_spans_lines() {
        assert_strip(&HTML, "<x/>\n<!-- a\n   b -->\n<y/>\n", "<x/>\n<y/>\n");
    }

    #[test]
    fn xml_block_drops_line() {
        assert_strip(&XML, "<root>\n<!-- doc -->\n</root>\n", "<root>\n</root>\n");
    }

    #[test]
    fn svg_block_drops_line() {
        assert_strip(&SVG, "<svg>\n<!-- doc -->\n</svg>\n", "<svg>\n</svg>\n");
    }

    #[test]
    fn markdown_block_drops_line() {
        assert_strip(
            &MARKDOWN,
            "# Title\n<!-- note -->\nBody\n",
            "# Title\nBody\n",
        );
    }
}
