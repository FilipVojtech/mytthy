# MYT-024 Package for Flatpak

## Description

Provide Flatpak as an additional distribution format. Preserve native Wayland behavior and the user's normal host shell session across the package boundary.

## Acceptance criteria

- Build the package from a Flatpak manifest, install it locally, and launch it on Fedora KDE Plasma using the Wayland socket without X11 or fallback-X11 permissions.
- Verify that the package starts the configured host shell with its expected startup configuration and access to host commands. Exercise PTY resizing, interrupts, and cleanup when a session closes.
- Verify the shipped features in the installed package, including settings persistence, mouse selection and caret placement, clipboard access, and all Ctrl+C cases.
- Document installation, the host-shell mechanism, and the permissions it requires. Explain the sandbox implications of running commands on the host.

## Related requirements

- FR-1 · Configured default shell
- FR-3 · Mouse selection
- FR-5 · Context-sensitive Ctrl+C
- FR-6 · Click to position
- NFR-4 · Native Wayland

## Dev notes

- Use the first release from MYT-018 as the packaging baseline. Validate host-shell execution early, including the terminal behavior across the sandbox boundary, before committing to the manifest permissions. Flatpak supports host execution through `flatpak-spawn --host`, which runs commands outside the sandbox.
