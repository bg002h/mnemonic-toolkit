//! Process-level secret-exposure hardening.
//!
//! `set_non_dumpable()` calls `prctl(PR_SET_DUMPABLE, 0)` (Linux), which:
//!   - makes `/proc/$PID/` owned by root + unreadable to OTHER non-root UIDs
//!     (so other users cannot read `/proc/$PID/cmdline` to harvest a secret
//!     passed inline on argv), and
//!   - disables core dumps (so a secret on argv/heap won't land in a core file).
//!
//! It does NOT hide cmdline from the SAME UID (a same-UID attacker already has
//! ptrace / `/proc/$PID/mem` access to the live process) — that residual is
//! accepted; this is the reliable, non-fragile companion to the `--*-stdin`
//! argv-leakage advisories (`secret_advisory`). Best-effort: a
//! `prctl` failure is ignored. The in-place argv-overwrite alternative was
//! deliberately declined (glibc/musl/static-linking-fragile + racy); see the
//! `argv-overwrite-after-parse` FOLLOWUP closure.

/// Deny other-UID `/proc/$PID` reads + core dumps for this process.
/// Linux-only; a no-op on other platforms.
pub fn set_non_dumpable() {
    #[cfg(target_os = "linux")]
    unsafe {
        // SAFETY: prctl(PR_SET_DUMPABLE, 0) takes no pointers; always sound.
        let _ = libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn set_non_dumpable_clears_dumpable_flag() {
        super::set_non_dumpable();
        // SAFETY: PR_GET_DUMPABLE takes no pointers.
        let d = unsafe { libc::prctl(libc::PR_GET_DUMPABLE) };
        assert_eq!(d, 0, "PR_SET_DUMPABLE(0) should leave dumpable == 0");
    }
}
