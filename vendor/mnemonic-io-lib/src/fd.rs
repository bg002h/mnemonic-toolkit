//! **MECHANISM ONLY: what was measured about stdout, never what it means.**
//!
//! `me` disqualifies a destination whose mode has any bit in `0o044` — read
//! for group or other. `mt` disqualifies `0o077` — *every* group/other bit,
//! because someone who can WRITE the file can alter the strings before they
//! are cut into metal. Measured 2026-08-26 on the same valid transaction with
//! stdout at mode 0620: `me sysw pack` exits 0 and writes 733 bytes; `mt
//! encode` exits 1 and refuses. Both exit 0 at 0600, so the divergence is the
//! mode and nothing else.
//!
//! **`mt` is arguably right, and P0 does not settle it.** The masks are a
//! deliberate disagreement between two tools; hoisting one of them would force
//! a decision neither has agreed to make, inside a shared dependency where the
//! argument is hardest to have — and the likely resolution is silent adoption
//! of whichever mask the shared crate happened to ship. **So no disqualifying
//! mask lives here. Not `0o044`, not `0o077`.**
//!
//! ## The contract, stated because "no policy" is not self-explaining
//!
//! - Return the **raw `mode & 0o777`** for a regular file — the number, not a
//!   verdict about it.
//! - Return `None` for a **character device**.
//! - Return `None` on a **failed `fstat`** — fail OPEN.
//!
//! **Those two `None`s are shared MECHANISM, not policy, and saying so is
//! load-bearing.** Both `me` and `mt` already implement both, identically,
//! comment sentences included. An implementer who reads "no policy" literally
//! would push the char-device exemption out to callers — where it is
//! load-bearing for `/dev/null`, which is mode **0666**, so a mode-only test
//! would refuse the most ordinary redirect there is, and where any one caller
//! can forget it. Fail-open is the same shape: *unreadable stdout is not
//! evidence of exposure*, and both tools say so in those words.

/// Reduce a `Metadata` to the mode a caller may reason about, or `None` if
/// there is nothing to reason about.
///
/// Split out from [`stdout_mode`] so the contract above is reachable by a unit
/// test: fd 1 belongs to the test harness and cannot be repointed from inside
/// a test without `dup2`, but the decision this function makes is a pure
/// function of the metadata.
///
/// **It returns `Some(0o620)`.** A masked implementation cannot — `0o620 &
/// 0o044 == 0` — which is exactly what makes the unit test below a gate rather
/// than a restatement.
#[cfg(unix)]
pub fn mode_of(md: &std::fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    // A terminal and /dev/null persist nothing, so neither can leak — and
    // /dev/null is 0666, so without this exemption `me … > /dev/null` would be
    // refused. (F-253 covers the terminal separately and for a different
    // reason: a terminal persists in SCROLLBACK, and sessions are logged.)
    if md.file_type().is_char_device() {
        return None;
    }
    // F-252: the MODE is returned, not a verdict, so a refusal can quote the
    // number it measured instead of asserting a reachability fact it never
    // established.
    Some(md.permissions().mode() & 0o777)
}

/// The mode of this process's stdout, if it is a regular file.
///
/// F-244. `me sysw pack ... > payload.bin` hands the container to a file `me`
/// never names -- but a process can `fstat` its own stdout, so the mode is
/// visible even when the path is not.
///
/// **KEYED ON MODE BITS, NOT ON `S_ISREG`** — R0 round 0, finding I3. The first
/// version of this asked `is_file()`, and its comment claimed a FIFO "has no
/// meaningful mode". **Measured false:** a NAMED fifo carries a mode (`mkfifo`
/// gives 0666) and a third party reading it really does receive the bytes. Only
/// the ANONYMOUS pipe behind `|` is 0600, which the mode test passes on its own.
#[cfg(unix)]
pub fn stdout_mode() -> Option<u32> {
    use std::mem::ManuallyDrop;
    use std::os::unix::io::FromRawFd;
    // ManuallyDrop: fd 1 belongs to the process, and dropping the File would
    // CLOSE stdout out from under everything downstream.
    let f = unsafe { ManuallyDrop::new(std::fs::File::from_raw_fd(1)) };
    match f.metadata() {
        Ok(md) => mode_of(&md),
        // Unreadable stdout is not evidence of exposure; fail OPEN rather than
        // refusing a write for a reason we cannot state.
        Err(_) => None,
    }
}

#[cfg(not(unix))]
pub fn stdout_mode() -> Option<u32> {
    None
}

#[cfg(all(test, unix))]
mod tests {
    use super::mode_of;
    use std::os::unix::fs::PermissionsExt;

    fn at_mode(dir: &std::path::Path, name: &str, mode: u32) -> std::fs::Metadata {
        let p = dir.join(name);
        std::fs::write(&p, b"x").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode)).unwrap();
        std::fs::metadata(&p).unwrap()
    }

    /// **The gate for the mask split.** The 0644 row passes under either
    /// implementation and proves only that something is returned. **The 0620
    /// row is the gate**: a mode with no read bit outside owner is invisible to
    /// a `& 0o044` implementation, which returns `None` for it — so this test
    /// goes RED the moment a disqualifying mask reappears in this file.
    #[test]
    fn the_raw_mode_is_returned_including_one_no_read_mask_can_see() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            mode_of(&at_mode(d.path(), "a", 0o644)),
            Some(0o644),
            "a 0644 file's mode is 0o644, not a verdict about it"
        );
        assert_eq!(
            mode_of(&at_mode(d.path(), "b", 0o620)),
            Some(0o620),
            "0o620 & 0o044 == 0 -- a masked implementation returns None here, \
             and returning the raw mode is the whole of the split"
        );
        assert_eq!(
            mode_of(&at_mode(d.path(), "c", 0o600)),
            Some(0o600),
            "even a mode nobody would refuse is REPORTED; deciding is the caller's"
        );
    }

    /// The char-device exemption is mechanism, not policy, and it stays here.
    /// `/dev/null` is mode 0666: without this, every `me … > /dev/null` would
    /// be refused.
    #[test]
    fn a_character_device_has_no_mode_to_reason_about() {
        let md = std::fs::metadata("/dev/null").unwrap();
        assert_eq!(md.permissions().mode() & 0o777, 0o666, "/dev/null is 0666");
        assert_eq!(
            mode_of(&md),
            None,
            "a character device persists nothing, so there is nothing to refuse"
        );
    }
}
