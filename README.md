# mytthy

mytthy is a terminal emulator project in Rust. The first release targets Fedora Linux with KDE Plasma on Wayland. Planned features include graphical settings, click-to-position editing at the shell prompt, and Ctrl+C behavior that respects both text selection and running commands.

The current binary prints `Hello, world!`. Follow [the project plan](PLAN.md) for requirements, milestones, and tasks.

This project is an exercise in vibe coding, using an AI-first development workflow from planning through implementation and verification.

## Setup

Install the Rust toolchain and Cargo with support for Rust 2024 edition. Run the commands below from the repository root.

## Build and run

- `cargo build` builds the debug binary.
- `cargo run` builds and runs the project.
