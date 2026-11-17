use iced::{
    Element, Length,
    widget::{button, checkbox, column, container, row, scrollable, text, text_input},
};
use mytthy::settings::{Settings, TerminalFontSize};

#[derive(Clone, Debug)]
pub(super) enum Message {
    FontSizeChanged(String),
    Reset,
    ReplaceInvalid(bool),
    Apply,
    Back,
}

pub(super) struct State {
    pub font_size: String,
    pub error: Option<String>,
    pub saved: bool,
    pub saving: bool,
    pub replacement_required: bool,
    pub replace_invalid: bool,
}

impl State {
    pub fn new(settings: &Settings, error: Option<String>, replacement_required: bool) -> Self {
        Self {
            font_size: settings
                .terminal_font_size
                .map_or_else(String::new, |size| size.points().to_string()),
            error,
            saved: false,
            saving: false,
            replacement_required,
            replace_invalid: false,
        }
    }

    pub fn font_size(&self) -> Result<Option<TerminalFontSize>, String> {
        let input = self.font_size.trim();
        if input.is_empty() {
            return Ok(None);
        }
        let points = input.parse::<i64>().map_err(|_| {
            "Enter a whole number between 6 and 72 points, or clear the field to use the default."
                .to_string()
        })?;
        TerminalFontSize::new(points)
            .map(Some)
            .map_err(|error| error.to_string())
    }
}

pub(super) fn view(state: &State) -> Element<'_, Message> {
    let enabled = !state.saving;
    let back = button("Back").on_press_maybe(enabled.then_some(Message::Back));
    let field = text_input("Default (12 points)", &state.font_size)
        .on_input_maybe(enabled.then_some(Message::FontSizeChanged))
        .width(180);
    let reset = button("Use default").on_press_maybe(enabled.then_some(Message::Reset));
    let apply = button(if state.saving { "Saving…" } else { "Apply" }).on_press_maybe(
        (enabled && (!state.replacement_required || state.replace_invalid))
            .then_some(Message::Apply),
    );
    let mut content = column![
        back,
        text("Settings").size(24),
        text("Terminal font size (points)"),
        row![field, reset].spacing(12),
        text("Applies to all terminal tabs."),
    ]
    .spacing(16);

    if let Some(error) = &state.error {
        content = content.push(text(error));
    }
    if state.replacement_required {
        content = content.push(
            checkbox(state.replace_invalid)
                .label("Replace the unreadable settings file when I apply")
                .on_toggle_maybe(enabled.then_some(Message::ReplaceInvalid)),
        );
    }
    if state.saved {
        content = content.push(text("Settings saved."));
    }
    content = content.push(apply);
    scrollable(container(content).padding(24).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_draft_before_apply_and_reset_uses_default() {
        let mut state = State::new(&Settings::default(), None, false);
        assert_eq!(state.font_size().unwrap(), None);
        for points in ["6", "12", "72", " 16 "] {
            state.font_size = points.into();
            assert!(state.font_size().unwrap().is_some());
        }
        for points in ["5", "73", "12.5", "large"] {
            state.font_size = points.into();
            assert!(state.font_size().is_err());
        }
        state.font_size.clear();
        assert_eq!(state.font_size().unwrap(), None);
    }
}
