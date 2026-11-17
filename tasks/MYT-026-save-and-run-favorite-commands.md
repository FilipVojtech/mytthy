# MYT-026 Save and run favorite commands

## Description

Save a command from a command block to a persistent favorites list. Run a favorite by clicking it in the list.

## Acceptance criteria

- Favorite a command from its output block and verify it remains listed after relaunch.
- Click a favorite with a supported shell at an idle prompt. Verify it runs once through the normal submission path.
- Verify that clicking a favorite while a foreground program is running does not inject input into that program.

## Related requirements

- FR-10 · Favorite commands
- FR-9 · Command output blocks

## Dev notes

- Depends on MYT-025 for saving from a block. Decide whether favorites reuse the original working directory or run in the current directory.
