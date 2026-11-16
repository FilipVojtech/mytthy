# MYT-007 Build PTY terminal window

## Description

Implement a window with a terminal backed by a PTY. Connect input and output, handle resizing, and clean up the child process when the session closes.

## Acceptance criteria

- Verify keyboard input reaches the shell, shell output renders, resize updates terminal dimensions, and closing the session cleans up its child process.

- Verify shared session and settings logic can build without linking the GUI. Exercise shell launch, input, output, resize and shell-state transitions through the boundary used by the neutral frontend. Record which terminal state the toolkit owns.
