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
- Native Wayland verification remains for the combined adapter/editor build.
  Check idle selected editor text and terminal output, idle unselected draft
  clearing, interruption of an external command and a Zsh builtin with a
  selection, unknown-state forwarding, and copy resuming at a fresh idle
  prompt. Observe the real clipboard and verify selections in separate tabs
  remain scoped to their own session.
