# Terminal emulator plan

Build a responsive terminal emulator with graphical settings and familiar mouse and keyboard behavior. The initial environment is Fedora Linux with KDE Plasma on Wayland, using Zsh with Oh My Zsh.

## Current work

No roadmap task is in progress.

## Functional requirements

| ID | Requirement | Priority |
| --- | --- | --- |
| FR-1 | New sessions use the user's configured default shell after a shell change without requiring a new terminal profile. Verify with Zsh and Oh My Zsh. | Core |
| FR-2 | Provide a graphical settings page. | Core |
| FR-3 | Support click-and-drag text selection in terminal output. | Core |
| FR-4 | Drag an existing tab to the desired position to create a split pane there. | Nice to have |
| FR-5 | Handle Ctrl+C according to the shell state and text selection. | Core |
| FR-6 | Left-click within the editable shell input to place the caret at the clicked text position, including on wrapped input lines. A drag still selects text. | Core |
| FR-7 | Support keyboard navigation forward and backward across word and CamelHumps boundaries in shell input, such as between `parse`, `Shell`, and `Input` in `parseShellInput`. | Nice to have |

## Non-functional requirements

- **NFR-1 · Performance.** Keep startup, typing, scrolling, tab switching, and opening settings responsive on the reference machine, the developer's current computer. Use the same machine for baseline and prototype measurements. Record timings and memory use independently of the machine's hardware identifiers.
- **NFR-2 · Usability.** Make settings easy to find and use. Launch the configured shell without requiring manual profile setup, and make mouse text selection and clipboard behavior predictable.
- **NFR-3 · KDE Plasma window integration.** The optional Plasma frontend in MYT-023 respects user-configured Plasma window behavior on Wayland. At minimum, the title bar must reflect the user's choice and order of buttons on either side, including Close, Minimize, and Keep Above Others. Each button must perform the corresponding window action. The toolkit may delegate decorations to KWin or draw its own. For toolkit-drawn decorations, the toolkit must either honor these settings directly or expose enough control over button layout and window actions for the app to implement the integration. Track additional Plasma integrations as they are identified.

## Roadmap

The Rust crate is a starter binary. The first release targets local terminal use on Fedora KDE Wayland. Each milestone lists its tasks and exit criterion. Task files contain acceptance criteria and development notes.

### 1. Establish feasibility and performance targets

- [x] [MYT-001 Measure Tabby baseline](tasks/MYT-001-measure-tabby-baseline.md)
- [x] [MYT-002 Prototype shell state detection](tasks/MYT-002-prototype-shell-state-detection.md)
- [ ] [MYT-004 Verify Plasma decorations in toolkit candidates](tasks/MYT-004-verify-plasma-decorations-in-toolkit-candidates.md)
- [ ] [MYT-005 Set performance targets and settings scope](tasks/MYT-005-set-performance-targets-and-settings-scope.md)
- [ ] [MYT-006 Prototype click to position](tasks/MYT-006-prototype-click-to-position.md)

**Exit criterion.** The prototype demonstrates all three FR-5 cases on the reference machine. The plan records the stack choice, settings scope, and performance targets.

### 2. Build the core terminal

- [ ] [MYT-007 Build PTY terminal window](tasks/MYT-007-build-pty-terminal-window.md)
- [ ] [MYT-008 Launch the configured shell](tasks/MYT-008-launch-the-configured-shell.md)
- [ ] [MYT-009 Support mouse selection and clipboard](tasks/MYT-009-support-mouse-selection-and-clipboard.md)
- [ ] [MYT-010 Implement context-sensitive Ctrl+C](tasks/MYT-010-implement-context-sensitive-ctrl-c.md)
- [ ] [MYT-011 Implement click to position](tasks/MYT-011-implement-click-to-position.md)
- [ ] [MYT-012 Add independent terminal tabs](tasks/MYT-012-add-independent-terminal-tabs.md)
- [ ] [MYT-013 Build graphical settings](tasks/MYT-013-build-graphical-settings.md)

**Exit criterion.** The app runs locally and satisfies FR-1, FR-2, FR-3, FR-5, and FR-6.

### 3. Verify and release the first build

- [ ] [MYT-014 Verify terminal lifecycle](tasks/MYT-014-verify-terminal-lifecycle.md)
- [ ] [MYT-015 Verify release performance](tasks/MYT-015-verify-release-performance.md)
- [ ] [MYT-016 Verify release usability](tasks/MYT-016-verify-release-usability.md)
- [ ] [MYT-017 Verify release Plasma integration](tasks/MYT-017-verify-release-plasma-integration.md)
- [ ] [MYT-018 Package and verify first release](tasks/MYT-018-package-and-verify-first-release.md)

**Exit criterion.** The first build installs and passes core acceptance checks and performance targets on Fedora KDE Wayland.

### 4. Add optional features as prioritized

- [ ] [MYT-019 Preview and create splits from tabs](tasks/MYT-019-preview-and-create-splits-from-tabs.md)
- [ ] [MYT-020 Move live sessions into splits](tasks/MYT-020-move-live-sessions-into-splits.md)
- [ ] [MYT-021 Measure split pane responsiveness](tasks/MYT-021-measure-split-pane-responsiveness.md)
- [ ] [MYT-022 Add CamelHumps navigation](tasks/MYT-022-add-camelhumps-navigation.md)

**Exit criterion.** Each selected feature passes its checks. If FR-4 is selected, live split sessions also meet the agreed performance targets.

## Decisions

- Planned features can appear in their intended menus, toolbars, or settings pages as disabled controls. Label features under active development **WIP** (Work in Progress) and features awaiting implementation **TBA** (To Be Added). Enable each control when its acceptance criteria pass.

- Use the current computer as the performance reference because Tabby's UI feels sluggish on it.
- Preserve foreground command interrupt behavior even when terminal text is selected.
- Defer drag-to-split from the first milestone. It remains a nice-to-have.
