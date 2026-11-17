# MYT-008 Launch the configured shell

## Description

Launch the configured login shell for new sessions and verify FR-1 after changing the account's default shell.

## Acceptance criteria

- Change the account default shell and verify a new session uses it without a new profile. Verify Zsh with Oh My Zsh.

## Related requirements

- FR-1 · Use the configured default shell for new sessions.
- NFR-5 · Dependency independence.

## Dev notes

- The shared resolver calls Linux NSS by real user ID on each invocation. It
  prefers an absolute account shell, falling back to the existing absolute
  `SHELL` or `/bin/sh` if no usable account shell is returned. It retains path
  bytes in a `PathBuf`. The already-locked `libc` crate provides bindings to
  the system C library. It adds no application-specific native dependency.
- GUI-free tests passed. A resolver probe matched the current account shell
  returned by `getent` with both an absolute stale `SHELL` and an invalid
  relative `SHELL`.
- MYT-029 calls the resolver off the GUI thread for each launch in the default
  Iced frontend. Its isolated Wayland checks covered cancellation during the
  account lookup and surfaced an invalid executable. Verify a new tab picks
  up an account shell change with the app still open and verify the configured
  Zsh with Oh My Zsh on native Wayland. MYT-012 provides the new-tab entry point.
- MYT-012 verified the new-tab entry point on native isolated KWin. A
  disposable NSS lookup fixture changed the reported account shell from Bash
  to Zsh while the app remained open, with a stale inherited SHELL. The new
  tab launched Zsh and loaded the configured Oh My Zsh. The existing Bash
  tab kept its process and state. An invalid account executable surfaced a
  per-tab launch error. This check used a private lookup fixture. A real
  account-default change through the system account tools remains unverified.
