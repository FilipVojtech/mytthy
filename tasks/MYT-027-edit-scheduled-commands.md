# MYT-027 Edit scheduled commands

## Description

Provide a graphical editor for the user's cron jobs with a human-readable schedule preview.

## Acceptance criteria

- Create, edit, and remove a scheduled command through the UI. Verify its schedule in the host crontab and confirm a test command runs with the app closed.
- Validate schedules before saving and verify unrelated crontab entries survive edits.

## Related requirements

- FR-11 · Scheduled commands

## Dev notes

- Verify availability of the host cron service and decide how to expose the execution shell, working directory, and environment. Assess host access for the Flatpak build in MYT-024.
