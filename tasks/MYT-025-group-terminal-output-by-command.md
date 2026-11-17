# MYT-025 Group terminal output by command

## Description

Show each submitted command with its own output block at supported shell prompts. Keep a live terminal view for interactive programs.

## Acceptance criteria

- Run successive commands with output, errors, and no output. Verify each command appears once with its associated output, including for multiline input.
- Exercise an interactive command and verify terminal input and display still work.

## Related requirements

- FR-9 · Command output blocks
- FR-6 · App-owned command input

## Dev notes

- Build on MYT-011 and use shell adapter events to identify command boundaries. Decide how to display asynchronous output whose source cannot be identified.
