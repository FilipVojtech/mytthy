# MYT-011 Implement app-owned command input

## Description

Implement the app-owned prompt editor from MYT-006. Keep buffer editing shared and put prompt detection, submission, and shell-specific behavior behind shell adapters.

## Acceptance criteria

- Click several positions on plain and wrapped input. Confirm typing inserts there and dragging still selects text.
- Edit multiline commands locally. Verify that Enter submits the full buffer through the normal shell command path, that embedded newlines execute nothing before submission, and that commands appear in shell history according to shell policy.
- Forward input to the PTY during foreground execution or unknown shell state and restore app editing at a confirmed idle prompt. Verify the FR-5 Ctrl+C cases.

## Related requirements

- FR-3 · Mouse selection
- FR-5 · Context-sensitive Ctrl+C
- FR-6 · App-owned command input

## Dev notes

- Use a shared completion interface. If a common declarative format covers the supported sources, keep its parser in shared app code. Put format-specific, shell-specific, or application-specific completion handling in specialized provider modules. Verify which formats and mechanisms are available before choosing a parser.
- Completion coverage and history search scope are open product decisions.
