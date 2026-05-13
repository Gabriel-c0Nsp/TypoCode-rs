# TypoCode-rs

TypoCode is a very simple terminal-based game where the typing
challenges are made of computer science algorithms or source code files
provided by the user. The idea is to help people who are either
practicing typing skills or learning programming in general.

The main goal of the game is to be a fun little warm-up before a coding
session, something lightweight you can run directly in your terminal,
anytime and anywhere.

Rust rewrite of the original C
[TypoCode](https://github.com/Gabriel-c0Nsp/TypoCode), built on
[Ratatui](https://ratatui.rs) and `crossterm` for a faithful,
resize-aware, cross-platform experience.

The crate lives in [`typocode/`](./typocode) and is published to
crates.io as `typocode`.

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

Any UTF-8 text file works. Source code is the intended use: the
pagination and strict-match rules are designed to make typing through a
real codebase feel natural.

```bash
typo src/main.rs
typo README.md
typo ~/notes/algorithms.py
```

## Controls

| Key         | Action                                   |
|-------------|------------------------------------------|
| any char    | Type the next character.                 |
| `Tab`       | Restart the run.                         |
| `Esc`       | Quit.                                    |

Wrong keystrokes don't skip past the expected character. You have to
backspace your mistake before the cursor advances. Accuracy counts
every keystroke you make; backspacing doesn't penalise you.

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

- Word-aware wrapping. Character wrap keeps layout predictable for code.
- WPM / words-per-minute stat. Accuracy and time are the headline
  metrics.

## Development

```bash
cd typocode
cargo run -- path/to/file
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt
```

Debugging the typing engine during play: file logging is disabled by
default to avoid creating `tracing.log` in the working directory on
every run. To enable it, uncomment the `logging::init()?` call in
`typocode/src/main.rs`. Once enabled, each keystroke is logged to
`tracing.log` (written to the current working directory) via
`tracing`, which you can tail from another shell. Use the `RUST_LOG`
env var to adjust the filter level (defaults to `info`).

## License

MIT. See [LICENSE](./LICENSE).
