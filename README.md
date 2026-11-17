# mytthy

mytthy is a terminal emulator project in Rust. The first release targets Fedora Linux with KDE Plasma on Wayland. Planned features include graphical settings, click-to-position editing at the shell prompt, and Ctrl+C behavior that respects both text selection and running commands.

Follow [the project plan](PLAN.md) for requirements, milestones, and tasks.

This project is an exercise in vibe coding, using an AI-first development workflow from planning through implementation and verification.

## Build and run

Use Rust 1.92 or later and a standard Linux linker. On Fedora, install the
toolchain and run from a KDE Plasma Wayland session.

```sh
sudo dnf install rust cargo gcc
cargo build --locked
cargo run --locked
```

The default frontend uses Iced and Alacritty with software rendering. It
loads the Wayland client and cursor libraries and xkbcommon from the
desktop runtime. Use `cargo check --locked --no-default-features --lib` to
check the shared library with frontend features disabled.

### GTK4/VTE reference

The original PTY window is available as `mytthy-gtk`. It uses GTK 4.18 or
later and VTE 0.78 or later. Install the development packages on Fedora or
in a dedicated [Toolbx](https://containertoolbx.org/install/) container.

```sh
sudo dnf install rust cargo gcc pkgconf-pkg-config gtk4-devel vte291-gtk4-devel
cargo run --locked --no-default-features --features gtk-frontend --bin mytthy-gtk
```

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
