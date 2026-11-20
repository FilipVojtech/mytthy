use alacritty_terminal::event_loop::{EventLoopSender, Msg};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{DirBuilderExt, OpenOptionsExt},
            net::{UnixListener, UnixStream},
        },
    },
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
pub(super) enum Event {
    Changed,
    Submitted {
        generation: u64,
        result: Result<(), String>,
    },
}

pub(super) struct Prepared {
    root: PathBuf,
    listener: UnixListener,
    wake: String,
}

impl Prepared {
    pub fn new() -> io::Result<Self> {
        let mut random = [0_u8; 16];
        File::open("/dev/urandom")?.read_exact(&mut random)?;
        let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        let root = std::env::temp_dir().join(format!("mytthy-input-{token}"));
        fs::DirBuilder::new().mode(0o700).create(&root)?;
        let result = (|| {
            fs::write(root.join(".zshenv"), include_str!("zsh/.zshenv"))?;
            fs::write(root.join(".zshrc"), include_str!("zsh/.zshrc"))?;
            fs::write(
                root.join("integration.zsh"),
                include_str!("zsh/integration.zsh"),
            )?;
            let listener = UnixListener::bind(root.join("socket"))?;
            listener.set_nonblocking(true)?;
            Ok(Self {
                root: root.clone(),
                listener,
                wake: format!("\x1b[mytthy-{token}~"),
            })
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(root);
        }
        result
    }

    pub fn environment(&self) -> Vec<(String, String)> {
        let mut env = vec![
            ("ZDOTDIR".into(), self.root.to_string_lossy().into_owned()),
            (
                "MYTTHY_INPUT_SOCKET".into(),
                self.root.join("socket").to_string_lossy().into_owned(),
            ),
            ("MYTTHY_INPUT_WAKE".into(), self.wake.clone()),
        ];
        if let Some(path) = std::env::var_os("ZDOTDIR") {
            env.push((
                "MYTTHY_USER_ZDOTDIR".into(),
                path.to_string_lossy().into_owned(),
            ));
        }
        env
    }

    pub fn start(
        self,
        master: File,
        pid: u32,
        sender: EventLoopSender,
        notify: impl Fn(Event) + Send + 'static,
    ) -> io::Result<Handle> {
        let (requests, receiver) = mpsc::channel();
        let shared = Arc::new(Shared {
            master,
            pid: pid as i32,
            state: Mutex::new(State::default()),
            stopped: AtomicBool::new(false),
        });
        let worker = Arc::clone(&shared);
        thread::Builder::new()
            .name("Zsh input adapter".into())
            .spawn(move || {
                self.run(&worker, receiver, sender, &notify);
                worker.stopped.store(true, Ordering::Release);
                let pending = worker.state.lock().unwrap().inflight.take();
                worker.state.lock().unwrap().prompt = None;
                if let Some(generation) = pending {
                    notify(Event::Submitted {
                        generation,
                        result: Err(
                            "Shell input channel closed before submission was acknowledged".into(),
                        ),
                    });
                }
                notify(Event::Changed);
            })?;
        Ok(Handle { shared, requests })
    }

    fn run(
        self,
        shared: &Shared,
        requests: mpsc::Receiver<Request>,
        sender: EventLoopSender,
        notify: &impl Fn(Event),
    ) {
        let mut stream = None;
        let mut received = Vec::new();
        let mut last_ready = None;
        let mut pending_file = None;
        while !shared.stopped.load(Ordering::Acquire) {
            if stream.is_none() {
                match self.listener.accept() {
                    Ok((socket, _)) if trusted_peer(&socket, shared.pid) => {
                        if socket.set_nonblocking(true).is_err() {
                            break;
                        }
                        stream = Some(socket);
                    }
                    Ok(_) => {}
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }
            }
            if let Some(socket) = &mut stream {
                let mut bytes = [0_u8; 256];
                loop {
                    match socket.read(&mut bytes) {
                        Ok(0) => return,
                        Ok(length) => {
                            received.extend_from_slice(&bytes[..length]);
                            if received.len() > 4096 {
                                return;
                            }
                            while let Some(end) = received.iter().position(|byte| *byte == b'\n') {
                                let record: Vec<_> = received.drain(..=end).collect();
                                if process_record(shared, &record, notify) {
                                    if let Some(path) = pending_file.take() {
                                        let _ = fs::remove_file(path);
                                    }
                                }
                            }
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => return,
                    }
                }
            }
            while let Ok(request) = requests.try_recv() {
                let result = self.queue(shared, stream.as_mut(), &sender, &request);
                match result {
                    Ok(path) => pending_file = Some(path),
                    Err(error) => {
                        shared.state.lock().unwrap().inflight = None;
                        notify(Event::Submitted {
                            generation: request.generation,
                            result: Err(error.to_string()),
                        });
                    }
                }
            }
            let ready = shared.prompt();
            if ready != last_ready {
                last_ready = ready;
                notify(Event::Changed);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn queue(
        &self,
        shared: &Shared,
        socket: Option<&mut UnixStream>,
        sender: &EventLoopSender,
        request: &Request,
    ) -> io::Result<PathBuf> {
        if !shared.foreground() {
            return Err(io::Error::other(
                "The shell is no longer at its confirmed prompt",
            ));
        }
        let socket =
            socket.ok_or_else(|| io::Error::other("Shell input channel is unavailable"))?;
        let path = self.root.join(format!("draft-{}", request.generation));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(request.draft.as_bytes())?;
        file.write_all(&[0])?;
        let record = format!("S {}\n", request.generation);
        // The control record is bounded and one write must queue it in full.
        // If it cannot, close the channel so a partial record can never become a command.
        match socket.write(record.as_bytes()) {
            Ok(length) if length == record.len() => {}
            _ => {
                let _ = socket.shutdown(std::net::Shutdown::Both);
                return Err(io::Error::other("Could not queue shell submission"));
            }
        }
        sender
            .send(Msg::Input(self.wake.as_bytes().to_vec().into()))
            .map_err(|_| io::Error::other("PTY closed while submitting the command"))?;
        Ok(path)
    }
}

impl Drop for Prepared {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Default)]
struct State {
    prompt: Option<u64>,
    inflight: Option<u64>,
}
struct Shared {
    master: File,
    pid: i32,
    state: Mutex<State>,
    stopped: AtomicBool,
}
impl Shared {
    fn foreground(&self) -> bool {
        // SAFETY: the descriptor is owned for the lifetime of this shared state.
        unsafe { libc::tcgetpgrp(self.master.as_raw_fd()) == self.pid }
    }
    fn prompt(&self) -> Option<u64> {
        if self.stopped.load(Ordering::Acquire) || !self.foreground() {
            return None;
        }
        let state = self.state.lock().unwrap();
        if state.inflight.is_some() {
            None
        } else {
            state.prompt
        }
    }
}
struct Request {
    generation: u64,
    draft: String,
}
#[derive(Clone)]
pub(super) struct Handle {
    shared: Arc<Shared>,
    requests: mpsc::Sender<Request>,
}
impl Handle {
    pub fn prompt(&self) -> Option<u64> {
        self.shared.prompt()
    }
    pub fn invalidate(&self) {
        self.shared.state.lock().unwrap().prompt = None;
    }
    pub fn stop(&self) {
        self.shared.stopped.store(true, Ordering::Release);
    }
    pub fn submit(&self, generation: u64, draft: String) -> Result<(), String> {
        if draft.contains('\0') {
            return Err("Commands cannot contain NUL characters".into());
        }
        if !self.shared.foreground() || self.shared.stopped.load(Ordering::Acquire) {
            return Err("The shell is no longer ready for command input".into());
        }
        let mut state = self.shared.state.lock().unwrap();
        if state.prompt != Some(generation) || state.inflight.is_some() {
            return Err("The shell prompt changed before submission".into());
        }
        state.inflight = Some(generation);
        state.prompt = None;
        if self.requests.send(Request { generation, draft }).is_err() {
            state.inflight = None;
            return Err("Shell input worker stopped".into());
        }
        Ok(())
    }
}

fn process_record(shared: &Shared, record: &[u8], notify: &impl Fn(Event)) -> bool {
    let Some(text) = std::str::from_utf8(record).ok() else {
        return false;
    };
    let Some((kind, value)) = text.trim().split_once(' ') else {
        return false;
    };
    let Ok(generation) = value.parse::<u64>() else {
        return false;
    };
    let mut state = shared.state.lock().unwrap();
    match kind {
        "P" => {
            if let Some(pending) = state.inflight.take() {
                notify(Event::Submitted {
                    generation: pending,
                    result: Err("The shell prompt changed before acceptance".into()),
                });
            }
            state.prompt = Some(generation);
        }
        "B" => state.prompt = None,
        "A" | "R" if state.inflight == Some(generation) => {
            state.inflight = None;
            state.prompt = None;
            notify(Event::Submitted {
                generation,
                result: if kind == "A" {
                    Ok(())
                } else {
                    Err("The shell rejected a stale command submission".into())
                },
            });
            return true;
        }
        _ => {}
    }
    false
}

fn trusted_peer(socket: &UnixStream, pid: i32) -> bool {
    let mut peer = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: getsockopt writes one ucred into the correctly sized output slot.
    let result = unsafe {
        libc::getsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            peer.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if result != 0 {
        return false;
    }
    // SAFETY: a successful SO_PEERCRED call initialized the complete ucred.
    let peer = unsafe { peer.assume_init() };
    peer.pid == pid && peer.uid == unsafe { libc::getuid() }
}
