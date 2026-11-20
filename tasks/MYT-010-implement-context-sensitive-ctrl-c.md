# MYT-010 Implement context-sensitive Ctrl+C

## Description

Implement FR-5 per session. Verify the acceptance matrix, including a foreground command with text selected and a return to copy behavior once the prompt is idle.

## Acceptance criteria

- Verify the Ctrl+C acceptance matrix with real selection and the Wayland clipboard. Confirm copy resumes at the idle prompt and Ctrl+C forwards while state is unknown.

| Session state | Selection | Ctrl+C behavior |
| --- | --- | --- |
| Idle shell prompt | Text selected | Copy the selection to the system clipboard. |
| Idle shell prompt | No selection | Clear the app-owned draft and forward Ctrl+C to the terminal session. |
| Foreground command running | Any | Forward Ctrl+C to the terminal session so the foreground command receives normal terminal input and interrupt handling. |

## Related requirements

- FR-3 · Mouse selection
- FR-5 · Context-sensitive Ctrl+C
- FR-6 · App-owned command input

## Conclusion

The integrated app passed the Ctrl+C acceptance matrix on native KWin
Wayland. Idle selections copied from the active editor or terminal.
An unselected idle draft cleared and forwarded Ctrl+C. Foreground and
unknown states forwarded Ctrl+C while retaining the clipboard, and copy
resumed at the next confirmed prompt.

## Dev notes

Read the [shell-state prototype result](MYT-002-prototype-shell-state-detection.md) for hook loading and event-ordering checks.


- Shared `session::ctrl_c_action` returns CopySelection only for a confirmed
  idle prompt with an active selection. An unselected idle prompt clears the
  local draft and forwards Ctrl+C. Every other state forwards to the PTY,
  including foreground execution, unknown state, and pending submission.
- The adapter owns readiness validation. Integration calls `Session::prompt()`
  at dispatch to validate the private control channel, primary-prompt
  generation, shell foreground process group, and submission state. Advisory
  terminal output cannot establish readiness.
- The frontend chooses the active selection between the editor and terminal.
  Its Ctrl+C handler must consume the event before widget copy handling, so
  a retained selection cannot copy while the foreground program is running.
  Forward normal Ctrl+C input through the PTY to preserve terminal interrupt
  handling and settings.
- Formatting and the GUI-free shared session tests passed. The policy test
  exercises all combinations of confirmed readiness and active selection.
  The existing marker parsing test still passes. No dependencies were added.
### Native Wayland verification

- On 2026-11-20, the reference machine ran KWin 6.8.3 with a private
  1024 × 768 virtual output and disposable Zsh startup and history files.
  Pointer drags and keyboard events went through KWin's fake-input
  protocol. Clipboard assertions read the compositor's real standard
  clipboard with an external `wl-paste` process.
- A selected app-owned draft copied with Ctrl+C. An unselected draft
  cleared, and a subsequent command executed normally. Selecting terminal
  output at an idle prompt copied its exact text. Separate tabs retained
  their own draft and selection state.
- An external `cat` received an exact line through the PTY. With its
  terminal output selected, Ctrl+C interrupted the program and preserved a
  seeded clipboard value. A subsequent command confirmed prompt editing
  resumed. The shell's `read` builtin passed the same interrupt check while
  sharing the shell's process group. Afterward, idle Ctrl+C copied the
  selected terminal output again.
- An unsupported Bash session exercised unknown-state fallback. It received
  normal PTY input, forwarded Ctrl+C with text selected, preserved the
  clipboard, and accepted the next command. A process-local account lookup
  fixture selected Bash for this check. Host account defaults were untouched.
- The final foreground, builtin and unknown checks used the exact copied
  integrated binary with SHA-256
  `e7cfaec857a2ff94645c4d83fd36fbc0533b847997578d7444cbd439d61b6ff1`.
  Earlier idle and tab checks passed the immediately preceding integrated
  build. Test scripts and logs were kept under `/tmp/myt010-verify`.
