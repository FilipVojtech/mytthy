mod grid;
mod input;
mod process;
mod session;

use self::{
    grid::{Grid, Metrics, palette_rgb},
    session::{Session, Update},
};
use alacritty_terminal::{event::Event as TerminalEvent, event_loop::Msg, grid::Scroll};
use iced::{
    Element, Event, Length, Size, Subscription, Task, Theme, event, keyboard,
    mouse::{self, ScrollDelta},
    widget::{center, text},
    window,
};
use mytthy::settings::Settings;

pub(super) fn run() -> iced::Result {
    session::log_pty_errors();
    let settings = Settings::config_path().and_then(|path| Settings::load(&path));
    let settings = settings.unwrap_or_else(|error| {
        eprintln!("{error}");
        Settings::default()
    });
    let window_size = Size::new(settings.window_width as f32, settings.window_height as f32);
    let font_size = settings
        .terminal_font_size
        .map_or(12.0, |size| f32::from(size.points()));
    iced::application(move || App::new(font_size), App::update, App::view)
        .title("mytthy")
        .theme(Theme::Dark)
        .window(window::Settings {
            size: window_size,
            min_size: Some(Size::new(120.0, 80.0)),
            exit_on_close_request: false,
            ..window::Settings::default()
        })
        .subscription(App::subscription)
        .run()
}

#[derive(Clone)]
enum Message {
    Opened(window::Id, Size),
    Resized(Size),
    CloseRequested(window::Id),
    Focused(bool),
    Key(keyboard::Event),
    Scroll(ScrollDelta),
    Session(Update),
}

struct App {
    metrics: Metrics,
    window: Option<window::Id>,
    session: Option<Session>,
    focused: bool,
    closing: bool,
    scroll_remainder: f32,
    error: Option<String>,
}

impl App {
    fn new(font_points: f32) -> Self {
        Self {
            metrics: Metrics::new(font_points * 96.0 / 72.0),
            window: None,
            session: None,
            focused: false,
            closing: false,
            scroll_remainder: 0.0,
            error: None,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status, id| match event {
            Event::Window(window::Event::Opened { size, .. }) => Some(Message::Opened(id, size)),
            Event::Window(window::Event::Resized(size)) => Some(Message::Resized(size)),
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Opened(window, size) => {
                self.window = Some(window);
                let (session, updates) = Session::start(self.metrics.grid_size(size));
                self.session = Some(session);
                return Task::run(updates, Message::Session);
            }
            Message::Resized(size) => {
                if let Some(session) = &mut self.session {
                    session.resize(self.metrics.grid_size(size));
                }
            }
            Message::CloseRequested(id) => {
                self.closing = true;
                if let Some(session) = &self.session {
                    session.close();
                } else {
                    return window::close(id);
                }
            }
            Message::Focused(focused) => self.focused = focused,
            Message::Key(event) => {
                if let Some(session) = &self.session {
                    let mut terminal = session.terminal.lock();
                    if let Some(bytes) = input::encode(&event, *terminal.mode()) {
                        if terminal.grid().display_offset() != 0 {
                            terminal.scroll_display(Scroll::Bottom);
                        }
                        drop(terminal);
                        session.send(Msg::Input(bytes.into()));
                    }
                }
            }
            Message::Scroll(delta) => {
                if let Some(session) = &self.session {
                    let lines = match delta {
                        ScrollDelta::Lines { y, .. } => y * 3.0,
                        ScrollDelta::Pixels { y, .. } => y / self.metrics.line_height(),
                    };
                    self.scroll_remainder += lines;
                    let delta = self.scroll_remainder.trunc();
                    self.scroll_remainder -= delta;
                    if delta != 0.0 {
                        session
                            .terminal
                            .lock()
                            .scroll_display(Scroll::Delta(delta as i32));
                    }
                }
            }
            Message::Session(update) => return self.session_update(update),
        }
        Task::none()
    }

    fn session_update(&mut self, update: Update) -> Task<Message> {
        let Some(session) = &mut self.session else {
            return Task::none();
        };
        match update {
            Update::Started if !self.closing => {
                session.send(Msg::Resize(session.size.window_size()));
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
                self.session = None;
                if !self.closing
                    && let Err(error) = result
                {
                    eprintln!("{error}");
                    self.error = Some(error);
                } else if let Some(id) = self.window {
                    return window::close(id);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.closing {
            return center(text("Closing terminal…")).into();
        }
        if let Some(session) = &self.session {
            return Element::new(Grid {
                metrics: self.metrics,
                terminal: &session.terminal,
                focused: self.focused,
            });
        }

        let status = self.error.as_deref().unwrap_or("Starting shell…");
        center(text(status).width(Length::Shrink))
            .padding(16)
            .into()
    }
}
