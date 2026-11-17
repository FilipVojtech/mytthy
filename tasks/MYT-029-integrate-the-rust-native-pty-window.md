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

## Dev notes

- Start with the feature set and dependency checks in [MYT-028](MYT-028-evaluate-rust-native-default-frontend.md). Verify the MYT-007 behaviors before switching the default.
- This gates frontend integration for MYT-008 and MYT-009 through MYT-013. MYT-008 already supplies shared shell lookup. MYT-013 already supplies shared terminal font size and persistence.
