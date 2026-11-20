# MYT-011 Implement app-owned command input

## Description

Implement the app-owned prompt editor from MYT-006. Keep buffer editing shared and put prompt detection, submission, and shell-specific behavior behind shell adapters.

Completion is tracked in [MYT-030](MYT-030-implement-command-completion.md).

## Acceptance criteria

- Click several positions on plain and wrapped input. Confirm typing inserts there and dragging still selects text.
- Edit multiline commands locally. Verify that Enter submits the full buffer through the normal shell command path, that embedded newlines execute nothing before submission, and that commands appear in shell history according to shell policy.
- Forward input to the PTY during foreground execution or unknown shell state and restore app editing at a confirmed idle prompt. Verify the FR-5 Ctrl+C cases.

## Related requirements

- FR-3 · Mouse selection
- FR-5 · Context-sensitive Ctrl+C
- FR-6 · App-owned command input

## Conclusion

The app-owned Zsh editor passed native Wayland caret, selection, multiline
submission, normal history, per-tab draft, font-size, and configured Oh My
Zsh checks. MYT-010 verified foreground and unknown-state PTY handoff and
the Ctrl+C matrix. Completion remains in MYT-030.

## Dev notes

- History search scope remains an open product decision.


### Implementation

- Each tab keeps an app-owned command draft and native editor content. The
  shared `command_input::CommandDraft` holds command text and the pending
  submission generation without GUI or terminal-engine types. Iced handles
  pointer hit testing, caret movement, wrapping, selection, and local editing.
- At a confirmed Zsh primary prompt, the editor appears below the terminal
  transcript. Enter submits the complete buffer, and Shift+Enter inserts a
  newline. Clipboard paste edits the local draft until explicit submission.
  Font-size settings apply to the transcript and command editor in all tabs.
- Prompt readiness comes from the shell adapter's private control channel,
  generation, foreground-process-group check, and submission state. The UI
  revalidates readiness at keyboard dispatch and submission. Unsupported
  shells and unknown or foreground state receive normal PTY input.
- The draft stays intact while submission is pending. Acceptance clears it.
  A stale acknowledgement is ignored, and rejection recovers the exact text.
  Rejected drafts and errors remain visible with editing disabled until the
  shell returns to a confirmed prompt. Editor widget state is keyed by tab
  identity, and every adapter update retains its session identity.
- The Zsh adapter sources the user's startup files and submits through a
  generation-checking ZLE widget and normal accept-line/history handling.
  UTF-8 command files preserve embedded and trailing newlines. The adapter
  acknowledges acceptance or rejection and serializes dispatch with input
  cancellation. Terminal output cannot publish readiness on this channel.
- Completion stays in MYT-030. History-search scope remains open.

### Verification

- All 13 default tests, the seven GUI-free library tests, formatting, and
  Clippy with warnings treated as errors passed on the integrated branch.
  The draft regression check verifies rejection, a stale acknowledgement,
  pending edit protection, and exact multiline UTF-8 retention.
- The isolated KWin Wayland check used a private compositor, XDG directories,
  shell startup file, and history. Clicking plain input inserted a typed
  marker at the chosen caret. Clicking a wrapped row and then the first row
  inserted markers at the corresponding positions. Dragging selected text
  and Ctrl+C copied that selection through the Wayland clipboard.
- Shift+Enter and local editing left both multiline commands unexecuted.
  Enter ran both together, and the next history query returned the complete
  multiline command through normal shell history. Distinct drafts survived
  switching between independent tabs.
- Using the Settings GUI to apply 24 points updated existing and new command
  editors. Caret clicks at the scaled font coordinates still inserted at the
  selected position, and the corresponding PTY reported 40 columns.
- A supplemental native check sourced the configured Oh My Zsh startup file
  with a disposable HISTFILE. App-owned prompt readiness, multiline execution,
  and normal full-buffer history all passed with that configuration.
- The combined foreground/unknown-state and Ctrl+C acceptance matrix is
  recorded in the completed MYT-010 acceptance results.
