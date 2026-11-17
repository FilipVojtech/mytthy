# MYT-028 Evaluate a Rust-native default frontend

## Description

Evaluate Rust-native GUI crates for the neutral frontend. Prefer a viable stack that builds the graphical app through Cargo with a standard Rust build toolchain and no manually installed application-specific GUI or terminal libraries. Keep shared session and settings policy independent of the GUI toolkit. The existing feasibility demos retain their dependencies.

## Acceptance criteria

- Compare candidates against native Wayland and the terminal input, selection, and PTY requirements. Check their transitive native build dependencies.
- Verify that the shared core builds without frontend features. For a viable candidate, demonstrate that the graphical app builds and launches on a supported Wayland desktop without separately installing application-specific libraries.
- Record the recommended stack and migration scope, or identify the concrete requirements and dependencies that prevent adopting a candidate.

## Related requirements

- NFR-4 · Native Wayland
- NFR-5 · Dependency independence
- FR-8 · Desktop-specific frontends

## Conclusion

Recommend Iced with a Wayland-only tiny-skia frontend and
`alacritty_terminal` for the neutral frontend. The candidate window built and
rendered on native KWin Wayland and exchanged text with the compositor
clipboard. A separate engine check exercised PTY I/O, resize, child exit,
and extraction of a wrapped selection. The shared mytthy library passed its
test with all frontend features disabled.

Land an integrated replacement for the MYT-007 PTY window before switching
the default frontend. Keep the shared session and settings policy independent
of Iced. The existing GTK4/VTE window and feasibility demos supply the behavior
reference for the migration.

## Dev notes

- Use the GTK4 + VTE build from MYT-007 as the reference. Check the input and rendering requirements before committing to the production frontend.

### Candidate comparison

| Stack | Native Wayland and dependencies | Terminal integration |
| --- | --- | --- |
| Iced 0.14.0 + tiny-skia + `alacritty_terminal` 0.26.0 | Disable default features and enable `wayland`, `tiny-skia`, `thread-pool`, and `advanced-shaping`. Winit and softbuffer dynamically load Wayland libraries. Smithay clipboard uses the Wayland data-device protocol, and key mapping loads xkbcommon. The checked Linux build has no GTK, GNOME VTE, X11, or GPU renderer dependency. | Iced supplies the app widgets, text input, pointer events, and clipboard. Alacritty supplies the grid, scrollback, selection model, ANSI parsing, and worker-thread PTY event loop. mytthy needs a terminal widget, terminal key and mouse encoding, and shell adapter integration. COSMIC Terminal demonstrates a related Iced-family and Alacritty pairing. [Iced features](https://docs.rs/crate/iced/0.14.0/features) · [Iced window backend](https://docs.rs/crate/iced_winit/0.14.1/source/Cargo.toml.orig) · [Iced software renderer](https://docs.rs/crate/iced_tiny_skia/0.14.1/source/Cargo.toml.orig) · [COSMIC Terminal](https://github.com/pop-os/cosmic-term/blob/master/src/terminal.rs) |
| eframe + `alacritty_terminal` | eframe exposes Wayland, X11, clipboard, and renderer features separately. An X11-free build needs an explicit feature set. A wgpu or glow build adds a GPU renderer and its platform driver or loader requirements. The transitive build and runtime dependencies for a pinned Wayland configuration remain unverified. | egui exposes key and mouse events, clipboard, and custom painting. The same terminal view, input encoding, and adapter work would be required. eframe was reviewed from its documentation. [eframe features](https://docs.rs/eframe/0.37.0/eframe/#feature-flags) |
| GTK4/VTE (MYT-007) | The current app selects only GDK Wayland. Fedora builds require GTK4 and VTE development packages, and execution requires their runtimes. | VTE already owns the renderer, grid, keyboard encoding, selection, and PTY lifecycle. MYT-006 demonstrated app-owned editing, Zsh handoff, selection and Ctrl+C on this stack. Use these as behavior checks during migration. [MYT-006](MYT-006-prototype-click-to-position.md) · [MYT-007](MYT-007-build-pty-terminal-window.md) |

### Verification on the reference machine

- Built a disposable Cargo fixture on Fedora 45 with Rust 1.100.0. It pinned
  Iced and Alacritty to the versions above, used the listed Iced features
  and disabled Alacritty default features. A fresh target built offline with
  `PKG_CONFIG=/bin/false` after caching the crates. No additional system
  packages were installed. The fixture used the standard Linux linker and
  the existing Wayland and xkbcommon runtimes.
- Inspected the Linux normal and build dependency graph with `cargo tree`.
  The active Wayland backends used runtime loading, and text used
  cosmic-text, fontdb and the Rust fontconfig parser. ELF `NEEDED` entries
  for the GUI were limited to `libgcc_s`, `libm`, `libc` and the system
  loader. The dlopened Wayland client, Wayland cursor, and xkbcommon
  libraries still need to be available from the desktop or release package
  under NFR-5.
- Launched the Iced text-input window under a private KWin 6.8.3 compositor
  with a virtual output, a private D-Bus session and no Xwayland.
  `WAYLAND_DEBUG=1` showed the `xdg_toplevel` and rendered buffer commits.
  Virtual keyboard input reached the widget. Copy was read by an external
  `wl-paste` process and paste reached the widget through the compositor.
- Exercised Alacritty separately with `tty::new` and `EventLoop`. Sent input
  to a disposable `/bin/sh` over a PTY, resized it from 6×8 to 12×8, observed
  the size and shell output in the grid, confirmed successful child exit,
  and extracted a single selection over an 8-column soft wrap.
- Ran `cargo test --offline --locked --no-default-features --lib` against
  mytthy. The shared library test passed.

### Migration scope

1. Connect Iced and Alacritty in a single terminal window with a per-session
   PTY worker and a custom renderer. Verify MYT-007 input, output, resize,
   launch-error, and close/child cleanup checks on KWin. Use damage-driven
   redraw and measure responsiveness under sustained output and scrollback
   before accepting tiny-skia as the release renderer.
2. Add visible grid selection, coordinate mapping through wrapped rows and
   scrollback, and clipboard actions. The text-input clipboard check and
   engine selection check covered the two components separately. MYT-009
   verifies their combined behavior with a real drag in the terminal.
3. Use the shared account shell lookup and settings model. Preserve the
   app-owned buffer and shell adapter split from MYT-006. The current
   `vte.ext.mytthy.shell-state` property is VTE-specific. Define how the
   Alacritty session receives session-scoped prompt generations and
   submission acknowledgments, such as through the adapter control channel.
   Verify confirmed readiness, foreground process group, focus handoff and
   Ctrl+C together in MYT-010 and MYT-011.
