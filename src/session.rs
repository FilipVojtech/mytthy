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

/// Resolve the initial interactive shell from the launch environment.
pub fn shell_executable() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|shell| shell.starts_with('/'))
        .unwrap_or_else(|| "/bin/sh".into())
}

#[cfg(test)]
mod tests {
    use super::ShellState;

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
