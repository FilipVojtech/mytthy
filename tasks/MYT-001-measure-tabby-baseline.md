# MYT-001 Measure Tabby baseline

## Description

Measure Tabby's cold startup, idle memory, and responsiveness while typing, scrolling, switching tabs, and opening settings on the reference machine. Record the workload and how each metric was measured.

## Related requirements

- NFR-1 · Performance

## Acceptance criteria

- Record reproducible workloads, measurement methods, and results for cold startup, idle memory, typing, scrolling, tab switching, and opening settings.

## Dev notes

### Reference environment

Measured on 2026-11-16 on the reference machine running Fedora KDE Plasma 44 in a Wayland session. Tabby 1.0.238 bundles Electron 43.7.0 and Chromium 150.0.7871.250. Tabby used the 800 × 600 default renderer viewport and the machine's configured Zsh with Oh My Zsh. The compositor backend used by Tabby itself was unverified.

Launch `/opt/Tabby/tabby --no-sandbox --remote-debugging-port=0 --user-data-dir=<fresh-directory>` with no Tabby process from the preceding run. The temporary directory contains a minimal `config.yaml` with `version: 8`, `enableWelcomeTab: false`, and `appearance.frame: native`. The launch uses the same sandbox flag as the installed desktop entry. Leave the OS file cache intact. Use a fresh process and temporary profile for each run. Start with the Tabby window in the foreground and keep other input out of the run.

### Workloads and endpoints

Connect to the local Chrome DevTools Protocol (CDP) page target as soon as it appears. Use a monotonic clock outside the app. For DOM readiness, poll for the stated element to have a nonzero bounding width after two nested `requestAnimationFrame` callbacks. Dispatch mouse input to the center of the actual control.

| Metric | Procedure |
| --- | --- |
| Cold startup | Start the clock immediately before spawning Tabby. Stop when the toolbar button containing `[data-icon=plus]` is ready. The endpoint is a usable window toolbar. The shell starts in the next workload. |
| Idle memory | Click the plus button to open one local shell tab. After the active `.xterm-screen` is ready, wait five seconds. Sum `Pss` from `/proc/<pid>/smaps_rollup` for the Tabby process and all its descendants, including the shell. Convert KiB to MiB. |
| Typing | Send ten lowercase keys, `abcdefghij`, at the idle prompt, with 100 ms between each completed measurement and the next input. For each key, snapshot the active `.xterm-screen` before input and time from dispatching key down to the first different renderer screenshot. |
| Scrolling | Clear the pending line with Ctrl+U, enter `seq 1 5000`, and wait 1.5 seconds. At the center of the terminal, dispatch ten upward wheel events of −480 CSS pixels vertically. Time from each dispatch to the first different screenshot of the active terminal, with a 150 ms wait after each measured event. Check that the viewport has scrollback and has moved away from the bottom. |
| Tab switching | Open a second local shell tab and wait 700 ms. Alternate clicks between the first and second `tab-header` ten times, waiting 150 ms after each measurement. Time from pointer release dispatch until the destination header is ready with the `.active` class. |
| Opening settings | Click the toolbar gear with settings closed. Time from pointer release dispatch until `tab-body.active settings-tab` is ready. |

Use `Page.captureScreenshot` with PNG output, the active terminal's bounding rectangle, scale 1, and `captureBeyondViewport: false`. Compare the returned PNGs. Before typing, record the duration of five captures of the idle terminal. Typing and scrolling include screenshot capture and polling overhead. The tab and settings results measure renderer readiness. All four UI metrics omit physical input and monitor presentation. The timings are comparable when using the same endpoints and instrumentation.

### Results

Each column is a fresh-process run. Typing, scrolling, and switching report the median of ten events per run. The capture control reports the median of five idle screenshots per run.

| Metric | Run 1 | Run 2 | Run 3 |
| --- | ---: | ---: | ---: |
| Cold startup (ms) | 1,774 | 1,786 | 1,855 |
| Idle memory (MiB PSS) | 546 | 559 | 559 |
| Typing (ms) | 189 | 189 | 188 |
| Scrolling (ms) | 183 | 191 | 190 |
| Tab switching (ms) | 30 | 31 | 22 |
| Opening settings (ms) | 73 | 68 | 69 |
| Idle screenshot capture (ms) | 184 | 190 | 183 |

The terminal viewport moved away from the bottom in all three runs. One tab switch took 185 ms in run 2. The screenshot control took about as long as the typing and scrolling measurements, so the reported values give a coarse time to observe a change. Use a lower-overhead observation method before setting typing and scrolling latency targets in MYT-005.

All six workloads ran on the reference desktop and have recorded measurements.
