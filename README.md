# minivim

A small vim-like modal editor in Rust. Keys are parsed into an `Action` by
`src/input.rs` — counts, operators, motions — and `src/editor.rs` applies it to
the buffer and cursor.

## Working today

- Normal and Insert modes.
- Motions: `h j k l`, `w W e E b B`, `0`, `$`, arrows.
- `f t F T` within the current line.
- `i a I A o O` to insert; typing, backspace, delete, enter and arrows all work
  there. `Esc` goes back.
- The grammar is parsed and tested: `12j`, `dw`, `2d3w`, `dd`, `Esc` to cancel.
- Scrolling (both axes), resize, status bar.
- `Ctrl-q` quits.

## Todo

### Parsed but ignored

The grammar understands these; the editor throws them away.

- [ ] `d`, `c`, `y` — `handle_operation` is an empty stub. Needs spans resolved
  against `MotionKind` (exclusive / inclusive / linewise) and somewhere to yank to.
- [ ] `x`, `p`, `P`, `J`, `u`, `Ctrl-r` — `handle_simple` is an empty stub.
  Undo wants an edit history in `Buffer` first.
- [ ] Counts, on movement. `3w` moves one word.
- [ ] Visual and Command modes set `self.mode` and stop there — no selection, no
  `:w`, no `:q`.

### Bugs

- [ ] `I` and `A` are swapped in `enter_insert`.
- [ ] `G` measures the wrong line and underflows on a one-line buffer.
  `g_moves_to_last_line` is `#[ignore]`d until it's fixed.
- [ ] `^` is an empty match arm.
- [ ] `gg` isn't parsed, so `Motion::FileStart` is unreachable.
- [ ] `$` stops on the end-of-line slot instead of the last character.
- [ ] `Ctrl-q` is caught before the parser, leaving `SimpleAction::Quit` dead.

### Not written yet

- [ ] Saving. `Buffer` knows it's modified but can't write itself out.
- [ ] Highlighting — `f` and an eventual `/` have no way to color what they find.
- [ ] `;` and `,` to repeat an `f`/`t`.
- [ ] `/`, `?`, `n`, `N`.
- [ ] `.`
- [ ] Registers.
- [ ] Text objects: `iw`, `ip`, `i(`.
- [ ] More than one buffer.

### Housekeeping

- [ ] Drop crossterm and set the terminal attributes by hand. C version
  [here](https://viewsourcecode.org/snaptoken/kilo/02.enteringRawMode.html).
- [ ] `View::mark_dirty` does nothing, so every frame repaints everything.
- [ ] `run` ends in `result.unwrap()` — panics after restoring the terminal
  rather than saying what went wrong.

---
