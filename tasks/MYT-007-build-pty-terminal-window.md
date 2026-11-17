# MYT-007 Build PTY terminal window

## Description

Implement a window with a terminal backed by a PTY. Connect input and output, handle resizing, and clean up the child process when the session closes.

## Acceptance criteria

- Verify keyboard input reaches the shell, shell output renders, resize updates terminal dimensions, and closing the session cleans up its child process.

- Verify shared session and settings logic can build without linking the GUI. Exercise shell launch, input, output, resize and shell-state transitions through the boundary used by the neutral frontend. Record which terminal state the toolkit owns.

## Related requirements

- NFR-4
- FR-8

## Conclusion

The GTK4/VTE window passed the PTY lifecycle checks on native KWin Wayland.
The frontend uses shared window defaults and translates VTE shell markers
into toolkit-independent states. The shared library builds without the GUI.
MYT-011 owns production Zsh adapter setup, MYT-010 owns Ctrl+C policy, and
MYT-013 owns the settings UI and persistence.

## Dev notes

- The GTK4 entry point limits GDK to Wayland. VTE's asynchronous spawn
  attaches a PTY, launches and watches the shell, and reports launch failures.
  VTE owns keyboard encoding, PTY I/O and sizing, parsing, the grid, scrollback,
  selection, and rendering. Another frontend would need to supply those pieces
  while reusing the shared launch policy.
  See the [VTE spawn contract](https://gnome.pages.gitlab.gnome.org/vte/gtk4/method.Terminal.spawn_with_fds_async.html).
- The first slice launches the absolute path in `SHELL` when available, with
  `/bin/sh` as the fallback. MYT-008 tracks resolving the account's current
  default shell.
- Formatting and the GUI-free library check passed. The app linked in the
  `mytthy` Toolbx with GTK 4.24.1 and VTE 0.84.1.
- The Toolbx-built binary passed an isolated KWin 6.8.3 Wayland smoke test with
  Bash and an extracted VTE runtime. Keyboard input reached Bash, a capture
  showed the command output, and maximizing changed the PTY from 26 rows by
  99 columns to 38 rows by 127 columns. Closing the window removed the shell
  process, shell `exit` closed the window, and an invalid executable produced
  a launch error.
- A second KWin Wayland run used disposable Zsh startup hooks and the binary
  built in the Toolbx. Keyboard input, rendered output, PTY resize, and shell
  cleanup passed again. An external `sleep` and Zsh's `read` builtin each
  transitioned from prompt to foreground and back. A malformed marker
  produced Unknown until the next prompt. The GUI-free test covers recognized,
  absent, and malformed markers.
- For boundary testing, a disposable Zsh configuration can emit
  `printf '\\e]666;vte.ext.mytthy.shell-state=%s\\e\\\\' "$1"` with `prompt`
  from a primary ZLE line-init hook and `foreground` from a preexec hook.
  Run with `G_MESSAGES_DEBUG=mytthy` to log observed state changes. VTE can
  coalesce rapid property changes. These markers are advisory, and input
  handling needs to validate readiness independently.
- On Plasma Wayland, run `cargo run --locked`, type `printf 'mytthy OK\\n'`,
  and confirm the output renders. Run `stty size` before and after resizing
  and confirm the dimensions change. Record the shell PID with `echo $$`,
  close the window, and check that the PID is gone from a separate terminal.
  Open a second session and confirm typing `exit` closes its window.
  Try `SHELL=/no/such/shell cargo run --locked` and confirm the window shows
  the failed executable and VTE error.
