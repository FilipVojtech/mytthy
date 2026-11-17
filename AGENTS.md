# Working on mytthy

## Task workflow

- Resolve uncertainty using the available evidence. If the request is still unclear or you are unsure how to proceed, ask a focused clarifying question before acting on an assumption.

- Before planning or implementing, read [PLAN.md](PLAN.md), check **Current work** to coordinate with other agents, and read the linked task. Use requirement IDs and acceptance criteria to define scope.
- Keep one Markdown file per task in `tasks/`, named `MYT-###-short-issue-name.md`. Use the next unused number and a lowercase hyphenated title. Keep the issue code stable.
- Start each task file with a level-one heading containing the issue code and title. Use these level-two headings in order: **Description**, **Acceptance criteria**, **Related requirements**, **Conclusion** when closing the task, and **Dev notes** when needed. List testable checks under Acceptance criteria and requirement IDs under Related requirements. Summarize the outcome and any handoff under Conclusion. Use Dev notes for dependencies, decisions, verification results, and remaining work.
- Link each task under its milestone in `PLAN.md`. When starting work, list the task link and its focus under **Current work**, one entry per task. Update the focus or name the blocker as work changes. Keep handoff details in the task file.
- Record what passed and what remains in the task file. Check off the milestone entry when acceptance criteria pass. Remove the **Current work** entry when work completes or pauses. When empty, write “No roadmap task is in progress.”

## Implementation and verification

- Keep diffs focused, follow nearby conventions, and limit lockfile changes to dependencies needed for the task.
- Keep blocking terminal I/O off the UI thread. Surface shell launch and PTY failures with enough detail to diagnose them.
- Use comments for contracts, invariants, and rationale that are hard to infer from code.
- Before changing saved settings or shell integration, identify the formats and behaviors to preserve. Verify them alongside the new behavior.
- Check existing coverage and run the smallest test set that exercises the change. Run relevant manual checks, including affected terminal interactions on KDE Plasma Wayland.

## Documentation

- Keep setup, build, and run instructions in `README.md`. Update `PLAN.md` when requirements, milestones, or lasting product decisions change. Record decision rationale there.
- Update this file when guidance becomes stale or agents need a lasting rule. Document repository conventions and invariants, including platform constraints, generated files, and behavior to preserve.
- Add a short component map as the architecture takes shape. Name entry points and allowed dependencies between layers. Feasibility prototypes live under `src/` on the `prototype` branch.
- Refer to the performance reference machine by role. Keep exact hardware specs out of tracked files.
- Write instructions directly. Omit phrases like “once it exists” unless they tell contributors when or how to act.

## Component map

- `src/main.rs` is the GTK4 entry point. It uses VTE for the terminal widget and PTY and calls shared session policy through the library.
- `src/lib.rs` exposes shared modules such as `src/session.rs`. Keep them independent of GTK4 and VTE so they build without the default frontend feature.
