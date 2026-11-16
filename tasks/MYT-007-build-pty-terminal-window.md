# MYT-007 Build PTY terminal window

## Description

Implement a window with a terminal backed by a PTY. Connect input and output, handle resizing, and clean up the child process when the session closes.

## Acceptance criteria

- Verify keyboard input reaches the shell, shell output renders, resize updates terminal dimensions, and closing the session cleans up its child process.
