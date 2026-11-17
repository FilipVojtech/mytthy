# mytthy

mytthy is a terminal emulator project in Rust. The first release targets Fedora Linux with KDE Plasma on Wayland. Planned features include graphical settings, click-to-position editing at the shell prompt, and Ctrl+C behavior that respects both text selection and running commands.

Follow [the project plan](PLAN.md) for requirements, milestones, and tasks.

This project is an exercise in vibe coding, using an AI-first development workflow from planning through implementation and verification.

## Setup

On Fedora, we encourage using a dedicated [Toolbx](https://containertoolbx.org/install/) container for mytthy, or an equivalent such as [Distrobox](https://distrobox.it/). Install project-specific RPM dependencies with `dnf` inside the container to keep them separate from the host.

```sh
toolbox create mytthy
toolbox enter mytthy
```

When you no longer need the environment, exit it and run `toolbox rm mytthy` to remove the container and its installed packages. Deleting the checkout alone leaves the container in place.

## Build and run

Use Rust 1.92 or later, GTK 4.18 or later, and VTE 0.78 or later. Install the
build dependencies on Fedora or in the Toolbx, then run the application from
a Wayland session.

```sh
sudo dnf install rust cargo gcc pkgconf-pkg-config gtk4-devel vte291-gtk4-devel
cargo build --locked
cargo run --locked
```

Run `cargo check --locked --no-default-features --lib` to check the shared
library without GTK4 or VTE.

## Run the MYT-006 demos

The demos live under `src/` on the `prototype` branch. Both use Fedora packages `gtk4`, `vte291-gtk4` (VTE 0.78 or later),
`python3-gobject`, and `zsh`. Run either command from the repository root in a
Plasma Wayland session.

- **App-owned editor** — `python3 src/app-owned-input/main.py` loads your
  Zsh configuration with temporary history. Enter submits and Shift+Enter adds a
  line.
- **Earlier ZLE demo** — `python3 src/click-to-position/main.py` uses a
  disposable Zsh configuration and supports printable ASCII.

See the [MYT-006 Dev notes](tasks/MYT-006-prototype-click-to-position.md#dev-notes)
for checks to try in each demo.
