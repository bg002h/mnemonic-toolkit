//! v0.9.0 Cycle A Phase 1 — `secret-in-argv` stderr advisory helper.
//!
//! Emits a uniform `warning: secret material on argv (<flag>) — pipe via
//! <alternative> to avoid /proc/$PID/cmdline exposure` line to stderr
//! for each inline-secret occurrence. Mirrors the existing
//! `secret-on-stdout` shape (cite `bundle.rs:697`, `convert.rs:799`,
//! `derive_child.rs:205`).
//!
//! No dedup: callers are expected to emit one advisory per
//! (flag, slot-index) occurrence so the user can see every leak site.
//! The shape is intentionally per-call, not once-per-flag —
//! `--slot @0.phrase=<X> --slot @1.phrase=<Y>` should emit two
//! advisories, not one (each is a distinct argv leak).
//!
//! Authoritative reference:
//! `design/SPEC_secret_memory_hygiene_v0_9_0.md` §1 item 1 + survey §6
//! cross-cutting observation 4.

use std::io::Write;

/// Emit a `secret-in-argv` advisory naming `flag` and pointing at the
/// `alternative` stdin route. Errors writing to `stderr` are silently
/// swallowed (advisory is best-effort; users get a degraded warning if
/// stderr is closed, never a fatal).
pub fn secret_in_argv_warning<W: Write>(stderr: &mut W, flag: &str, alternative: &str) {
    let _ = writeln!(
        stderr,
        "warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warning_byte_exact_for_simple_flag() {
        let mut buf: Vec<u8> = Vec::new();
        secret_in_argv_warning(&mut buf, "--passphrase", "--passphrase-stdin");
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(
            s,
            "warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure\n"
        );
    }

    #[test]
    fn warning_byte_exact_for_slot_flag() {
        let mut buf: Vec<u8> = Vec::new();
        secret_in_argv_warning(&mut buf, "--slot @0.phrase=", "--slot @0.phrase=-");
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("warning: secret material on argv (--slot @0.phrase=)"));
        assert!(s.contains("--slot @0.phrase=-"));
        assert!(s.ends_with("/proc/$PID/cmdline exposure\n"));
    }
}
