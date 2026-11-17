# MYT-009 Support mouse selection and clipboard

## Description

Implement drag selection and system clipboard integration on Wayland. Verify FR-3 across wrapped lines and scrollback.

## Acceptance criteria

- Verify dragging selects text across wrapped lines and scrollback and that selected text can be copied to the Wayland clipboard.

## Related requirements

- FR-3 · Mouse selection.
- NFR-4 · Native Wayland.

## Conclusion

The Iced terminal grid supports drag selection, a visible selection
highlight, and Ctrl+Shift+C to copy to the system clipboard. A drag can be
extended through scrollback using the wheel. Ctrl+Shift+V sends clipboard
text through the terminal input callback and honors bracketed paste mode.

## Dev notes

- Selection belongs to each Alacritty terminal. The widget maps pointer
  coordinates through its actual layout bounds, padding, cell metrics and
  scroll offset. Alacritty extracts soft-wrapped text and handles wide and
  combining characters. Pointer state resets when the widget switches to a
  different terminal, using a weak reference to distinguish live sessions.
- The explicit copy shortcut works independently of shell state.
  MYT-010 owns context-sensitive Ctrl+C. The paste callback carries encoded
  bytes to the active session. Bracketed paste retains embedded newlines
  and strips embedded Escape characters that could terminate the bracket.
- The standalone Wayland fixture passed on the reference machine using a
  private KWin 6.8.3 compositor, an 800×500 window and disposable Zsh
  startup files. Virtual pointer input dragged from the first row to the
  next soft-wrapped row. An external `wl-paste` returned the exact selected
  substring without a newline at the soft wrap.
- After printing 100 numbered history lines, a drag selected older output
  and remained active while the wheel moved farther into scrollback.
  Copy returned 241 characters across the selected lines. Captures showed
  the highlighted selection.
- An external Wayland clipboard value containing a multiline command was
  pasted into Zsh. Neither line executed before explicit Enter. After Enter,
  both lines executed intact.
- Formatting, the default tests and Clippy with warnings treated as errors
  passed with disposable app wiring. Regression tests cover pointer
  coordinates and cell sides, scroll offsets, selection after reflow, and
  paste newline/bracket encoding. Alacritty clears the existing selection
  when a resize changes the column count. New selection uses the reflowed
  cells and current font metrics.
- The tabs integration supplies `Grid::on_input` with a callback scoped to a
  stable tab ID. Combined tabs and settings verification belongs to the
  coordinating integration run.

### Combined build verification

- The freshly rebuilt merged app passed wrapped selection, scrollback drag
  with wheel input, clipboard checks, multiline paste, and independent
  selections across tab switches. Font-size changes resized the PTY and
  new selection across reflowed rows copied correctly. Column-count changes
  clear the existing selection through Alacritty's resize behavior.
