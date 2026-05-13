//! Lisp-family specs.
//!
//! All use `;` as the line comment marker. Common Lisp, Scheme, and
//! Racket also have the `#| |#` block comment (nestable). Clojure's
//! discarding reader `#_` is form-level, not character-level, so it
//! isn't modelled here.

use crate::comments::spec::{BlockComment, LanguageSpec, StringLiteral};

const SEMI_LINE: &[&str] = &[";"];
const HASH_PIPE_BLOCK: &[BlockComment] = &[BlockComment {
    start: "#|",
    end: "|#",
    nestable: true,
    long_bracket: false,
}];
const LISP_STRINGS: &[StringLiteral] = &[StringLiteral::Simple {
    delim: '"',
    escape: Some('\\'),
}];

pub const CLOJURE: LanguageSpec = LanguageSpec {
    names: &["clojure"],
    extensions: &["clj", "cljs", "cljc", "edn"],
    line_comments: SEMI_LINE,
    block_comments: &[],
    strings: LISP_STRINGS,
};

pub const COMMON_LISP: LanguageSpec = LanguageSpec {
    names: &["common-lisp", "lisp"],
    extensions: &["lisp", "lsp", "cl"],
    line_comments: SEMI_LINE,
    block_comments: HASH_PIPE_BLOCK,
    strings: LISP_STRINGS,
};

pub const SCHEME: LanguageSpec = LanguageSpec {
    names: &["scheme"],
    extensions: &["scm", "ss"],
    line_comments: SEMI_LINE,
    block_comments: HASH_PIPE_BLOCK,
    strings: LISP_STRINGS,
};

pub const RACKET: LanguageSpec = LanguageSpec {
    names: &["racket"],
    extensions: &["rkt"],
    line_comments: SEMI_LINE,
    block_comments: HASH_PIPE_BLOCK,
    strings: LISP_STRINGS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comments::languages::assert_strip;

    #[test]
    fn clojure_line_comment_drops_line() {
        assert_strip(&CLOJURE, "; header\n(defn f [] 1)\n", "(defn f [] 1)\n");
    }

    #[test]
    fn clojure_semi_inside_string_preserved() {
        let input = "(def s \"; not a comment\")\n";
        assert_strip(&CLOJURE, input, input);
    }

    #[test]
    fn common_lisp_block_drops_line() {
        assert_strip(&COMMON_LISP, "code\n#| doc |#\nmore\n", "code\nmore\n");
    }

    #[test]
    fn scheme_block_nests() {
        let input = "#| outer #| inner |# outer |#\n(define x 1)\n";
        assert_strip(&SCHEME, input, "(define x 1)\n");
    }

    #[test]
    fn racket_line_comment_drops_line() {
        assert_strip(&RACKET, "; header\n(define x 1)\n", "(define x 1)\n");
    }
}
