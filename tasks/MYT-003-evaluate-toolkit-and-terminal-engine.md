# MYT-003 Evaluate toolkit and terminal engine

## Description

Evaluate candidate Rust-compatible GUI toolkits and terminal engines against Wayland clipboard and selection behavior, PTY handling, shell integration, and expected sources of overhead. Record the first prototype choice and hand off live verification.

## Acceptance criteria

- Compare candidates on Wayland selection and clipboard, PTY handling, shell integration, and expected overhead. Record the first prototype choice, rationale, and live checks for follow-up.

## Related requirements

- FR-1, FR-2, FR-3, FR-5, FR-6, NFR-1 and NFR-2 guide the shortlist.
- FR-4 and FR-7 inform extensibility. They are optional.

## Conclusion

Start the neutral frontend prototype with plain GTK4 + VTE. GTK4 provides settings controls, and VTE gives the prototype a terminal widget with selection, rendering and async PTY support. Keep shell policy, session logic and settings independent of toolkit types. VTE retains the terminal view and its state. A later Qt frontend would need its own terminal implementation or a migration to a headless engine.

Use Iced + `alacritty_terminal` as the leading alternative if VTE cannot support the required input handling or the project prioritizes sharing the terminal engine across toolkits. Keep Tauri + xterm.js as a fallback. Published WebView comparisons leave terminal performance on Fedora unresolved.

The evaluation is complete and the stack choice is provisional. MYT-007 owns the live PTY and core/frontend boundary checks, MYT-009 and MYT-010 own Wayland selection, clipboard and Ctrl+C verification, and MYT-006 owns wrapped click-to-caret feasibility. MYT-005 sets performance targets and MYT-015 measures release performance on the reference machine. MYT-023 owns the optional Plasma title-bar checks. No live candidate performance measurements were taken for this evaluation.

## Dev notes

### GUI survey · 2026-11-16

GitHub stars are rounded snapshots of repository interest. They give a rough popularity signal across Rust GUI projects. Production examples provide direct evidence that a stack can host a terminal.

| GUI | Interest or adoption | Fit for mytthy |
| --- | --- | --- |
| GTK4 via gtk-rs | 2.4k stars for the Rust bindings. Fedora ships the GTK4 VTE library. | **First prototype with VTE.** GTK supplies settings widgets, tabs and drag-and-drop. VTE supplies the terminal view. Check input interception before investing in settings. [gtk4-rs](https://github.com/gtk-rs/gtk4-rs) · [Fedora VTE](https://packages.fedoraproject.org/pkgs/vte291/) |
| Qt 6 via CXX-Qt | 1.6k stars for the Rust bridge. Qt is the KDE desktop stack's GUI foundation. | **Plasma integration reference and fallback.** Try a standard decorated Qt window under KWin. Qt Quick can host settings and drag targets. Pair with a Rust terminal engine and budget for a custom terminal item plus the Qt/QML build and Rust bridge. [CXX-Qt](https://github.com/KDAB/cxx-qt) · [KDE stack overview](https://conf.kde.org/event/5/contributions/161/attachments/98/119/kde-stack-overview-training-2023-edition.pdf) |
| Iced | 32.1k stars. COSMIC Terminal provides a working Iced-family example through libcosmic. | **First pure-Rust alternative.** Settings and pane layout fit Iced. Expect custom grid rendering and terminal input. Use COSMIC Terminal as an implementation reference and verify the exact Iced window and clipboard backend on KWin. [Iced](https://github.com/iced-rs/iced) · [COSMIC Terminal](https://github.com/pop-os/cosmic-term/blob/master/src/terminal.rs) |
| GPUI | Newer standalone toolkit with a production terminal and graphical settings in Zed. | **Strong emerging alternative.** Zed demonstrates the exact engine pairing and provides a reference for terminal rendering and settings. Check how much Zed-specific code the integration needs and whether the selected GPUI window mode delegates decorations to KWin. [GPUI](https://gpui.rs/) · [Zed terminal](https://zed.dev/features) · [Zed settings](https://zed.dev/blog/settings-ui) |
| egui / eframe | 32.3k stars. | Immediate-mode controls suit a settings prototype. Terminal grid, input and selection still need a custom view. Verify redraw behavior under sustained output and confirm compositor decoration negotiation with the chosen backend. [egui](https://github.com/emilk/egui) · [winit Wayland backend](https://docs.rs/winit/latest/winit/platform/wayland/index.html) |
| Slint | 24.6k stars. | Declarative controls suit settings. Validate the terminal rendering extension point and the selected window backend before investing in a custom grid. Review the license options when choosing distribution terms. [Slint](https://github.com/slint-ui/slint) |
| Tauri / Dioxus Desktop | 112.9k and 39.8k stars respectively across projects with broader scope than Linux desktop. | WebView settings and xterm.js give a quick path to a terminal UI. The app owns the Rust PTY and byte transport across the frontend boundary. Measure the Linux WebKitGTK process tree and output latency, and test clipboard behavior. Keep as a fallback. [Tauri](https://github.com/tauri-apps/tauri) · [Dioxus](https://github.com/DioxusLabs/dioxus) · [xterm.js](https://xtermjs.org/docs/api/terminal/classes/terminal/) |

### Terminal engine and application responsibilities

| Engine | Selection and clipboard | PTY and shell integration | Overhead to measure |
| --- | --- | --- | --- |
| VTE for GTK4 | Widget owns grid rendering and drag selection. Rust bindings expose selection state, selection-change events and clipboard actions. Verify copy, paste and PRIMARY on Wayland. | Async spawn and PTY APIs allow an explicit shell per session. Test whether the app can intercept Ctrl+C and access enough shell state and grid coordinates for wrapped click-to-caret. VTE shell signals are worth testing against the actual MYT-002 hook protocol. | GTK/VTE startup, process-tree PSS, widget redraw and large scrollback. [VTE terminal API](https://world.pages.gitlab.gnome.org/Rust/vte4-rs/stable/latest/docs/vte4/struct.Terminal.html) |
| `alacritty_terminal` | Engine provides grid and selection primitives. The GUI maps pointer coordinates to cells, draws selection and handles Wayland clipboard and PRIMARY. | Engine provides a PTY and event loop. Keep PTY work off the UI thread and expose spawn failures. The app can integrate prompt markers and cell mapping explicitly. Zed and COSMIC Terminal demonstrate this division of responsibilities. | GUI renderer, glyph caches, grid-to-frame work and scrollback. [Alacritty](https://github.com/alacritty/alacritty) · [COSMIC Terminal integration](https://github.com/pop-os/cosmic-term/blob/master/src/terminal.rs) |
| xterm.js | Terminal view exposes selection and input callbacks. Verify the WebView and host clipboard path on Wayland. | Rust host owns the PTY, shell resolution and process state. Test how raw output markers and UI key events remain ordered across the bridge. | WebView startup and PSS, PTY-to-WebView batching and output-to-frame latency. [xterm.js API](https://xtermjs.org/docs/api/terminal/classes/terminal/) |

FR-1 and FR-5 remain application policy in every stack. Resolve the configured login shell on each new session and test with the existing Zsh and Oh My Zsh setup. Carry forward MYT-002's prompt hooks and foreground-process-group guard, including the foreground builtin case. Check Ctrl+C with an idle selection, idle without selection, and a foreground command with selection. FR-6 needs a plain click to map through wrapped terminal cells to an editable ZLE position while a drag still selects text. FR-7 would build on ZLE navigation. FR-4 needs the layout to move a live session without respawning its PTY.

### Prototype order and outstanding verification

1. Confirm that shortlisted toolkits can open a usable window on Fedora KDE Wayland. MYT-023 owns the customized Plasma title-bar checks.
2. Prototype GTK4 + VTE. Exercise real Wayland selection and clipboard, the MYT-002 Ctrl+C behavior and the wrapped-input API needed by MYT-006. If VTE cannot expose the control required by FR-5 or FR-6, compare a custom view using `alacritty_terminal` in the highest-ranked viable GUI. GPUI is an additional option.
3. Compare viable stacks on the MYT-001 reference machine with matching window size, shell setup, workloads and measurement endpoints. Record startup to a usable toolbar, full process-tree idle PSS, input and scroll response, tab switching and settings opening. Use a shared observation method and measure its overhead. MYT-001 screenshot capture dominated its typing and scrolling figures.

Desktop research is complete. Candidate overhead, live Wayland clipboard behavior remain unverified. Confirm the prototype choice after the follow-up checks.

### KDE, GNOME and other desktop environments · 2026-11-16

- **Plain GTK4 + VTE is the first cross-desktop prototype.** On GNOME, GTK4 supplies familiar controls. Libadwaita adds GNOME-specific widgets and automatically follows supported dark, accent and high-contrast preferences. Start with plain GTK4 to avoid coupling the settings UI and window chrome to GNOME conventions. KDE's GTK configurator syncs desktop preferences such as fonts, icons, cursors and dark mode. Its Breeze GTK repository describes a GTK2/3 theme, so that integration alone gives no assurance of Breeze GTK4 controls. Check the actual Fedora GTK4 widgets. [Libadwaita appearance](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/styles-and-appearance.html) · [KDE GTK configurator](https://github.com/KDE/kde-gtk-config) · [Breeze GTK](https://github.com/KDE/breeze-gtk)
- **Qt 6 + CXX-Qt favors Plasma.** KDE provides a Qt Quick desktop style. Qt Quick upstream defaults to Fusion on Linux, so check which style the packaged build actually picks up. GNOME-style Wayland decorations are available in modern Qt, with support dependent on the shipped Qt version and packaging. For full GNOME widget styling, the historical QGnomePlatform bridge is archived and uses private Qt headers. That is a serious maintenance drawback for relying on that bridge. A Qt window can visually fit Plasma well while remaining usable with a neutral style elsewhere. The terminal renderer and Rust/Qt bridge remain implementation costs. [KDE Qt Quick style](https://github.com/KDE/qqc2-desktop-style) · [Qt Quick styles](https://doc.qt.io/qt-6/qtquickcontrols-styles.html) · [Flatpak desktop integration](https://github.com/flatpak/flatpak-docs/blob/master/docs/desktop-integration.rst) · [QGnomePlatform](https://github.com/FedoraQt/QGnomePlatform)
- **A neutral UI with desktop preferences is portable.** Iced, GPUI or a WebView frontend can use the XDG Settings portal for color scheme, accent, contrast and reduced motion where the backend exports them. Use fallbacks for missing keys. The app still supplies its controls and layout, and decorations need a separate compositor or toolkit path. For Xfce, Cinnamon or other desktops, check the settings that the actual session exposes. [XDG Settings portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
- **Two desktop-native frontends can share one Rust core.** Qt/KDE and GTK/libadwaita can each use their own widgets and window conventions. Sharing PTY and shell policy still leaves two settings views, window integrations, accessibility and input paths, and package/test matrices. Using VTE for only the GTK frontend also splits terminal behavior between implementations. That maintenance cost is substantial for the first release.

For MYT-023, compare titles and window actions separately from settings controls and theme preferences. Verify Plasma NFR-3 on Fedora KDE Wayland. During any GNOME follow-up, check the actual title bar, system light/dark and accent where available, default fonts and scaling, and keyboard focus in settings. Record the package/runtime used, since a sandbox may need runtime theme extensions. Research verified available mechanisms. Live desktop behavior remains unverified.

### Tauri overhead

On Linux Tauri uses the installed WebKitGTK runtime with a Rust host. It avoids bundling Chromium, but still runs a browser renderer. A Tauri terminal adds PTY-to-WebView transport and xterm.js rendering. Its installer size tells us little about process-tree memory or input-to-frame latency. Delta Chat's published comparison reported a considerably smaller Debian package with higher observed RAM for its Tauri build. A Tauri issue reports an Ubuntu PSS comparison where the default Tauri app used slightly less memory than the default Electron app, and different results for loaded web applications. These workloads do not establish mytthy's performance. [Tauri process model](https://v2.tauri.app/concept/process-model/) · [Delta Chat comparison](https://github.com/deltachat/deltachat-tauri/issues/20) · [Tauri memory benchmark discussion](https://github.com/tauri-apps/tauri/issues/5889)

Keep Tauri + xterm.js as a fallback. If it advances, measure a release build with the same shell, scrollback and workloads as the GTK4 + VTE prototype. Count WebKit subprocesses and the shell in idle PSS. In addition to startup, typing and scroll latency, measure input-to-visible-echo while sustained output arrives, since the bridge and renderer contend under that load. Record the Fedora WebKitGTK version and keep the measurement method consistent across candidates. No Tauri terminal performance result has been established on the reference machine.

### Frontend boundary

Use one neutral frontend for the first release. Keep shell resolution, shell-state tracking, Ctrl+C policy, session lifecycle and settings data in Rust modules that compile independently of the GUI toolkit. The frontend owns event handling, pixel-to-cell mapping, rendering, clipboard access and window decorations. A headless terminal engine can keep the grid, scrollback and selection model reusable across toolkits.

GTK4 + VTE can share the application policy, but the VTE widget owns the grid, selection and rendering. A later Qt frontend would need a different terminal view and engine or a migration to a common headless engine. Treat that as a stack-selection trade-off. If cross-toolkit terminal reuse is important, prefer a headless engine such as `alacritty_terminal` if the prototype passes the input, performance and Wayland checks.

During MYT-007, verify the proposed core compiles and runs session and settings checks without linking the GUI. Exercise shell launch, input, output, resize and shell-state transitions through the same boundary the neutral frontend uses. Record any state VTE retains in the widget and what a second toolkit would have to replace. Begin with concrete Rust modules and add common interfaces where the prototype demonstrates a need.
