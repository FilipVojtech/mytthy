# MYT-010 Implement context-sensitive Ctrl+C

## Description

Implement FR-5 per session. Verify the acceptance matrix, including a foreground command with text selected and a return to copy behavior once the prompt is idle.

## Acceptance criteria

- Verify the Ctrl+C acceptance matrix with real selection and the Wayland clipboard. Confirm copy resumes at the idle prompt and Ctrl+C forwards while state is unknown.

| Session state | Selection | Ctrl+C behavior |
| --- | --- | --- |
| Idle shell prompt | Text selected | Copy the selection to the system clipboard. |
| Idle shell prompt | No selection | Forward Ctrl+C to the terminal session. |
| Foreground command running | Any | Forward Ctrl+C to the terminal session so the foreground command receives normal terminal input and interrupt handling. |

## Dev notes

Read the [shell-state prototype result](MYT-002-prototype-shell-state-detection.md) for hook loading and event-ordering checks.
