# MYT-023 Add desktop-specific frontends

## Description

Add optional frontends that follow KDE Plasma and GNOME conventions. Keep the neutral frontend available as the default. Reuse the shared Rust session and settings logic and, where the selected stack supports it, the terminal engine.

## Related requirements

- FR-8 · Desktop-specific frontends
- FR-1, FR-2, FR-3, FR-5, FR-6 and NFR-1 through NFR-3 apply to the relevant frontend behavior.

## Acceptance criteria

- Launch the neutral, Plasma and GNOME frontends independently. Each can open a shell, edit settings and reopen settings saved by the others using the same schema.
- Pass the core shell, input and selection checks in each frontend. Verify GNOME appearance, title bar and keyboard navigation on GNOME Wayland.
- On KDE Plasma Wayland, verify the default title bar and a customized layout. Add, remove, and reorder buttons on both sides. Confirm the Plasma frontend reflects the configured layout and each button works, including Close, Minimize, and Keep Above Others. Focus another overlapping normal window and confirm the terminal stays above it with Keep Above Others enabled and can be covered when disabled. Demonstrate any custom decoration integration needed to pass.
- Verify the shared core builds without either desktop toolkit and the default frontend can be built and run without the optional frontend dependencies.

## Dev notes

Low priority. Absorbs the toolkit and release title-bar checks from MYT-004 and MYT-017. Depends on the frontend boundary evaluated in MYT-003 and the neutral frontend implementation. Choose toolkits and packaging when this task is scheduled. Record frontend-specific terminal code, maintenance costs and verification results here.
