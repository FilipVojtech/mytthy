# MYT-013 Build graphical settings

## Description

Build the graphical settings page. Verify the agreed controls take effect and persist after relaunch.

## Acceptance criteria

- Open the settings page through the GUI. Define the first-release controls. Change each included setting and verify its effect and persistence across relaunch.

## Related requirements

- FR-2 · Graphical settings
- FR-8 · Shared settings across frontends
- NFR-2 · Settings discoverability and usability

## Dev notes

- The first release exposes one control, **Terminal font size**. The default
  uses the frontend's normal monospace size. Allow a whole-number override
  between 6 and 72 points and a return to the default. Applying the setting
  updates open tabs, new tabs, and the app-owned prompt editor. Keep the
  existing 800 × 500 initial window size as an internal default. Shell
  selection follows the account configuration under MYT-008.
- The shared settings file is `$XDG_CONFIG_HOME/mytthy/settings.toml`. If
  `XDG_CONFIG_HOME` is absent, empty, or relative, use
  `$HOME/.config/mytthy/settings.toml`. The optional integer
  `terminal_font_size_points` key specifies the override. A missing file or
  key uses the defaults. There is no previous settings file to migrate.
- The toolkit-independent Rust library loads the settings and saves by
  replacing the file after a successful write to a temporary file in the
  same directory. It preserves unrecognized TOML keys from the loaded
  document. The `toml` crate is also an existing GTK4/VTE build-time
  dependency and adds no native libraries.
- After MYT-028 selects the frontend, add the settings entry and page. Keep
  the current setting if Apply fails and present the save error. On read
  errors, show the error and use the defaults for the running app. Preserve
  an invalid file until the user explicitly chooses to replace it.
- GUI-free checks passed for missing files, font size save and reset,
  preservation of unknown keys, invalid data, and save failure cleanup.
  On KDE Plasma Wayland, verify opening settings, applying the override
  to all tabs, relaunching with that size, and persisting the return to
  defaults. Check the prompt editor after MYT-011 integration.
