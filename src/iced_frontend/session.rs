use super::process::ChildProcess;
use alacritty_terminal::{
    Term,
    event::{Event, EventListener, WindowSize},
    event_loop::{EventLoop, EventLoopSender, Msg},
    grid::Dimensions,
    sync::FairMutex,
    tty::{self, Options, Shell},
};
use iced::futures::channel::mpsc;
use mytthy::session::resolve_shell_executable;
use std::io;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

pub(super) type Terminal = Arc<FairMutex<Term<Listener>>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridSize {
    pub columns: u16,
    pub lines: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

impl GridSize {
    pub fn window_size(self) -> WindowSize {
        WindowSize {
            num_cols: self.columns,
            num_lines: self.lines,
            cell_width: self.cell_width,
            cell_height: self.cell_height,
        }
    }
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        self.lines.into()
    }

    fn columns(&self) -> usize {
        self.columns.into()
    }
}

#[derive(Clone)]
pub(super) enum Update {
    Started,
    Wakeup,
    Terminal(Event),
    Stopped(Result<(), String>),
}

#[derive(Clone)]
pub(super) struct Listener {
    updates: mpsc::UnboundedSender<Update>,
    redraw_pending: Arc<AtomicBool>,
    child_exited: Arc<AtomicBool>,
    control: Arc<Control>,
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let update = match event {
            Event::Wakeup => {
                // One queued wakeup covers all PTY changes before the next UI update.
                if self.redraw_pending.swap(true, Ordering::Relaxed) {
                    return;
                }
                Update::Wakeup
            }
            Event::ChildExit(_) => {
                self.child_exited.store(true, Ordering::Relaxed);
                return;
            }
            event @ (Event::PtyWrite(_)
            | Event::TextAreaSizeRequest(_)
            | Event::ColorRequest(_, _)) => Update::Terminal(event),
            _ => return,
        };
        let _ = self.updates.unbounded_send(update);
    }
}

pub(super) struct Session {
    pub terminal: Terminal,
    pub size: GridSize,
    redraw_pending: Arc<AtomicBool>,
    control: Arc<Control>,
}

impl Session {
    pub fn start(size: GridSize) -> (Self, impl iced::futures::Stream<Item = Update>) {
        let (updates, receiver) = mpsc::unbounded();
        let listener = Listener {
            updates,
            redraw_pending: Arc::new(AtomicBool::new(false)),
            child_exited: Arc::new(AtomicBool::new(false)),
            control: Arc::new(Control::default()),
        };
        let terminal = Arc::new(FairMutex::new(Term::new(
            Default::default(),
            &size,
            listener.clone(),
        )));
        let session = Self {
            terminal: Arc::clone(&terminal),
            size,
            redraw_pending: Arc::clone(&listener.redraw_pending),
            control: Arc::clone(&listener.control),
        };
        let updates = listener.updates.clone();
        let launch = thread::Builder::new()
            .name("terminal supervisor".into())
            .spawn(move || {
                let result =
                    run(terminal, listener.clone(), size).map_err(|error| error.to_string());
                listener.control.stopped.store(true, Ordering::Release);
                let _ = listener.updates.unbounded_send(Update::Stopped(result));
            });
        if let Err(error) = launch {
            let _ = updates.unbounded_send(Update::Stopped(Err(format!(
                "Could not start the terminal worker: {error}"
            ))));
        }
        (session, receiver)
    }

    pub fn wakeup_received(&self) {
        self.redraw_pending.store(false, Ordering::Relaxed);
    }

    pub fn resize(&mut self, size: GridSize) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.terminal.lock().resize(size);
        self.send(Msg::Resize(size.window_size()));
    }

    pub fn close(&self) {
        self.control.closing.store(true, Ordering::SeqCst);
        if !self.control.stopped.load(Ordering::Acquire)
            && let Some(pty) = self.control.pty.get()
        {
            pty.stop();
        }
    }

    pub fn send(&self, message: Msg) {
        if let Some(pty) = self.control.pty.get() {
            // A closed channel races with the supervisor's Stopped notification.
            let _ = pty.sender.send(message);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Default)]
struct Control {
    closing: AtomicBool,
    stopped: AtomicBool,
    pty: OnceLock<PtyControl>,
}

struct PtyControl {
    sender: EventLoopSender,
    child: Option<ChildProcess>,
    stopping: AtomicBool,
}

impl PtyControl {
    fn stop(&self) {
        if !self.stopping.swap(true, Ordering::Relaxed) {
            let _ = self.sender.send(Msg::Shutdown);
            if let Some(child) = &self.child {
                child.kill_after_hangup_grace_period();
            }
        }
    }
}

fn run(terminal: Terminal, listener: Listener, size: GridSize) -> io::Result<()> {
    let shell = resolve_shell_executable();
    if listener.control.closing.load(Ordering::SeqCst) {
        return Ok(());
    }
    let program = shell.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Could not launch {}: shell path is not UTF-8",
                shell.display()
            ),
        )
    })?;
    let options = Options {
        shell: Some(Shell::new(program.to_owned(), Vec::new())),
        env: [
            ("TERM".into(), "xterm-256color".into()),
            ("COLORTERM".into(), "truecolor".into()),
            ("SHELL".into(), program.into()),
        ]
        .into(),
        ..Options::default()
    };
    let pty = tty::new(&options, size.window_size(), 0).map_err(|error| {
        io::Error::new(error.kind(), format!("Could not launch {program}: {error}"))
    })?;
    let child = ChildProcess::open(pty.child().id())
        .map_err(|error| {
            eprintln!("Could not monitor shell exit for {program}: {error}");
            error
        })
        .ok();
    let event_loop =
        EventLoop::new(terminal, listener.clone(), pty, true, false).map_err(|error| {
            io::Error::new(error.kind(), format!("Could not start PTY I/O: {error}"))
        })?;
    let control = listener.control.pty.get_or_init(|| PtyControl {
        sender: event_loop.channel(),
        child,
        stopping: AtomicBool::new(false),
    });
    // The close request and publication can arrive in either order. A close
    // that could not see the published control is handled by the worker here.
    if listener.control.closing.load(Ordering::SeqCst)
        || listener.updates.unbounded_send(Update::Started).is_err()
    {
        control.stop();
    }
    // The returned loop owns the PTY. Drop it here so shell hangup and reaping
    // happen before notifying the UI that the window can close.
    drop(
        event_loop
            .spawn()
            .join()
            .map_err(|_| io::Error::other("PTY worker panicked"))?,
    );
    if !listener.child_exited.load(Ordering::Relaxed)
        && !listener.control.closing.load(Ordering::SeqCst)
    {
        return Err(io::Error::other(
            "PTY worker stopped before the shell exited",
        ));
    }
    Ok(())
}

pub(super) fn log_pty_errors() {
    struct PtyLogger;
    impl log::Log for PtyLogger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Warn
                && metadata.target().starts_with("alacritty_terminal::")
        }

        fn log(&self, record: &log::Record<'_>) {
            if self.enabled(record.metadata()) {
                eprintln!("{}: {}", record.target(), record.args());
            }
        }

        fn flush(&self) {}
    }

    if log::set_logger(&PtyLogger).is_ok() {
        log::set_max_level(log::LevelFilter::Warn);
    }
}
