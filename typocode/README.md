# typocode

Terminal typing game that uses source code as practice text. Loads any
text file, paginates it to fit your terminal. Live timer and accuracy in the footer; end-of-run summary with your final
time and accuracy.

Rust rewrite of the original C version
[TypoCode](https://github.com/Gabriel-c0Nsp/TypoCode), built on
[Ratatui](https://ratatui.rs) and `crossterm` for a faithful, resize-
aware, cross-platform experience.

## Install

```bash
cargo install typocode
```

Or from source:

```bash
git clone https://github.com/Gabriel-c0Nsp/TypoCode-rs
cd TypoCode-rs/typocode
cargo install --path .
```

## Usage

```bash
typo <path-to-file>
```

The crate is published as `typocode`, but the installed binary is
`typo`, matching the original C version's command name.

Any UTF-8 text file works. Source code is the intended use, the
pagination and strict-match rules are designed to make typing through
a real codebase feel natural.

```bash
typo src/main.rs
typo README.md
typo ~/notes/algorithms.py
```

### Stripping comments

Pass `--no-comments` (or `-C`) to remove comments before the run.
The language is auto-detected from the file extension; files in
unsupported languages pass through unchanged.

```bash
typo --no-comments src/main.rs
typo -C app.py
```

Both line and block comments are removed. Lines that contain only a
comment disappear entirely, so the player never has to type through a
hole left by a removed comment. String literals are respected, so a
comment-looking marker inside a string is left alone.

Built-in languages: Rust, C, C++, Java, JavaScript, TypeScript, Go,
Swift, Kotlin, Scala, C#, Dart, PHP, Python, Ruby, Bash, Fish,
Elixir, R, Perl, Lua, Haskell, Elm, PureScript, SQL, HTML, XML, SVG,
Markdown, Clojure, Common Lisp, Scheme, Racket, OCaml, F#, SML,
Erlang, Julia, Nim, Crystal, Zig, Gleam.

## Controls

| Key         | Action                                   |
|-------------|------------------------------------------|
| any char    | Type the next character.                 |
| `Tab`       | Restart the run.                         |
| `Esc`       | Quit.                                    |

## Features

- Strict-match typing with a visible extras buffer.
- Green / red / default colouring per character state.
- Line-number gutter that tracks **source** lines (visual wraps don't
  bump the counter).
- Adaptive pagination that reflows on resize.
- Live `mm:ss` timer and rounded accuracy percentage.
- Summary panel on completion showing final time + accuracy.
- Tab to restart from the top with a clean slate.

## Non-goals (for now)

- Word-aware wrapping. Char wrap keeps layout predictable for code.
- WPM / words-per-minute stat, accuracy and time are the headline metrics.

## License

MIT. See [LICENSE](../LICENSE).
