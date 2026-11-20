/// App-owned command text and its submission transaction, independent of a GUI.
#[derive(Default)]
pub struct CommandDraft {
    text: String,
    pending: Option<u64>,
}

impl CommandDraft {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Widget adapters replace the draft after applying local editing actions.
    /// Pending text stays intact until the shell acknowledges its generation.
    pub fn replace(&mut self, text: String) {
        if !self.pending() {
            self.text = text;
        }
    }

    pub fn begin_submission(&mut self, generation: u64) -> Option<String> {
        if self.pending() || self.text.contains('\0') {
            return None;
        }
        self.pending = Some(generation);
        Some(self.text.clone())
    }

    /// Ignore acknowledgements from a previous generation. Rejection recovers
    /// the exact draft, including UTF-8 and newlines, for another attempt.
    pub fn acknowledge(&mut self, generation: u64, accepted: bool) -> bool {
        if self.pending != Some(generation) {
            return false;
        }
        self.pending = None;
        if accepted {
            self.text.clear();
        }
        true
    }

    pub fn clear(&mut self) {
        if !self.pending() {
            self.text.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandDraft;

    #[test]
    fn rejection_and_stale_acknowledgement_preserve_the_draft() {
        let mut draft = CommandDraft::default();
        draft.replace("print café\nprint done".into());
        assert_eq!(
            draft.begin_submission(2).as_deref(),
            Some("print café\nprint done")
        );
        draft.replace("replacement".into());
        assert!(draft.begin_submission(3).is_none());
        assert!(!draft.acknowledge(1, true));
        assert!(draft.acknowledge(2, false));
        assert_eq!(draft.text(), "print café\nprint done");
        assert!(draft.begin_submission(3).is_some());
        assert!(draft.acknowledge(3, true));
        assert_eq!(draft.text(), "");
    }
}
