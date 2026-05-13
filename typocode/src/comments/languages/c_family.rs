//! C-family language specs.
//!
//! All members share `//` line comments and `/* */` block comments;
//! the differences live in whether blocks nest and what string
//! literals exist (raw strings in Rust, backticks in JS/TS, an
//! additional `#` line marker in PHP).

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const C_LINE: &[&str] = &["//"];
const C_BLOCK_PLAIN: &[BlockComment] = &[BlockComment {
    start: "/*",
    end: "*/",
    nestable: false,
    long_bracket: false,
}];
const C_BLOCK_NESTABLE: &[BlockComment] = &[BlockComment {
    start: "/*",
    end: "*/",
    nestable: true,
    long_bracket: false,
}];

const STRING_AND_CHAR: &[StringLiteral] = &[
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

const JS_STRINGS: &[StringLiteral] = &[
    StringLiteral::Backtick,
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

const RUST_STRINGS: &[StringLiteral] = &[
    StringLiteral::RawHash {
        prefix: "r",
        delim: '"',
    },
    StringLiteral::RawHash {
        prefix: "br",
        delim: '"',
    },
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

pub const RUST: LanguageSpec = LanguageSpec {
    names: &["rust"],
    extensions: &["rs"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_NESTABLE,
    strings: RUST_STRINGS,
};

pub const C: LanguageSpec = LanguageSpec {
    names: &["c"],
    extensions: &["c", "h"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: STRING_AND_CHAR,
};

pub const CPP: LanguageSpec = LanguageSpec {
    names: &["cpp", "c++"],
    extensions: &["cpp", "cxx", "cc", "hpp", "hxx", "hh"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: STRING_AND_CHAR,
};

pub const JAVA: LanguageSpec = LanguageSpec {
    names: &["java"],
    extensions: &["java"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: STRING_AND_CHAR,
};

pub const JAVASCRIPT: LanguageSpec = LanguageSpec {
    names: &["javascript", "js"],
    extensions: &["js", "mjs", "cjs", "jsx"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: JS_STRINGS,
};

pub const TYPESCRIPT: LanguageSpec = LanguageSpec {
    names: &["typescript", "ts"],
    extensions: &["ts", "tsx"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: JS_STRINGS,
};

pub const GO: LanguageSpec = LanguageSpec {
    names: &["go"],
    extensions: &["go"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: &[
        StringLiteral::Backtick,
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

pub const SWIFT: LanguageSpec = LanguageSpec {
    names: &["swift"],
    extensions: &["swift"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_NESTABLE,
    strings: STRING_AND_CHAR,
};

pub const KOTLIN: LanguageSpec = LanguageSpec {
    names: &["kotlin"],
    extensions: &["kt", "kts"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_NESTABLE,
    strings: STRING_AND_CHAR,
};

pub const SCALA: LanguageSpec = LanguageSpec {
    names: &["scala"],
    extensions: &["scala", "sc"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_NESTABLE,
    strings: STRING_AND_CHAR,
};

pub const CSHARP: LanguageSpec = LanguageSpec {
    names: &["csharp", "c#"],
    extensions: &["cs"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_PLAIN,
    strings: STRING_AND_CHAR,
};

pub const DART: LanguageSpec = LanguageSpec {
    names: &["dart"],
    extensions: &["dart"],
    line_comments: C_LINE,
    block_comments: C_BLOCK_NESTABLE,
    strings: STRING_AND_CHAR,
};

pub const PHP: LanguageSpec = LanguageSpec {
    names: &["php"],
    extensions: &["php"],
    line_comments: &["//", "#"],
    block_comments: C_BLOCK_PLAIN,
    strings: STRING_AND_CHAR,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn rust_line_comment_alone_drops_line() {
        assert_strip(&RUST, "// header\nfn main() {}\n", "fn main() {}\n");
    }

    #[test]
    fn rust_line_comment_after_code_preserves_code() {
        assert_strip(&RUST, "let x = 1; // tail\n", "let x = 1;\n");
    }

    #[test]
    fn rust_line_marker_inside_string_preserved() {
        let input = "let url = \"http://example.com\";\n";
        assert_strip(&RUST, input, input);
    }

    #[test]
    fn rust_block_alone_drops_line() {
        assert_strip(&RUST, "code\n/* block */\nmore\n", "code\nmore\n");
    }

    #[test]
    fn rust_block_spans_lines() {
        assert_strip(&RUST, "a\n/*\n one\n two\n*/\nb\n", "a\nb\n");
    }

    #[test]
    fn rust_nested_block_balances() {
        let input = "a /* outer /* inner */ outer */ b\n";
        let out = crate::comments::stripper::strip(input, &RUST);
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        assert!(!out.contains("outer"));
    }

    #[test]
    fn rust_raw_string_protects_inner_comment_marker() {
        let input = "let s = r#\"// not a comment\"#;\n";
        assert_strip(&RUST, input, input);
    }

    #[test]
    fn rust_byte_raw_string_supported() {
        let input = "let b = br\"// inside\";\n";
        assert_strip(&RUST, input, input);
    }

    #[test]
    fn c_block_does_not_nest() {
        // First */ closes; remainder is code.
        let input = "/* outer /* inner */ outer */\n";
        let out = crate::comments::stripper::strip(input, &C);
        assert!(out.contains("outer"));
    }

    #[test]
    fn cpp_handles_both_comment_styles() {
        let input = "int x = 0; /* a */ // tail\nint y = 1;\n";
        assert_strip(&CPP, input, "int x = 0;\nint y = 1;\n");
    }

    #[test]
    fn java_line_comment_drops_line() {
        assert_strip(
            &JAVA,
            "// header\npublic class X {}\n",
            "public class X {}\n",
        );
    }

    #[test]
    fn javascript_backtick_preserves_comment_markers() {
        let input = "const s = `// not a comment /* also */`;\n";
        assert_strip(&JAVASCRIPT, input, input);
    }

    #[test]
    fn javascript_block_comment_inline() {
        assert_strip(
            &JAVASCRIPT,
            "const x = 1 /* hint */ + 2;\n",
            "const x = 1  + 2;\n",
        );
    }

    #[test]
    fn typescript_line_comment_drops_line() {
        assert_strip(
            &TYPESCRIPT,
            "// header\nlet x: number = 1;\n",
            "let x: number = 1;\n",
        );
    }

    #[test]
    fn go_backtick_string_preserves_markers() {
        let input = "s := `// not a comment`\n";
        assert_strip(&GO, input, input);
    }

    #[test]
    fn swift_nested_block_balances() {
        let input = "/* outer /* inner */ outer */\nlet x = 1\n";
        assert_strip(&SWIFT, input, "let x = 1\n");
    }

    #[test]
    fn kotlin_nested_block_balances() {
        let input = "/* a /* b */ c */\nval x = 1\n";
        assert_strip(&KOTLIN, input, "val x = 1\n");
    }

    #[test]
    fn scala_nested_block_balances() {
        let input = "/* a /* b */ c */\nval x = 1\n";
        assert_strip(&SCALA, input, "val x = 1\n");
    }

    #[test]
    fn csharp_line_comment_drops_line() {
        assert_strip(
            &CSHARP,
            "// header\nclass X {}\n",
            "class X {}\n",
        );
    }

    #[test]
    fn dart_nested_block_balances() {
        let input = "/* a /* b */ c */\nvoid main() {}\n";
        assert_strip(&DART, input, "void main() {}\n");
    }

    #[test]
    fn php_hash_line_comment_drops_line() {
        assert_strip(&PHP, "# header\n$x = 1;\n", "$x = 1;\n");
    }

    #[test]
    fn php_slash_slash_line_comment_drops_line() {
        assert_strip(&PHP, "// header\n$x = 1;\n", "$x = 1;\n");
    }
}
