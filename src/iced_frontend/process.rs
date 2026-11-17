use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub(super) struct ChildProcess(Arc<OwnedFd>);

impl ChildProcess {
    pub fn open(pid: u32) -> io::Result<Self> {
        // SAFETY: pidfd_open uses integer arguments and returns a new owned FD.
        let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: No other owner exists for the descriptor just returned.
        Ok(Self(Arc::new(unsafe { OwnedFd::from_raw_fd(fd as i32) })))
    }

    pub fn kill_after_hangup_grace_period(&self) {
        let child = Arc::clone(&self.0);
        let launched = thread::Builder::new()
            .name("shell shutdown".into())
            .spawn(move || wait_then_kill(&child));
        if let Err(error) = launched {
            eprintln!("Could not start shell shutdown timer: {error}");
            kill(&self.0);
        }
    }
}

fn wait_then_kill(child: &OwnedFd) {
    // The PTY gets time to deliver SIGHUP and the shell gets time to exit.
    // A pidfd avoids signalling a different process if the PID gets reused.
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut poll = libc::pollfd {
        fd: child.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    loop {
        let timeout = deadline.saturating_duration_since(Instant::now());
        let timeout_ms = timeout.as_millis().max(1) as i32;
        // SAFETY: poll points to one valid pollfd with a live file descriptor.
        match unsafe { libc::poll(&mut poll, 1, timeout_ms) } {
            0 => break,
            ready if ready > 0 => return,
            _ if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted => continue,
            _ => break,
        }
    }
    kill(child);
}

fn kill(child: &OwnedFd) {
    // SAFETY: The FD references the exact shell process. No signal payload is sent.
    let result = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            child.as_raw_fd(),
            libc::SIGKILL,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            eprintln!("Could not terminate shell after hangup: {error}");
        }
    }
}
