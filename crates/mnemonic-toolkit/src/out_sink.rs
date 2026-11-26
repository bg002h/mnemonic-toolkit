//! The one stdout sink (FIX-tk-stdout-write-errors, T6).
//!
//! `main` hands every verb this writer instead of `io::stdout()`. A failed write is recorded (the first
//! non-`BrokenPipe` error only), later writes are dropped, and the call still returns `Ok`, so the verb runs to its
//! own end and keeps its own exit code and stderr lines. `main` then flushes the sink once and maps the exit code
//! (`OutSink::exit_code`). The `nothing is echoed` rule: the recorded error is an `io::Error` and names no output
//! content.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set before `main` when fd 1 was closed at exec (`>&-`). std's runtime start-up then reopens fd 1 on
/// `/dev/null`, so the output would be discarded with exit 0; the sink reports it as `EBADF` instead.
static STDOUT_CLOSED_AT_START: AtomicBool = AtomicBool::new(false);

/// Record that fd 1 was closed when the process started. Called from the binary's pre-`main` probe
/// (`src/main.rs`); it only stores a flag, so it is safe to call before std's runtime is set up.
pub fn mark_stdout_closed_at_start() {
    STDOUT_CLOSED_AT_START.store(true, Ordering::Relaxed);
}

/// A read-only fd 1 (`1</dev/null`): std's `Stdout` reports `EBADF` as a successful write, so check it once.
/// A fd 1 that was closed at exec (see `STDOUT_CLOSED_AT_START`) is the same `EBADF`.
/// A failing `fcntl` is not an error here. Non-unix: no check.
#[cfg(unix)]
fn fd1_read_only() -> Option<io::Error> {
    if STDOUT_CLOSED_AT_START.load(Ordering::Relaxed) {
        return Some(io::Error::from_raw_os_error(libc::EBADF));
    }
    // SAFETY: `fcntl(F_GETFL)` reads the status flags of fd 1 and touches no memory.
    let flags = unsafe { libc::fcntl(1, libc::F_GETFL) };
    if flags < 0 {
        return None;
    }
    let mode = flags & libc::O_ACCMODE;
    if mode != libc::O_WRONLY && mode != libc::O_RDWR {
        return Some(io::Error::from_raw_os_error(libc::EBADF));
    }
    None
}

#[cfg(not(unix))]
fn fd1_read_only() -> Option<io::Error> {
    None
}

/// Wraps a writer (the process stdout in the binary) and swallows its failures into a recorded state.
pub struct OutSink<W: Write> {
    inner: W,
    check_fd1: bool,
    stopped: bool,
    error: Option<io::Error>,
}

impl OutSink<io::Stdout> {
    /// The sink over the process stdout, with the read-only fd 1 check.
    pub fn stdout() -> Self {
        let mut s = OutSink::new(io::stdout());
        s.check_fd1 = true;
        s
    }
}

impl<W: Write> OutSink<W> {
    pub fn new(inner: W) -> Self {
        OutSink {
            inner,
            check_fd1: false,
            stopped: false,
            error: None,
        }
    }

    #[cfg(test)]
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// The recorded non-`BrokenPipe` error, if any.
    pub fn error(&self) -> Option<&io::Error> {
        self.error.as_ref()
    }

    /// Record a failure: `BrokenPipe` only closes the sink; any other error is kept (the first one only).
    pub fn record(&mut self, e: io::Error) {
        self.stopped = true;
        if e.kind() != io::ErrorKind::BrokenPipe && self.error.is_none() {
            self.error = Some(e);
        }
    }

    /// Run the read-only fd 1 check once (no-op for a sink not made by `stdout()`).
    pub fn check_read_only(&mut self) {
        if self.check_fd1 {
            self.check_fd1 = false;
            if let Some(e) = fd1_read_only() {
                self.record(e);
            }
        }
    }

    /// The exit code once stdout failed (T6, FIX section 3.2). `exit4_is_result` is true for `repair` and
    /// `inspect`, whose exit 4 carries the result on stdout. No recorded error: the code is unchanged.
    pub fn exit_code(&self, code: u8, exit4_is_result: bool) -> u8 {
        if self.error.is_none() {
            return code;
        }
        match code {
            0 | 5 => 1,
            4 if exit4_is_result => 1,
            c => c,
        }
    }
}

impl<W: Write> Write for OutSink<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.check_read_only();
        if self.stopped {
            return Ok(buf.len());
        }
        match self.inner.write(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => Err(e),
            Err(e) => {
                self.record(e);
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.stopped {
            return Ok(());
        }
        match self.inner.flush() {
            Err(e) if e.kind() == io::ErrorKind::Interrupted => Err(e),
            Err(e) => {
                self.record(e);
                Ok(())
            }
            Ok(()) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OutSink;
    use std::io::{self, Write};

    /// Accepts `accept` bytes in total, then every write returns `err`; optionally `flush` fails.
    struct Flaky {
        data: Vec<u8>,
        accept: usize,
        err: i32,
        flush_fails: bool,
        failing_calls: usize,
    }

    impl Flaky {
        fn new(accept: usize, err: i32, flush_fails: bool) -> Self {
            Flaky {
                data: Vec::new(),
                accept,
                err,
                flush_fails,
                failing_calls: 0,
            }
        }
    }

    impl Write for Flaky {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let room = self.accept - self.data.len();
            if room == 0 {
                self.failing_calls += 1;
                return Err(io::Error::from_raw_os_error(self.err));
            }
            let n = room.min(buf.len());
            self.data.extend_from_slice(&buf[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            if self.flush_fails {
                self.failing_calls += 1;
                return Err(io::Error::from_raw_os_error(self.err));
            }
            Ok(())
        }
    }

    #[test]
    fn r19_sink_prefix_then_error_records_first_error_and_stops() {
        let mut s = OutSink::new(Flaky::new(4, libc::ENOSPC, false));
        assert_eq!(s.write(b"0123456789").unwrap(), 4);
        // The failing second call inside write_all is absorbed.
        s.write_all(b"0123456789").unwrap();
        let calls_after_first_failure = s.get_ref().failing_calls;
        s.write_all(b"more").unwrap();
        assert_eq!(
            s.get_ref().failing_calls,
            calls_after_first_failure,
            "no write after the first failure"
        );
        assert_eq!(s.get_ref().data, b"0123");
        assert_eq!(s.error().unwrap().raw_os_error(), Some(libc::ENOSPC));
    }

    #[test]
    fn r19_first_write_returns_ok_when_the_first_call_fails() {
        let mut s = OutSink::new(Flaky::new(0, libc::ENOSPC, false));
        assert_eq!(s.write(b"abc").unwrap(), 3);
        assert_eq!(s.get_ref().failing_calls, 1);
        assert_eq!(s.write(b"abc").unwrap(), 3);
        assert_eq!(s.get_ref().failing_calls, 1);
        assert_eq!(s.error().unwrap().raw_os_error(), Some(libc::ENOSPC));
    }

    #[test]
    fn r20_sink_flush_only_failure_is_recorded() {
        let mut s = OutSink::new(Flaky::new(100, libc::ENOSPC, true));
        s.write_all(b"abc").unwrap();
        assert!(s.error().is_none());
        s.flush().unwrap();
        assert_eq!(s.error().unwrap().raw_os_error(), Some(libc::ENOSPC));
    }

    #[test]
    fn r21_exit_code_mapping() {
        // No error: every code kept.
        let s = OutSink::new(Flaky::new(100, libc::ENOSPC, false));
        for c in [0u8, 1, 2, 3, 4, 5, 64] {
            assert_eq!(s.exit_code(c, false), c);
            assert_eq!(s.exit_code(c, true), c);
        }
        // Closed pipe only: every code kept.
        let mut s = OutSink::new(Flaky::new(0, libc::EPIPE, false));
        s.write_all(b"x").unwrap();
        assert!(s.error().is_none());
        for c in [0u8, 1, 2, 3, 4, 5, 64] {
            assert_eq!(s.exit_code(c, false), c);
            assert_eq!(s.exit_code(c, true), c);
        }
        // ENOSPC.
        let mut s = OutSink::new(Flaky::new(0, libc::ENOSPC, false));
        s.write_all(b"x").unwrap();
        for (c, other, repair_inspect) in [
            (0u8, 1u8, 1u8),
            (5, 1, 1),
            (1, 1, 1),
            (2, 2, 2),
            (3, 3, 3),
            (64, 64, 64),
            (4, 4, 1),
        ] {
            assert_eq!(s.exit_code(c, false), other, "code {c} other verb");
            assert_eq!(
                s.exit_code(c, true),
                repair_inspect,
                "code {c} repair/inspect"
            );
        }
    }
}
