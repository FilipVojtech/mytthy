use std::ffi::{CStr, OsStr};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;

/// Advisory state reported by shell integration. Terminal output can emit these
/// markers, so decisions about input handling require independent validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellState {
    #[default]
    Unknown,
    Prompt,
    Foreground,
}

impl ShellState {
    pub fn from_marker(marker: Option<&str>) -> Self {
        match marker {
            Some("prompt") => Self::Prompt,
            Some("foreground") => Self::Foreground,
            _ => Self::Unknown,
        }
    }
}

/// The frontend applies one action to the active session for each Ctrl+C.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtrlCAction {
    CopySelection,
    ClearDraftAndForward,
    Forward,
}

/// Choose copy or interrupt using independently confirmed prompt readiness.
///
/// The caller must revalidate readiness at dispatch, including the adapter's
/// current primary-prompt generation, foreground process group, and absence of
/// an in-flight submission. Advisory terminal output markers alone cannot
/// establish readiness. Select the active editor or terminal selection before
/// calling this policy, and consume the event before widget default handling.
/// Forward sends normal Ctrl+C input to the PTY, preserving terminal interrupt
/// configuration. ClearDraftAndForward also clears the local app-owned draft.
pub fn ctrl_c_action(prompt_ready: bool, has_selection: bool) -> CtrlCAction {
    match (prompt_ready, has_selection) {
        (true, true) => CtrlCAction::CopySelection,
        (true, false) => CtrlCAction::ClearDraftAndForward,
        (false, _) => CtrlCAction::Forward,
    }
}

/// Resolve the initial interactive shell from the launch environment.
pub fn shell_executable() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|shell| shell.starts_with('/'))
        .unwrap_or_else(|| "/bin/sh".into())
}

/// Resolve the real user's configured shell for a new session.
///
/// Call this off the GUI thread for each launch. The account lookup follows
/// NSS configuration and can block on a directory service.
pub fn resolve_shell_executable() -> PathBuf {
    account_shell().unwrap_or_else(|| shell_executable().into())
}

fn account_shell() -> Option<PathBuf> {
    // SAFETY: getuid has no preconditions.
    let uid = unsafe { libc::getuid() };
    let mut buffer = vec![0; 1024];
    let mut account = MaybeUninit::<libc::passwd>::uninit();
    // Bound retries in case a faulty NSS provider always returns ERANGE.
    const MAX_BUFFER_SIZE: usize = 1024 * 1024;

    loop {
        let mut result = ptr::null_mut();
        // SAFETY: The output slots and buffer are valid for the duration of
        // the call. The returned strings borrow the buffer.
        let status = unsafe {
            libc::getpwuid_r(
                uid,
                account.as_mut_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && buffer.len() < MAX_BUFFER_SIZE {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if status != 0 || result.is_null() {
            return None;
        }

        // SAFETY: On success a non-null result points to the populated record.
        let shell = unsafe { (*result).pw_shell };
        if shell.is_null() {
            return None;
        }
        // SAFETY: pw_shell points to a NUL-terminated string in the live buffer.
        let path = PathBuf::from(OsStr::from_bytes(
            unsafe { CStr::from_ptr(shell) }.to_bytes(),
        ));
        return path.is_absolute().then_some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::{CtrlCAction, ShellState, ctrl_c_action};

    #[test]
    fn copies_only_at_a_confirmed_idle_prompt() {
        let cases = [
            (true, true, CtrlCAction::CopySelection),
            (true, false, CtrlCAction::ClearDraftAndForward),
            (false, true, CtrlCAction::Forward),
            (false, false, CtrlCAction::Forward),
        ];
        for (ready, selection, expected) in cases {
            assert_eq!(ctrl_c_action(ready, selection), expected);
        }
    }

    #[test]
    fn maps_shell_markers_to_advisory_state() {
        assert_eq!(ShellState::from_marker(Some("prompt")), ShellState::Prompt);
        assert_eq!(
            ShellState::from_marker(Some("foreground")),
            ShellState::Foreground
        );
        assert_eq!(ShellState::from_marker(None), ShellState::Unknown);
        assert_eq!(ShellState::from_marker(Some("other")), ShellState::Unknown);
    }
}
