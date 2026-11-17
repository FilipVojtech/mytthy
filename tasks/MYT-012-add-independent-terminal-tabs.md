# MYT-012 Add independent terminal tabs

## Description

Add basic tabs with independent sessions so tab switching can be tested and tabs can later serve as the source for drag-to-split.

## Acceptance criteria

- Open and switch among tabs. Check that each tab retains its own shell session and terminal state.

## Related requirements

- NFR-1 · Responsive tab switching.
- NFR-4 · Native Wayland.
- FR-1 · Resolve the configured shell for each new session.

## Conclusion

Independent tabs passed their acceptance checks on native KWin Wayland.
The integrated frontend includes MYT-009 selection and MYT-013 settings,
with per-session shell resolution covered by the MYT-008 fixture checks.

## Dev notes

- Each tab owns its PTY session, terminal grid and scrollback, selection,
  fractional scroll position, shutdown state, and launch error. Worker updates
  carry stable session IDs so a switch or removal cannot reroute them.
- The tab toolbar opens, switches and closes sessions. Ctrl+Shift+T opens a
  session, Ctrl+Shift+W closes the active session, and Ctrl+Tab or
  Ctrl+Shift+Tab switches sessions. The tab strip scrolls horizontally.
- The terminal area reports its actual layout size. All sessions resize when
  that area or the terminal font size changes. Shell exit removes its tab,
  a launch error stays in its own tab, and window close waits for workers.
- A native isolated KWin check passed with the Iced frontend and integrated
  selection and settings modules. Bash and configured Zsh with Oh My Zsh
  retained separate variables, processes and rendered content after switching.
  Resizing changed the PTY dimensions. Exiting Bash preserved the Zsh tab.
  A new tab with an invalid executable displayed its error, and closing it
  preserved Zsh. Closing the window reaped the remaining shell.
- Integration depends on MYT-009's Grid input callback and MYT-013's settings
  view. The composition file also routes asynchronous settings saves and
  applies font metrics only after persistence succeeds. The merged
  build passed all 11 tests, Clippy with warnings denied, formatting, and
  the repeated native tab/session checks. A regression test verifies
  last-session exit while settings are saving closes after the save ends.

### Verification environment

- Toolbox discovery and private D-Bus session launch attempts correlated
  with SELinux domain-transition denials reported by the user. SELinux
  remained enforcing and its policy was unchanged. The final merged app
  passed the native checks in the disposable test environment.
