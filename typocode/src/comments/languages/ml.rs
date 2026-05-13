//! ML-family specs (OCaml, F#, SML).
//!
//! All use nestable `(* *)` block comments. OCaml has no line comment.
//! F# adds `//` as a line comment. SML follows OCaml.

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const ML_BLOCK: &[BlockComment] = &[BlockComment {
    start: "(*",
    end: "*)",
    nestable: true,
    long_bracket: false,
}];
const ML_STRINGS: &[StringLiteral] = &[StringLiteral::Simple {
    delim: '"',
    escape: Some('\\'),
}];

pub const OCAML: LanguageSpec = LanguageSpec {
    names: &["ocaml"],
    extensions: &["ml", "mli"],
    line_comments: &[],
    block_comments: ML_BLOCK,
    strings: ML_STRINGS,
};

pub const FSHARP: LanguageSpec = LanguageSpec {
    names: &["fsharp", "f#"],
    extensions: &["fs", "fsi", "fsx"],
    line_comments: &["//"],
    block_comments: ML_BLOCK,
    strings: ML_STRINGS,
};

pub const SML: LanguageSpec = LanguageSpec {
    names: &["sml", "standard-ml"],
    extensions: &["sml"],
    line_comments: &[],
    block_comments: ML_BLOCK,
    strings: ML_STRINGS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn ocaml_block_drops_line() {
        assert_strip(
            &OCAML,
            "let x = 1\n(* doc *)\nlet y = 2\n",
            "let x = 1\nlet y = 2\n",
        );
    }

    #[test]
    fn ocaml_nested_block_balances() {
        let input = "(* a (* b *) c *)\nlet x = 1\n";
        assert_strip(&OCAML, input, "let x = 1\n");
    }

    #[test]
    fn ocaml_marker_inside_string_preserved() {
        let input = "let s = \"(* not a comment *)\"\n";
        assert_strip(&OCAML, input, input);
    }

    #[test]
    fn fsharp_line_comment_drops_line() {
        assert_strip(&FSHARP, "// header\nlet x = 1\n", "let x = 1\n");
    }

    #[test]
    fn fsharp_block_drops_line() {
        assert_strip(&FSHARP, "code\n(* doc *)\nmore\n", "code\nmore\n");
    }

    #[test]
    fn sml_block_drops_line() {
        assert_strip(
            &SML,
            "val x = 1\n(* doc *)\nval y = 2\n",
            "val x = 1\nval y = 2\n",
        );
    }
}
