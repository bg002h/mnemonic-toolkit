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
        secret_on_stdout_warning_unconditional(stderr);
    }
}

/// Unconditional form of [`secret_on_stdout_warning`] for callers whose
/// stdout payload is ALWAYS secret material and is not a `CardKind`
/// (e.g. `electrum-decrypt`'s recovered Electrum seed phrase / xprv).
/// Added v0.33.0. The `CardKind`-gated wrapper above delegates here for
/// `Ms1` (behavior-preserving).
pub(crate) fn secret_on_stdout_warning_unconditional<W: Write + ?Sized>(stderr: &mut W) {
    let _ = writeln!(
        stderr,
        "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"
    );
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

/// Security class of what a command wrote to stdout. Variant declaration order
/// is ascending sensitivity (Template < WatchOnly < PrivateKeyMaterial) so
/// `#[derive(Ord)]`'s `.max()` returns the most-sensitive class. "inert" is the
/// ABSENCE of a class (modeled as `Option::None`), not a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputClass { Template, WatchOnly, PrivateKeyMaterial }

/// Max over the artifacts a command wrote to stdout; `None` == all-inert → no line.
pub fn worst_class_on_stdout(artifacts: &[OutputClass]) -> Option<OutputClass> {
    artifacts.iter().copied().max()
}

/// Map a repaired/inspected card kind to its output class.
pub fn card_kind_class(kind: crate::repair::CardKind) -> OutputClass {
    match kind {
        crate::repair::CardKind::Ms1 => OutputClass::PrivateKeyMaterial,
        crate::repair::CardKind::Mk1 => OutputClass::WatchOnly,
        crate::repair::CardKind::Md1 => OutputClass::Template,
    }
}

/// Emit the one-line stderr class advisory. Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: Write + ?Sized>(class: OutputClass, stderr: &mut W) {
    let line = match class {
        OutputClass::PrivateKeyMaterial =>
            "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')",
        OutputClass::WatchOnly => "note: stdout is watch-only — public keys only, cannot spend",
        OutputClass::Template => "note: stdout is a keyless descriptor template (no keys)",
    };
    let _ = writeln!(stderr, "{line}");
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

    #[test]
    fn output_class_lattice_and_lines() {
        use super::{OutputClass::*, worst_class_on_stdout, emit_output_class_advisory};
        assert_eq!(worst_class_on_stdout(&[]), None);
        assert_eq!(worst_class_on_stdout(&[Template, WatchOnly]), Some(WatchOnly));
        assert_eq!(worst_class_on_stdout(&[WatchOnly, PrivateKeyMaterial, Template]), Some(PrivateKeyMaterial));
        let mut b = Vec::new();
        emit_output_class_advisory(PrivateKeyMaterial, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(),
            "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n");
        let mut b = Vec::new(); emit_output_class_advisory(WatchOnly, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(), "note: stdout is watch-only — public keys only, cannot spend\n");
        let mut b = Vec::new(); emit_output_class_advisory(Template, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(), "note: stdout is a keyless descriptor template (no keys)\n");
    }

    #[test]
    fn card_kind_maps_to_class() {
        use super::{card_kind_class, OutputClass};
        use crate::repair::CardKind;
        assert_eq!(card_kind_class(CardKind::Ms1), OutputClass::PrivateKeyMaterial);
        assert_eq!(card_kind_class(CardKind::Mk1), OutputClass::WatchOnly);
        assert_eq!(card_kind_class(CardKind::Md1), OutputClass::Template);
    }
}
