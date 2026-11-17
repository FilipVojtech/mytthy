# MYT-029 Integrate the Rust-native PTY window

## Description

Connect Iced and `alacritty_terminal` in one native Wayland window. Provide the session and rendering entry points needed by the other core terminal tasks. Use the GTK4/VTE implementation from MYT-007 as the behavior reference.

## Acceptance criteria

- Build with Cargo without application-specific system development packages and launch natively on KDE Plasma Wayland. Verify that the shared library still builds with frontend features disabled.
- Exercise keyboard input, shell output, scrolling, PTY resize, launch errors, child exit, and child cleanup on close in the integrated window. Keep blocking PTY I/O off the UI thread.
- Record typing and scrolling responsiveness under sustained output on the reference machine. Resolve rendering blockers before making the new frontend the default.

## Related requirements

- NFR-1 · Performance
- NFR-4 · Native Wayland
- NFR-5 · Dependency independence
- FR-8 · Desktop-specific frontends

## Conclusion

Iced with tiny-skia and Alacritty is now the default native Wayland frontend.
The integrated window passed the PTY lifecycle checks and remained responsive
in the sustained-output test on the reference machine. The GTK4/VTE window
remains available as the optional `mytthy-gtk` reference.

## Dev notes

- Start with the feature set and dependency checks in [MYT-028](MYT-028-evaluate-rust-native-default-frontend.md). Verify the MYT-007 behaviors before switching the default.
- This gates frontend integration for MYT-008 and MYT-009 through MYT-013. MYT-008 already supplies shared shell lookup. MYT-013 already supplies shared terminal font size and persistence.

### Build and runtime

- The release build passed with a fresh Cargo target directory,
  `PKG_CONFIG=/bin/false`, and the Wayland-only Iced feature set. It needed
  the standard Rust and Linux linker toolchain. No application-specific
  development packages were installed. The dependency tree and ELF dynamic
  entries exclude GTK, GNOME VTE, X11 and GPU renderers from the default
  build. The desktop Wayland and xkbcommon libraries are loaded at runtime
  and must be available to the release package under NFR-5.
- The GUI-free library tests, default frontend test, formatting, and Clippy
  with warnings treated as errors passed. The optional GTK4/VTE reference
  also passed `cargo check`.

### Native Wayland checks

- Ran the default release build on the reference machine under a private
  KWin 6.8.3 compositor and a 1024×768 virtual output. KWin had no Xwayland.
  The shell used a disposable Zsh startup file. Keyboard input reached Zsh,
  output rendered, wheel input moved backward through 200 lines of output,
  and maximizing resized the PTY from 24×81 to 36×105. Captures confirmed
  the command output and the older scrollback lines.
- A normal window close reaped the shell, and shell `exit` closed the
  window. Closing during a delayed account lookup canceled launch. An
  invalid account shell path left an error in the window with the
  executable and OS reason. The lookup delay and invalid path used a
  process-local test fixture with no account changes.
- A disposable Zsh configured to ignore SIGHUP exposed a hang on close.
  Closing now gives the PTY time to hang up and uses a Linux pidfd to
  terminate the exact shell if it stays alive for one second. Repeating
  the check closed the window and reaped the child.

### Sustained output on the reference machine

An instrumented release build ran an 800×500 window at 24×81. A PTY fixture
wrote 240 numbered, partially colored lines per second for 18 seconds and
echoed typed keys on a visible status row. Timing starts at client receipt
of the Wayland key or wheel event and ends at the first `wl_surface.commit`
of a buffer showing that key echo or scroll. This endpoint excludes display
presentation and differs from the screenshot-based Tabby baseline.

| Action | Samples | Median | p95 | Maximum |
| --- | ---: | ---: | ---: | ---: |
| Typing with PTY echo | 32 | 16.21 ms | 24.34 ms | 27.46 ms |
| Scrolling visible history | 32 | 15.96 ms | 22.17 ms | 26.13 ms |

### Handoff

- MYT-008's shared account lookup now runs in the PTY supervisor for each
  launch. MYT-008 still needs to verify a changed account shell in a new
  session within a running app and the configured Zsh with Oh My Zsh.
  MYT-012 provides the new-tab entry point for that check.
- MYT-009 through MYT-011 own terminal selection and clipboard, shell prompt
  state, and app-owned editing. MYT-013 owns the GUI for the persisted font
  size that this window reads at launch.
