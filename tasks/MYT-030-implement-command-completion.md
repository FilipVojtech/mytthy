# MYT-030 Implement command completion

## Description

Add completion to the app-owned command editor from MYT-011. Use a shared completion interface with specialized providers for shell, format, or application behavior.

## Acceptance criteria

- Define the initial completion sources, supported shells, and interaction before implementation.
- Verify which published completion formats and mechanisms the selected sources use. Use a shared parser where a common declarative format covers them and specialized providers where needed.
- Verify that candidates correspond to the current buffer and caret position. Accepting a candidate replaces the intended text and preserves surrounding input.
- Verify completion can be requested, navigated, accepted, and dismissed without submitting a command or sending completion input to a foreground program.

## Related requirements

- FR-6 · App-owned command input
- NFR-4 · Native Wayland
- NFR-5 · Dependency independence

## Dev notes

- Depends on MYT-011 for the editable buffer and shell-state boundary.
- Completion coverage remains an open product decision. Record the agreed sources and examples before implementation.
