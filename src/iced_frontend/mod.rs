mod grid;
mod input;
mod process;
mod session;
mod settings_view;

use self::{
    grid::{Grid, Metrics, palette_rgb},
    session::{Session, Update},
};
use alacritty_terminal::{event::Event as TerminalEvent, event_loop::Msg, grid::Scroll};
use iced::{
    Element, Event, Length, Size, Subscription, Task, Theme, event, keyboard,
    mouse::{self, ScrollDelta},
    widget::{button, center, column, row, scrollable, sensor, text},
    window,
};
use mytthy::settings::Settings;

pub(super) fn run() -> iced::Result {
    session::log_pty_errors();
    let (settings, load_error) =
        match Settings::config_path().and_then(|path| Settings::load(&path)) {
            Ok(settings) => (settings, None),
            Err(error) => {
                eprintln!("{error}");
                (Settings::default(), Some(error.to_string()))
            }
        };
    let window_size = Size::new(settings.window_width as f32, settings.window_height as f32);
    iced::application(
        move || App::new(settings.clone(), load_error.clone()),
        App::update,
        App::view,
    )
    .title("mytthy")
    .theme(Theme::Dark)
    .window(window::Settings {
        size: window_size,
        min_size: Some(Size::new(240.0, 140.0)),
        exit_on_close_request: false,
        ..window::Settings::default()
    })
    .subscription(App::subscription)
    .run()
}

type SessionId = u64;

#[derive(Clone)]
enum Message {
    Opened(window::Id, Size),
    GridResized(Size),
    CloseRequested(window::Id),
    Focused(bool),
    Key(keyboard::Event),
    Scroll(ScrollDelta),
    Session(SessionId, Update),
    NewTab,
    OpenSettings,
    Settings(settings_view::Message),
    SettingsSaved(Settings, Result<(), String>),
    Activate(SessionId),
    CloseTab(SessionId),
    Input(SessionId, Vec<u8>),
}

struct Tab {
    id: SessionId,
    session: Option<Session>,
    closing: bool,
    scroll_remainder: f32,
    error: Option<String>,
}

struct App {
    settings: Settings,
    settings_state: settings_view::State,
    settings_open: bool,
    metrics: Metrics,
    window: Option<window::Id>,
    tabs: Vec<Tab>,
    active: Option<SessionId>,
    next_id: SessionId,
    grid_size: Size,
    focused: bool,
    closing: bool,
}

impl App {
    fn new(settings: Settings, load_error: Option<String>) -> Self {
        let font_points = settings
            .terminal_font_size
            .map_or(12.0, |size| f32::from(size.points()));
        let settings_state =
            settings_view::State::new(&settings, load_error.clone(), load_error.is_some());
        Self {
            settings,
            settings_state,
            settings_open: false,
            metrics: Metrics::new(font_points * 96.0 / 72.0),
            window: None,
            tabs: Vec::new(),
            active: None,
            next_id: 1,
            grid_size: Size::new(800.0, 450.0),
            focused: false,
            closing: false,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status, id| match event {
            Event::Window(window::Event::Opened { size, .. }) => Some(Message::Opened(id, size)),
            Event::Window(window::Event::CloseRequested) => Some(Message::CloseRequested(id)),
            Event::Window(window::Event::Focused) => Some(Message::Focused(true)),
            Event::Window(window::Event::Unfocused) => Some(Message::Focused(false)),
            Event::Keyboard(event) if status == event::Status::Ignored => Some(Message::Key(event)),
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if status == event::Status::Ignored =>
            {
                Some(Message::Scroll(delta))
            }
            _ => None,
        })
    }

    fn open_tab(&mut self) -> Task<Message> {
        if self.closing {
            return Task::none();
        }
        let id = self.next_id;
        self.next_id += 1;
        let (session, updates) = Session::start(self.metrics.grid_size(self.grid_size));
        self.tabs.push(Tab {
            id,
            session: Some(session),
            closing: false,
            scroll_remainder: 0.0,
            error: None,
        });
        self.active = Some(id);
        // Worker messages always retain their session identity through tab switches/removal.
        Task::run(updates, move |update| Message::Session(id, update))
    }

    fn remove_tab(&mut self, id: SessionId) -> Task<Message> {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return Task::none();
        };
        self.tabs.remove(index);
        if self.active == Some(id) {
            self.active = self
                .tabs
                .get(index.min(self.tabs.len().saturating_sub(1)))
                .map(|tab| tab.id);
        }
        if self.tabs.is_empty()
            && !self.settings_state.saving
            && let Some(window) = self.window
        {
            return window::close(window);
        }
        Task::none()
    }

    fn close_tab(&mut self, id: SessionId) -> Task<Message> {
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
            tab.closing = true;
            if let Some(session) = &tab.session {
                session.close();
            } else {
                return self.remove_tab(id);
            }
        }
        Task::none()
    }

    fn send_input(&self, id: SessionId, bytes: Vec<u8>) {
        if let Some(tab) = self.tabs.iter().find(|tab| tab.id == id && !tab.closing)
            && let Some(session) = &tab.session
        {
            let mut terminal = session.terminal.lock();
            terminal.scroll_display(Scroll::Bottom);
            drop(terminal);
            session.send(Msg::Input(bytes.into()));
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Opened(window, size) => {
                self.window = Some(window);
                self.grid_size = Size::new(size.width, (size.height - 40.0).max(1.0));
                return self.open_tab();
            }
            Message::GridResized(size) => {
                self.grid_size = size;
                for tab in &mut self.tabs {
                    if let Some(session) = &mut tab.session {
                        session.resize(self.metrics.grid_size(size));
                    }
                }
            }
            Message::CloseRequested(id) => {
                self.closing = true;
                self.tabs.retain(|tab| tab.session.is_some());
                if self.tabs.is_empty() && !self.settings_state.saving {
                    return window::close(id);
                }
                for tab in &mut self.tabs {
                    tab.closing = true;
                    if let Some(session) = &tab.session {
                        session.close();
                    }
                }
            }
            Message::Focused(focused) => self.focused = focused,
            Message::OpenSettings => {
                self.settings_open = true;
            }
            Message::Settings(message) => return self.update_settings(message),
            Message::SettingsSaved(candidate, result) => {
                self.settings_state.saving = false;
                match result {
                    Ok(()) => {
                        self.settings = candidate;
                        self.settings_state.saved = true;
                        self.settings_state.error = None;
                        self.settings_state.replacement_required = false;
                        self.settings_state.replace_invalid = false;
                        let points = self
                            .settings
                            .terminal_font_size
                            .map_or(12.0, |size| f32::from(size.points()));
                        self.metrics = Metrics::new(points * 96.0 / 72.0);
                        for tab in &mut self.tabs {
                            if let Some(session) = &mut tab.session {
                                session.resize(self.metrics.grid_size(self.grid_size));
                            }
                        }
                    }
                    Err(error) => self.settings_state.error = Some(error),
                }
                if self.tabs.is_empty()
                    && let Some(id) = self.window
                {
                    return window::close(id);
                }
            }
            Message::NewTab => return self.open_tab(),
            Message::Activate(id) => {
                if self.tabs.iter().any(|tab| tab.id == id) {
                    self.active = Some(id);
                }
            }
            Message::CloseTab(id) => return self.close_tab(id),
            Message::Input(id, bytes) => self.send_input(id, bytes),
            Message::Key(event) if !self.settings_open => {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = &event {
                    if modifiers.control() && modifiers.shift() {
                        if key.as_ref() == keyboard::Key::Character("T")
                            || key.as_ref() == keyboard::Key::Character("t")
                        {
                            return self.open_tab();
                        }
                        if (key.as_ref() == keyboard::Key::Character("W")
                            || key.as_ref() == keyboard::Key::Character("w"))
                            && let Some(id) = self.active
                        {
                            return self.close_tab(id);
                        }
                    }
                    if modifiers.control()
                        && key.as_ref() == keyboard::Key::Named(keyboard::key::Named::Tab)
                    {
                        if let Some(index) =
                            self.tabs.iter().position(|tab| Some(tab.id) == self.active)
                        {
                            let next = if modifiers.shift() {
                                (index + self.tabs.len() - 1) % self.tabs.len()
                            } else {
                                (index + 1) % self.tabs.len()
                            };
                            self.active = Some(self.tabs[next].id);
                        }
                        return Task::none();
                    }
                }
                if let Some(tab) = self
                    .tabs
                    .iter()
                    .find(|tab| Some(tab.id) == self.active && !tab.closing)
                    && let Some(session) = &tab.session
                {
                    let bytes = input::encode(&event, *session.terminal.lock().mode());
                    if let Some(bytes) = bytes {
                        self.send_input(tab.id, bytes);
                    }
                }
            }
            Message::Scroll(delta) if !self.settings_open => {
                if let Some(tab) = self
                    .tabs
                    .iter_mut()
                    .find(|tab| Some(tab.id) == self.active && !tab.closing)
                    && let Some(session) = &tab.session
                {
                    let lines = match delta {
                        ScrollDelta::Lines { y, .. } => y * 3.0,
                        ScrollDelta::Pixels { y, .. } => y / self.metrics.line_height(),
                    };
                    tab.scroll_remainder += lines;
                    let delta = tab.scroll_remainder.trunc();
                    tab.scroll_remainder -= delta;
                    if delta != 0.0 {
                        session
                            .terminal
                            .lock()
                            .scroll_display(Scroll::Delta(delta as i32));
                    }
                }
            }
            Message::Key(_) | Message::Scroll(_) => {}
            Message::Session(id, update) => return self.session_update(id, update),
        }
        Task::none()
    }

    fn update_settings(&mut self, message: settings_view::Message) -> Task<Message> {
        if self.settings_state.saving {
            return Task::none();
        }
        match message {
            settings_view::Message::Back => self.settings_open = false,
            settings_view::Message::FontSizeChanged(value) => {
                self.settings_state.font_size = value;
                self.settings_state.saved = false;
            }
            settings_view::Message::Reset => {
                self.settings_state.font_size.clear();
                self.settings_state.saved = false;
            }
            settings_view::Message::ReplaceInvalid(value) => {
                self.settings_state.replace_invalid = value
            }
            settings_view::Message::Apply => {
                if self.settings_state.replacement_required && !self.settings_state.replace_invalid
                {
                    return Task::none();
                }
                let font_size = match self.settings_state.font_size() {
                    Ok(size) => size,
                    Err(error) => {
                        self.settings_state.error = Some(error);
                        return Task::none();
                    }
                };
                let mut candidate = self.settings.clone();
                candidate.terminal_font_size = font_size;
                self.settings_state.saving = true;
                self.settings_state.saved = false;
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                let saved_settings = candidate.clone();
                let started = std::thread::Builder::new()
                    .name("settings save".into())
                    .spawn(move || {
                        let result = Settings::config_path()
                            .and_then(|path| saved_settings.save(&path))
                            .map_err(|error| error.to_string());
                        let _ = sender.send(result);
                    });
                if let Err(error) = started {
                    self.settings_state.saving = false;
                    self.settings_state.error =
                        Some(format!("Could not start settings save: {error}"));
                    return Task::none();
                }
                return Task::perform(
                    async move {
                        receiver.await.unwrap_or_else(|_| {
                            Err("Settings save worker stopped unexpectedly".into())
                        })
                    },
                    move |result| Message::SettingsSaved(candidate.clone(), result),
                );
            }
        }
        Task::none()
    }

    fn session_update(&mut self, id: SessionId, update: Update) -> Task<Message> {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) else {
            return Task::none();
        };
        let Some(session) = &mut tab.session else {
            return Task::none();
        };
        match update {
            Update::Started if !tab.closing => {
                session.send(Msg::Resize(session.size.window_size()))
            }
            Update::Started => {}
            Update::Wakeup => session.wakeup_received(),
            Update::Terminal(event) => {
                let response = match event {
                    TerminalEvent::PtyWrite(text) => text,
                    TerminalEvent::TextAreaSizeRequest(format) => {
                        format(session.size.window_size())
                    }
                    TerminalEvent::ColorRequest(index, format) => {
                        let terminal = session.terminal.lock();
                        format(palette_rgb(terminal.renderable_content().colors, index))
                    }
                    _ => return Task::none(),
                };
                session.send(Msg::Input(response.into_bytes().into()));
            }
            Update::Stopped(result) => {
                tab.session = None;
                if !tab.closing
                    && let Err(error) = result
                {
                    eprintln!("{error}");
                    tab.error = Some(error);
                } else {
                    return self.remove_tab(id);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.closing {
            return center(text("Closing terminals…")).into();
        }
        if self.settings_open {
            return settings_view::view(&self.settings_state).map(Message::Settings);
        }
        let tabs = row(self.tabs.iter().map(|tab| {
            row![
                button(text(format!("Terminal {}", tab.id)))
                    .on_press(Message::Activate(tab.id))
                    .style(if self.active == Some(tab.id) {
                        button::primary
                    } else {
                        button::secondary
                    }),
                button("×").on_press(Message::CloseTab(tab.id))
            ]
            .into()
        }))
        .spacing(4);
        let toolbar = row![
            scrollable(tabs)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default()
                ))
                .width(Length::Fill),
            button("+").on_press(Message::NewTab),
            button("Settings").on_press(Message::OpenSettings)
        ]
        .spacing(8)
        .padding(4);
        let content: Element<'_, Message> =
            match self.tabs.iter().find(|tab| Some(tab.id) == self.active) {
                Some(tab) if tab.closing => center(text("Closing terminal…")).into(),
                Some(tab) => match &tab.session {
                    Some(session) => Element::new(Grid {
                        metrics: self.metrics,
                        terminal: &session.terminal,
                        focused: self.focused,
                        on_input: Box::new(move |bytes| Message::Input(tab.id, bytes)),
                    }),
                    None => center(text(tab.error.as_deref().unwrap_or("Starting shell…")))
                        .padding(16)
                        .into(),
                },
                None => center(text("Starting shell…")).into(),
            };
        column![
            toolbar,
            sensor(content)
                .on_show(Message::GridResized)
                .on_resize(Message::GridResized)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finishing_settings_save_closes_after_last_session_exits() {
        let mut app = App::new(Settings::default(), None);
        app.window = Some(window::Id::unique());
        app.settings_state.saving = true;
        assert!(!app.closing);
        let task = app.update(Message::SettingsSaved(Settings::default(), Ok(())));
        assert_eq!(task.units(), 1);
        assert!(!app.settings_state.saving);
    }
}
