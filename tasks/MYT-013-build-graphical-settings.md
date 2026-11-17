# MYT-013 Build graphical settings

## Description

Build the graphical settings page. Verify the agreed controls take effect and persist after relaunch.

## Acceptance criteria

- Open the settings page through the GUI. Define the first-release controls. Change each included setting and verify its effect and persistence across relaunch.

## Related requirements

- FR-2 · Graphical settings
- FR-8 · Shared settings across frontends
- NFR-2 · Settings discoverability and usability

## Conclusion

The graphical font-size controls passed current-tab, new-tab, relaunch,
reset, invalid-file replacement, and save-failure checks in the integrated
build on native KWin Wayland. MYT-011 carries font-size inheritance into
the app-owned prompt editor.

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
- The Iced settings view presents a font-size field, Use default, and Apply.
  Reset changes the draft and Apply saves it. The frontend composition owns
  navigation, asynchronous persistence, and updating metrics on all tabs.
  Save failures retain the effective setting and display the path and reason.
  Loading errors use defaults and require an explicit replacement checkbox
  before Apply can replace the unreadable file.
- GUI-free checks passed for missing files, font size save and reset,
  preservation of unknown keys, invalid data, and save failure cleanup.
  On KDE Plasma Wayland, verify opening settings, applying the override
  to all tabs, relaunching with that size, and persisting the return to
  defaults. Check the prompt editor after MYT-011 integration.

### Settings UI verification

- The settings view compiled in an isolated fixture with Iced 0.14. The
  draft-validation regression test passed for blank defaults, allowed
  boundaries, whitespace, out-of-range sizes, fractions, and non-numbers.
- A combined fixture using the MYT-012 session composition passed native
  KWin Wayland checks with disposable XDG settings and Zsh startup files.
  The Settings button opened the page. Applying 24 points changed the
  existing PTY from 20 × 81 cells to 10 × 40. A new tab also reported
  10 × 40, and relaunch restored that size. Use default and Apply removed
  the override and restored 20 × 81.
- The same fixture preserved an unrelated TOML key. An invalid font-size
  value displayed the path and reason and stayed intact when Apply was
  clicked before replacement was selected. Selecting the replacement
  checkbox allowed saving. Replacing the target file with a directory
  caused a visible save error, kept the prior effective font size, and
  allowed returning to the terminal and issuing commands.
- Screenshots of the normal page, saved font, invalid-file confirmation,
  and save failure were inspected. Test scripts, logs, and screenshots
  are under `/tmp/myt013-ui.O9JGNF`.
- The fresh merged build repeated the settings checks successfully.
  App-owned prompt editor inheritance is verified during MYT-011.
