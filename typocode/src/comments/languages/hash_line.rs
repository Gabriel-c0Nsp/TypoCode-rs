//! Hash-line language specs.
//!
//! Languages that use `#` (or `%` / `--` cousins) as their primary
//! line-comment marker. Ruby and Perl also have a block-comment form
//! (`=begin/=end`, `=pod/=cut`); the built-in spec matches them
//! anywhere on the line for simplicity, even though both languages
//! technically require column 0.

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const HASH_LINE: &[&str] = &["#"];

const QUOTED_STRINGS: &[StringLiteral] = &[
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

const PYTHON_STRINGS: &[StringLiteral] = &[
    StringLiteral::Triple { delim: '"' },
    StringLiteral::Triple { delim: '\'' },
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

pub const PYTHON: LanguageSpec = LanguageSpec {
    names: &["python"],
    extensions: &["py", "pyw", "pyi"],
    line_comments: HASH_LINE,
    block_comments: &[],
    strings: PYTHON_STRINGS,
};

pub const RUBY: LanguageSpec = LanguageSpec {
    names: &["ruby"],
    extensions: &["rb"],
    line_comments: HASH_LINE,
    block_comments: &[BlockComment {
        start: "=begin",
        end: "=end",
        nestable: false,
        long_bracket: false,
    }],
    strings: QUOTED_STRINGS,
};

pub const BASH: LanguageSpec = LanguageSpec {
    names: &["bash"],
    extensions: &["sh", "bash", "zsh"],
    line_comments: HASH_LINE,
    block_comments: &[],
    strings: QUOTED_STRINGS,
};

pub const FISH: LanguageSpec = LanguageSpec {
    names: &["fish"],
    extensions: &["fish"],
    line_comments: HASH_LINE,
    block_comments: &[],
    strings: QUOTED_STRINGS,
};

pub const ELIXIR: LanguageSpec = LanguageSpec {
    names: &["elixir"],
    extensions: &["ex", "exs"],
    line_comments: HASH_LINE,
    block_comments: &[],
    strings: PYTHON_STRINGS,
};

pub const R: LanguageSpec = LanguageSpec {
    names: &["r"],
    extensions: &["r"],
    line_comments: HASH_LINE,
    block_comments: &[],
    strings: QUOTED_STRINGS,
};

pub const PERL: LanguageSpec = LanguageSpec {
    names: &["perl"],
    extensions: &["pl", "pm"],
    line_comments: HASH_LINE,
    block_comments: &[BlockComment {
        start: "=pod",
        end: "=cut",
        nestable: false,
        long_bracket: false,
    }],
    strings: QUOTED_STRINGS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn python_line_comment_alone_drops_line() {
        assert_strip(&PYTHON, "# header\nx = 1\n", "x = 1\n");
    }

    #[test]
    fn python_line_comment_after_code_preserves_code() {
        assert_strip(&PYTHON, "x = 1  # tail\n", "x = 1\n");
    }

    #[test]
    fn python_hash_inside_string_preserved() {
        let input = "x = \"#not a comment\"\n";
        assert_strip(&PYTHON, input, input);
    }

    #[test]
    fn python_triple_quote_preserves_hash() {
        let input = "doc = \"\"\"line one\n# not a comment\nline three\"\"\"\n";
        assert_strip(&PYTHON, input, input);
    }

    #[test]
    fn python_triple_quote_single_quotes() {
        let input = "doc = '''has # inside'''\n";
        assert_strip(&PYTHON, input, input);
    }

    #[test]
    fn ruby_block_comment_drops_lines() {
        let input = "code\n=begin\n  doc\n=end\nmore\n";
        assert_strip(&RUBY, input, "code\nmore\n");
    }

    #[test]
    fn ruby_line_comment_drops_line() {
        assert_strip(&RUBY, "# header\nputs 1\n", "puts 1\n");
    }

    #[test]
    fn bash_line_comment_drops_line() {
        assert_strip(&BASH, "# header\necho hi\n", "echo hi\n");
    }

    #[test]
    fn bash_shebang_is_dropped_like_any_comment() {
        // The shebang is syntactically a comment in the shell; tests
        // pin that behaviour so users aren't surprised.
        assert_strip(&BASH, "#!/bin/sh\necho hi\n", "echo hi\n");
    }

    #[test]
    fn bash_hash_inside_string_preserved() {
        let input = "echo \"#not a comment\"\n";
        assert_strip(&BASH, input, input);
    }

    #[test]
    fn fish_line_comment_drops_line() {
        assert_strip(&FISH, "# header\necho hi\n", "echo hi\n");
    }

    #[test]
    fn elixir_line_comment_drops_line() {
        assert_strip(&ELIXIR, "# header\nIO.puts(1)\n", "IO.puts(1)\n");
    }

    #[test]
    fn elixir_triple_quote_heredoc_preserves_hash() {
        let input = "doc = \"\"\"\n# not a comment\n\"\"\"\n";
        assert_strip(&ELIXIR, input, input);
    }

    #[test]
    fn r_line_comment_drops_line() {
        assert_strip(&R, "# header\nx <- 1\n", "x <- 1\n");
    }

    #[test]
    fn perl_line_comment_drops_line() {
        assert_strip(&PERL, "# header\nmy $x = 1;\n", "my $x = 1;\n");
    }

    #[test]
    fn perl_pod_block_drops_lines() {
        let input = "code\n=pod\n  doc\n=cut\nmore\n";
        assert_strip(&PERL, input, "code\nmore\n");
    }
}
