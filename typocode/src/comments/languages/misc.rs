//! Specs that don't cluster into a bigger family.
//!
//! Nim (`#` line, `#[ ]#` nestable block), Crystal (`#` line),
//! Zig (`//` line), Gleam (`//` line).

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const QUOTED: &[StringLiteral] = &[
    StringLiteral::Simple {
        delim: '"',
        escape: Some('\\'),
    },
    StringLiteral::Simple {
        delim: '\'',
        escape: Some('\\'),
    },
];

pub const NIM: LanguageSpec = LanguageSpec {
    names: &["nim"],
    extensions: &["nim"],
    line_comments: &["#"],
    block_comments: &[BlockComment {
        start: "#[",
        end: "]#",
        nestable: true,
        long_bracket: false,
    }],
    strings: QUOTED,
};

pub const CRYSTAL: LanguageSpec = LanguageSpec {
    names: &["crystal"],
    extensions: &["cr"],
    line_comments: &["#"],
    block_comments: &[],
    strings: QUOTED,
};

pub const ZIG: LanguageSpec = LanguageSpec {
    names: &["zig"],
    extensions: &["zig"],
    line_comments: &["//"],
    block_comments: &[],
    strings: QUOTED,
};

pub const GLEAM: LanguageSpec = LanguageSpec {
    names: &["gleam"],
    extensions: &["gleam"],
    line_comments: &["//"],
    block_comments: &[],
    strings: QUOTED,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn nim_line_comment_drops_line() {
        assert_strip(&NIM, "# header\necho 1\n", "echo 1\n");
    }

    #[test]
    fn nim_nested_block_balances() {
        let input = "#[ a #[ b ]# c ]#\necho 1\n";
        assert_strip(&NIM, input, "echo 1\n");
    }

    #[test]
    fn crystal_line_comment_drops_line() {
        assert_strip(&CRYSTAL, "# header\nputs 1\n", "puts 1\n");
    }

    #[test]
    fn zig_line_comment_drops_line() {
        assert_strip(&ZIG, "// header\nconst x = 1;\n", "const x = 1;\n");
    }

    #[test]
    fn gleam_line_comment_drops_line() {
        assert_strip(&GLEAM, "// header\nlet x = 1\n", "let x = 1\n");
    }
}
