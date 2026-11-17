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

## Dev notes

- Use the GTK4 + VTE build from MYT-007 as the reference. Check the input and rendering requirements before committing to the production frontend.
