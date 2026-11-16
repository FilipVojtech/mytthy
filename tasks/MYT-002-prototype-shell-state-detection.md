# MYT-002 Prototype shell state detection

## Description

Prototype shell-state detection with the existing Zsh and Oh My Zsh setup. Check idle prompts, foreground commands, and the return to the prompt after a command exits or is interrupted. Verify that the PTY still receives Ctrl+C when shell state is unknown.

## Acceptance criteria

- Verify idle and running states, forwarding when unknown, and restored copy behavior after normal exit and interruption. Cover an external command and a Zsh builtin.

## Dev notes

A disposable PTY harness ran the existing interactive Zsh and Oh My Zsh configuration. A ZLE `line-init` hook marked the prompt idle, and a `preexec` hook marked command execution. Checking the PTY foreground process group provided an additional guard for external jobs. The hooks were needed for a foreground Zsh builtin, which ran in the shell's own process group. The harness passed all three FR-5 cases, forwarded Ctrl+C before shell state was known, and restored copy behavior after normal exit and interruption. Selection and clipboard were simulated. The app still needs to verify real selection and clipboard behavior on Wayland, how it loads the hooks, and ordered handling of markers and key events around prompt transitions.
