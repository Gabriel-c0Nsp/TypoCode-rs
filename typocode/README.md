# typocode

Terminal typing game that uses source code as practice text. Loads any
text file, paginates it to fit your terminal, and walks you through it
character by character — correct keystrokes turn green, wrong ones
stack as red glyphs you must backspace before typing can resume. Live
timer and accuracy in the footer; end-of-run summary with your final
time and accuracy.

Rust rewrite of the original C
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
typocode <path-to-file>
```

Any UTF-8 text file works. Source code is the intended use — the
pagination and strict-match rules are designed to make typing through
a real codebase feel natural.

```bash
typocode src/main.rs
typocode README.md
typocode ~/notes/algorithms.py
```

## Controls

| Key         | Action                                   |
|-------------|------------------------------------------|
| any char    | Type the next character.                 |
| `Space`     | Same as typing a literal space.          |
| `Enter`     | Commit the current line / advance page.  |
| `Backspace` | Undo the most recent extra or character. |
| `Tab`       | Restart the run.                         |
| `Esc`       | Quit.                                    |

Wrong keystrokes don't skip past the expected character — you have to
backspace your mistake before the cursor advances. Accuracy counts
every keystroke you make; backspacing doesn't penalise you.

## Features

- Strict-match typing with a visible extras buffer.
- Green / red / default colouring per character state.
- Line-number gutter that tracks **source** lines — visual wraps don't
  bump the counter.
- Adaptive pagination that reflows on resize.
- Live `mm:ss` timer and rounded accuracy percentage.
- Summary panel on completion showing final time + accuracy.
- Tab to restart from the top with a clean slate.

## Non-goals (for now)

- Word-aware wrapping. Char wrap keeps layout predictable for code.
- WPM / words-per-minute stat — accuracy is the headline metric.
- Themes, multiple languages, network leaderboards.

## License

MIT. See [LICENSE](../LICENSE).
