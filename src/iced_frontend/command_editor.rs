use iced::{
    Element, Font, Length,
    keyboard::{Key, key::Named},
    widget::{column, text, text_editor},
};
use mytthy::command_input::CommandDraft;

#[derive(Clone, Debug)]
pub(super) enum Message {
    Action(text_editor::Action),
    Submit,
    Interrupt,
}

#[derive(Default)]
pub(super) struct Editor {
    pub draft: CommandDraft,
    pub content: text_editor::Content,
    pub error: Option<String>,
}

impl Editor {
    pub fn perform(&mut self, action: text_editor::Action) {
        if self.draft.pending() {
            return;
        }
        self.content.perform(action);
        self.draft.replace(self.content.text());
        self.error = None;
    }

    pub fn clear(&mut self) {
        self.draft.clear();
        self.content = text_editor::Content::with_text(self.draft.text());
        self.error = None;
    }

    pub fn acknowledge(&mut self, generation: u64, result: Result<(), String>) {
        if self.draft.acknowledge(generation, result.is_ok()) {
            if result.is_ok() {
                self.content = text_editor::Content::new();
            }
            self.error = result.err();
        }
    }
}

pub(super) fn view<'a>(
    editor: &'a Editor,
    font_size: f32,
    terminal_selection: bool,
    ready: impl Fn() -> bool + 'a,
) -> Element<'a, Message> {
    let pending = editor.draft.pending();
    let ready_now = ready();
    let mut input = text_editor(&editor.content)
        .id("command-editor")
        .font(Font::MONOSPACE)
        .size(font_size)
        .wrapping(iced::widget::text::Wrapping::Glyph)
        .padding(8)
        .height(Length::Fixed((font_size * 5.0).clamp(100.0, 220.0)))
        .key_binding(move |mut press| {
            if pending || !ready() {
                return None;
            }
            let modifiers = press.modifiers;
            if modifiers.control() {
                if press.key.as_ref() == Key::Named(Named::Tab) {
                    return None;
                }
                if let Key::Character(key) = press.key.as_ref() {
                    if key.eq_ignore_ascii_case("c") && !modifiers.shift() {
                        return Some(text_editor::Binding::Custom(Message::Interrupt));
                    }
                    if modifiers.shift()
                        && (key.eq_ignore_ascii_case("t") || key.eq_ignore_ascii_case("w"))
                    {
                        return None;
                    }
                    if terminal_selection && modifiers.shift() && key.eq_ignore_ascii_case("c") {
                        return None;
                    }
                }
            }
            if press.key.as_ref() == Key::Named(Named::Enter) && !modifiers.shift() {
                return Some(text_editor::Binding::Custom(Message::Submit));
            }
            // The app owns prompt input even after selecting transcript text.
            // Mouse actions still control the native editor's caret/selection.
            press.status = text_editor::Status::Focused { is_hovered: false };
            text_editor::Binding::from_key_press(press)
        });
    if !pending && ready_now {
        input = input.on_action(Message::Action);
    }
    let mut panel = column![input];
    if pending {
        panel = panel.push(text("Submitting…"));
    } else if !ready_now {
        panel = panel.push(text("Draft retained until the shell prompt is ready."));
    }
    if let Some(error) = &editor.error {
        panel = panel.push(text(error));
    }
    panel.into()
}
