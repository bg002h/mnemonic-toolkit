//! The one stdout sink (FIX-tk-stdout-write-errors, T6). RED stage: tests only.

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
            Flaky { data: Vec::new(), accept, err, flush_fails, failing_calls: 0 }
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
        assert_eq!(s.get_ref().failing_calls, calls_after_first_failure, "no write after the first failure");
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
        for (c, other, repair_inspect) in [(0u8, 1u8, 1u8), (5, 1, 1), (1, 1, 1), (2, 2, 2), (3, 3, 3), (64, 64, 64), (4, 4, 1)] {
            assert_eq!(s.exit_code(c, false), other, "code {c} other verb");
            assert_eq!(s.exit_code(c, true), repair_inspect, "code {c} repair/inspect");
        }
    }
}
