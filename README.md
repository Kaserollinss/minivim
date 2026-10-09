# minivim

Small vim clone in Rust. `src/input.rs` turns keypresses into an `Action`
(counts, operators, motions) and `src/editor.rs` runs it against the buffer and
cursor.

## What works

- Normal and Insert mode
- Motions: `h j k l`, `w W e E b B`, `0`, `$`, arrow keys
- `f t F T` on the current line
- `i a I A o O` for insert. Typing, backspace, delete, enter and arrows work.
  `Esc` gets you out.
- Parser handles `12j`, `dw`, `2d3w`, `dd`, and `Esc` to cancel (tested)
- Scrolling both ways, resize, status bar
- `Ctrl-q` to quit

## Todo

### Parsed but not hooked up

The parser gets these but the editor doesn't do anything with them yet.

- [ ] `d`, `c`, `y`. `handle_operation` is empty. Needs spans resolved against
  `MotionKind` (exclusive, inclusive, linewise) plus somewhere to store yanks.
- [ ] `x`, `p`, `P`, `J`, `u`, `Ctrl-r`. `handle_simple` is empty. Undo needs an
  edit history in `Buffer` first.
- [ ] Counts on movement. `3w` only moves one word.
- [ ] Visual and Command mode just set `self.mode`. No selection, no `:w`, no `:q`.

### Bugs

- [x] `I` and `A` are backwards in `enter_insert`
- [x] `G` checks the wrong line and underflows on a one-line buffer
- [ ] `^` match arm is empty
- [ ] `gg` isn't parsed so `Motion::FileStart` never gets hit
- [ ] `$` lands on the end-of-line slot instead of the last char
- [ ] `Ctrl-q` gets caught before the parser, so `SimpleAction::Quit` is dead code

### Not started

- [ ] Saving. `Buffer` tracks modified but can't write to disk.
- [ ] Highlighting, so `f` and eventually `/` can show what they matched
- [ ] `;` and `,` to repeat `f`/`t`
- [ ] `/`, `?`, `n`, `N`
- [ ] `.`
- [ ] Registers
- [ ] Text objects: `iw`, `ip`, `i(`
- [ ] Multiple buffers

### Cleanup

- [ ] Drop crossterm and set up raw mode by hand. C version
  [here](https://viewsourcecode.org/snaptoken/kilo/02.enteringRawMode.html).
- [ ] `View::mark_dirty` is a no-op so every frame redraws everything
- [ ] `run` ends with `result.unwrap()`, so errors panic after the terminal is
  restored instead of printing something useful
