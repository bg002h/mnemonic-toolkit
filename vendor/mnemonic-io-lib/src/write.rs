//! **Creating a file that a bearer artifact may be written into.**
//!
//! One function. It is here rather than in [`super::channel`] or [`super::fd`]
//! because both of those declare narrower contracts than an effectful write:
//! `channel` is *"classification ONLY"* and says in as many words that
//! `destination` **never touches a path**, and `fd` is *"what was measured about
//! stdout"* — the read side of one file descriptor. Widening either header to
//! admit a path-writer would cost more than a seventh module does.
//!
//! ## Why `0o600` here is mechanism, when `0o044` and `0o077` are not
//!
//! [`super::fd`] refuses to publish a *disqualifying* mask because `me` rules
//! `0o044` and `mt` rules `0o077`, and the crate does not get to settle which
//! of them is right. **That is a disagreement about somebody else's file.**
//!
//! The mode a tool creates *its own* output at is not that question, and the two
//! consumers do not disagree about it: a file this crate's callers create for a
//! bearer artifact is owner-only. So `0o600` is a constant here rather than a
//! parameter. If a third consumer ever needs a different creation mode, that is
//! the moment to add one — not before, and a parameter added speculatively would
//! be the first place a caller could weaken the mode by accident.
//!
//! ## F-244 — the half that is easy to leave out
//!
//! `OpenOptions::mode()` binds **on create only**. Open an existing file with it
//! and the mode on disk is untouched, so `me sysw pack --out stale.bin` over a
//! target already at `0644` left it at `0644` — and that is the case an operator
//! re-running a command actually hits, not an exotic one.
//!
//! The fix is the `set_permissions` call below, and it is deliberately made on
//! the **open file** rather than on the path. A path-based `set_permissions`
//! names the file a second time, and between the two calls the name can be made
//! to point somewhere else; a handle cannot be redirected once it is open.

/// Write `bytes` to `path`, creating or truncating it owner-only.
///
/// `truncate(true)` is load-bearing and not a stylistic echo of
/// `std::fs::write`: without it a **shrinking** overwrite — a smaller manifest
/// written over a larger one — leaves the tail of the old file in place, which
/// for a JSON artifact means trailing bytes after the closing brace.
///
/// On non-Unix the mode calls compile out and the create/truncate semantics
/// remain. The threat model is POSIX: mode bits do not mean the same thing
/// elsewhere, and pretending otherwise would be worse than saying so.
pub fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    // F-244. `opts.mode()` above did nothing if `path` already existed, so the
    // mode is set a second time on the OPEN FILE -- see the module header for
    // why the handle and not the path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    f.write_all(bytes)
}

#[cfg(all(test, unix))]
mod tests {
    use super::write_private;
    use std::os::unix::fs::PermissionsExt;

    fn mode_of(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// **THE GATE, and the pre-existing target is the whole of it.**
    ///
    /// The fresh-file half passes under any implementation that sets a mode at
    /// all, so on its own it proves nothing. The `0644` half is what fails
    /// against `OpenOptions::mode()` alone — measured: without the
    /// `set_permissions` call this assertion reports `0o644` — which makes it a
    /// gate on F-244 rather than a restatement of the doc comment.
    ///
    /// It also pins the CONTENTS, because a function that tightened the mode and
    /// wrote nothing would satisfy a permissions-only test.
    #[test]
    fn an_existing_world_readable_target_is_tightened_not_inherited() {
        let d = tempfile::tempdir().unwrap();

        let fresh = d.path().join("fresh.bin");
        write_private(&fresh, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&fresh),
            0o600,
            "a file this function creates is owner-only"
        );

        let stale = d.path().join("stale.bin");
        std::fs::write(&stale, b"old").unwrap();
        std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            mode_of(&stale),
            0o644,
            "the control: the target really is 0644 before the call"
        );

        write_private(&stale, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&stale),
            0o600,
            "F-244: `0o600` binds on CREATE, so an implementation that only passes \
             it to OpenOptions leaves this at 0644 and reports success"
        );
        assert_eq!(
            std::fs::read(&stale).unwrap(),
            b"new bytes",
            "the new contents must be there -- tightening a file this function \
             failed to write would pass a mode-only assertion"
        );
    }

    /// A **shrinking** overwrite leaves no tail of the previous file.
    ///
    /// Dropping `truncate(true)` is the tidy-up that looks harmless: the mode
    /// assertions above all still pass, and a smaller JSON manifest written over
    /// a larger one silently acquires trailing bytes after its closing brace.
    #[test]
    fn a_shrinking_overwrite_leaves_no_stale_tail() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("manifest.json");
        write_private(&p, b"{\"a\":1,\"padding\":\"xxxxxxxxxxxxxxxxxxxx\"}").unwrap();
        write_private(&p, b"{\"a\":1}").unwrap();
        assert_eq!(
            std::fs::read(&p).unwrap(),
            b"{\"a\":1}",
            "without truncate(true) the tail of the longer write survives"
        );
    }
}
