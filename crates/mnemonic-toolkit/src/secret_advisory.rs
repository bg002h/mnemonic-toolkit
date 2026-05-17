//! Stderr advisory helpers for secret material.
//!
//! Two advisory classes live here:
//!
//! 1. **argv-leakage class** ([`secret_in_argv_warning`]) — emits a
//!    uniform `warning: secret material on argv (<flag>) — pipe via
//!    <alternative> to avoid /proc/$PID/cmdline exposure` line to stderr
//!    for each inline-secret occurrence. No dedup; callers emit one
//!    advisory per (flag, slot-index) so the user sees every leak site.
//!    Mirrors the secret-on-stdout shape (cite `bundle.rs:697`,
//!    `convert.rs:799`, `derive_child.rs:205`).
//!
//! 2. **path-permission class** ([`warn_if_world_readable`]) — emits a
//!    `warning: --json-out <path> inherits umask (file may be
//!    world-readable, mode <m>); consider --json-out /dev/stdout or
//!    chmod 0600 the path before invoking` advisory when the side-effect
//!    JSON envelope path is world/group readable. Extracted at v0.13.0
//!    P2.2 GREEN (R0 Q5 fold) from its prior private home at
//!    `cmd/seed_xor.rs::emit_world_readable_advisory`; 3 lockstep call
//!    sites — `cmd/seed_xor.rs`, `cmd/final_word.rs`,
//!    `cmd/slip39.rs`.
//!
//! Authoritative reference:
//! `design/SPEC_secret_memory_hygiene_v0_9_0.md` §1 item 1 + survey §6
//! cross-cutting observation 4.

use std::io::Write;
use std::path::Path;

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

/// Emit a `secret-on-stdout` advisory when sensitive card material is
/// being written to stdout (ms1 = BIP-39 entropy). Mirrors the bundle
/// command's secret-on-stdout warning emission. Errors writing to
/// `stderr` are silently swallowed (advisory is best-effort).
///
/// Added v0.22.0 for the `repair` + `inspect` features per plan D9.
/// No-op for kinds other than `Ms1` (mk1 / md1 are not secret-bearing).
pub fn secret_on_stdout_warning<W: Write + ?Sized>(kind: crate::repair::CardKind, stderr: &mut W) {
    if matches!(kind, crate::repair::CardKind::Ms1) {
        let _ = writeln!(
            stderr,
            "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"
        );
    }
}

/// Emit a `--json-out <path>` world-readable / group-readable advisory
/// if the file at `path` has permissions outside the user-private mask
/// (i.e. `mode & 0o077 != 0`). Unix-only; no-op on non-Unix platforms.
///
/// Errors reading metadata are silently swallowed (advisory is
/// best-effort).
pub fn warn_if_world_readable<E: Write>(path: &Path, stderr: &mut E) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o077 != 0 {
                let _ = writeln!(
                    stderr,
                    "warning: --json-out {} inherits umask (file may be world-readable, mode {:o}); consider --json-out /dev/stdout or chmod 0600 the path before invoking",
                    path.display(),
                    mode & 0o777,
                );
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (stderr, path); // suppress unused warnings on non-Unix
    }
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
    fn warning_shape_for_slot_flag() {
        let mut buf: Vec<u8> = Vec::new();
        secret_in_argv_warning(&mut buf, "--slot @0.phrase=", "--slot @0.phrase=-");
        let s = String::from_utf8(buf).unwrap();
        assert!(s.starts_with("warning: secret material on argv (--slot @0.phrase=)"));
        assert!(s.contains("--slot @0.phrase=-"));
        assert!(s.ends_with("/proc/$PID/cmdline exposure\n"));
    }

    #[cfg(unix)]
    #[test]
    fn warn_if_world_readable_emits_for_0o644() {
        use std::os::unix::fs::PermissionsExt;
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::set_permissions(f.path(), std::fs::Permissions::from_mode(0o644)).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        warn_if_world_readable(f.path(), &mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.contains("world-readable") && s.contains("644"),
            "0o644 must emit world-readable advisory with mode in stem; got: {s}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn warn_if_world_readable_silent_for_0o600() {
        use std::os::unix::fs::PermissionsExt;
        let f = tempfile::NamedTempFile::new().unwrap();
        std::fs::set_permissions(f.path(), std::fs::Permissions::from_mode(0o600)).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        warn_if_world_readable(f.path(), &mut buf);
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.is_empty(),
            "0o600 must NOT emit advisory; got: {s}"
        );
    }
}
