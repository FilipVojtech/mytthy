# MYT-006 Prototype app-owned command input

## Description

Prototype an app-owned editor at a confirmed idle prompt, using Zsh and Oh My Zsh as the first shell adapter. Keep editing independent of the shell. Submit the buffer through the adapter and hand input back to the PTY while a foreground program is running or prompt state is unknown.

## Acceptance criteria

- Demonstrate caret placement with a single click on plain and wrapped input. Verify a drag remains a selection gesture.
- Submit single-line and multiline commands without executing any part before explicit submission. Verify normal shell execution and history behavior with the configured Zsh.
- Verify PTY input during foreground execution and unknown state, and restore the editor when the prompt is confirmed idle. Check the FR-5 Ctrl+C behavior through these transitions.
- Identify what the editor and shell adapter each own, including where completion and history providers could connect. Record any limitations with shell-native behavior.

## Related requirements

- FR-6: place the caret with a click on plain and wrapped editable input.
- FR-3: preserve drag selection.
- FR-5: copy or interrupt according to prompt state and selection.
- NFR-4: verify on native Wayland.

## Conclusion

The app-owned GTK editor passed the revised checks with the configured Zsh
and Oh My Zsh on native KWin Wayland. Carry the editor and adapter split
forward to MYT-011. Keep the earlier ZLE demo as a separate reference.

## Dev notes

Both demos live under `src/` on the `prototype` branch.

### Previous ZLE prototype

GTK4 + VTE can support FR-6 for the controlled Zsh prompt. Plain clicks
and clicks on both rows of wrapped input inserted text at the expected
caret positions. Dragging still selected text and left the caret in place.
Continue with GTK4 + VTE for MYT-007. MYT-011 owns production shell
integration and general text layout.

- The standalone prototype is in `src/click-to-position/`. GTK observes
  button and motion events in the capture phase without consuming them. VTE
  retains selection. The frontend uses VTE's absolute cursor row, scroll
  adjustment and cell size to translate a click into a ZLE buffer offset.
- ZLE publishes its cursor offset, buffer length and terminal width through a
  [VTE terminal property](https://gnome.pages.gitlab.gnome.org/vte/gtk4/func.install_termprop.html).
  The frontend sends the target to a ZLE file-descriptor handler over a
  nonblocking pipe, leaving the PTY input stream untouched. ZLE rejects
  requests for older redraw generations and updates `CURSOR` directly.
- Verified on the reference machine with Fedora 44, KWin 6.8.2, GTK 4.22.5,
  VTE 0.84.1 and Zsh 5.9. Input went through an isolated native KWin Wayland
  compositor with a virtual output. The test checked rendered text after
  repeated plain clicks, clicks on row two and back on row one at 80 columns,
  and a drag that activated VTE selection without changing the insertion point.
  Python and Zsh syntax checks passed.
- The fixture runs without the user's Zsh startup files or Oh My Zsh. It
  assumes a fixed prompt and printable ASCII occupying one cell per character.
  MYT-011 must account for the active prompt and rendered text, including wide
  and combining characters, continuation lines, redraw ordering, and resize.
  Recheck with the configured Zsh and Oh My Zsh setup before shipping.

### Run the earlier ZLE demo

On Fedora, install `gtk4`, `vte291-gtk4` (VTE 0.78 or later), `python3-gobject`,
and `zsh`. From a Plasma Wayland session, run:

```sh
/usr/bin/python3 src/click-to-position/main.py
```

The prototype opens its own Zsh session with a fixed `myt> ` prompt and a
disposable startup configuration. It supports printable ASCII input and leaves
your Zsh configuration alone. Set `MYT_TRACE=1` to log the caret offset and
selection state.

1. Type `abcdefghij` without pressing Enter. Click the `d` cell and type `x`;
   the line should read `abcxdefghij`.
2. Press Ctrl+C, then type or paste enough letters to wrap onto a second row.
   Click a letter on the second row and type a marker. It should go before the
   clicked letter. Repeat on the first row.
3. Drag across several letters away from the caret. The letters should be
   highlighted. Type another marker and check that it goes at the previous
   caret position.

### App editor and shell adapter

- The app owns the draft, caret, wrapping, selection, and submission gesture.
  GTK's text view handles text layout and pointer interaction. VTE renders the
  shell prompt, transcript, and foreground programs. The demo places the
  editor in a separate panel below the terminal.
- The Zsh adapter sources the configured startup file and then uses a
  temporary history file. A ZLE hook publishes a session token and prompt
  generation when the primary prompt starts with an empty buffer. The app
  also checks the PTY foreground process group. On submission it disables
  the editor until a new idle prompt is confirmed.
- The adapter writes the entire UTF-8 draft to a nonblocking pipe and sends
  a private key sequence to wake ZLE. A widget checks the generation and
  transfers the draft to ZLE's buffer before invoking its built-in
  accept-line widget. This uses normal parsing, execution, and history.
  A pipe callback alone could update ZLE's buffer but left it waiting for a
  terminal key.
- A completion provider could receive the draft and caret position and return
  candidates with replacement ranges. Reuse a parser if a common format
  covers the supported sources. Otherwise use providers specific to the
  shell, format, or application. A history provider can query native shell
  history through the adapter and return entries for the app to insert.
  Completion and history-search scope remain open. ZLE widgets, autosuggestions,
  completion, and interactive history are available only when ZLE owns input.
- The demo uses a one-record pipe write and caps submissions at about 1 KiB
  of UTF-8 before encoding. It clears the draft when it queues the request.
  Production needs an acceptance acknowledgment and recovery if the prompt
  changes between the foreground check and the wake key. Only Zsh is wired up.
  Shell startup runs normally, so any startup hooks other than the history
  file setting retain their usual effects.

### Run the app-owned editor demo

On Fedora, install `gtk4`, `vte291-gtk4` (VTE 0.78 or later),
`python3-gobject`, and `zsh`. From the repository root in a Plasma Wayland
session, run:

```sh
python3 src/app-owned-input/main.py
```

1. Type a draft in the editor below the terminal. Click inside the text
   and type a marker. Repeat after the draft wraps onto a second row.
   Drag across text, then press Ctrl+C and paste to check the selection.
2. Submit a simple command with Enter. For a multiline draft, type
   `print -r -- first`, press Shift+Enter, and type `print -r -- second`.
   Check that both lines run only after Enter, then use `fc -ln -2` to
   inspect the last two entries in this session's temporary history.
3. Submit `cat`, type a line in the terminal, and confirm it echoes the line.
   Select some output and press Ctrl+C. The program should exit and the
   editor should return. At an idle prompt, press Ctrl+C with an unselected
   draft and check that it clears.
4. Submit `if true; then` and confirm the shell displays a continuation
   prompt with PTY input. Type `print -r -- done; fi` there or press
   Ctrl+C. The editor should return at the next primary prompt.

### Verification of the app-owned editor

On the reference machine, the isolated KWin Wayland test used Fedora 45,
KWin 6.8.3, GTK 4.24.1, VTE 0.84.1 and Zsh 5.9.2 with the configured Oh My
Zsh prompt. Pointer and keyboard events went through KWin's virtual output.

- Clicks placed the insertion point on plain input and the second row of
  wrapped input. Drag selected text and idle Ctrl+C copied that selection
  to the Wayland clipboard. Idle Ctrl+C without a selection cleared the draft.
- The edited single-line command ran and appeared in `fc`. Both lines of a
  multiline draft stayed in the editor until Enter, then ran and appeared as
  one escaped multiline history entry in `fc`.
- The external `cat` process received PTY input. Ctrl+C interrupted it with
  VTE output selected and left the clipboard intact. The Zsh `read` builtin
  received PTY input and returned normally. Both restored the editor.
- An incomplete `if` handed input to the shell's continuation prompt.
  Completing the command and interrupting a separate continuation both
  restored the editor at the next primary prompt.
