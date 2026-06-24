//! Process-level secret-exposure hardening.
//!
//! `set_non_dumpable()` calls `prctl(PR_SET_DUMPABLE, 0)` (Linux), which:
//!   - makes `/proc/$PID/` owned by root + unreadable to OTHER non-root UIDs
//!     (so other users cannot read `/proc/$PID/cmdline` to harvest a secret
//!     passed inline on argv), and
//!   - disables core dumps (so a secret on argv/heap won't land in a core file).
//!
//! On the BSDs (FreeBSD/OpenBSD/NetBSD) `prctl` does not exist; the function
//! gets parity via a second cfg arm:
//!   - FreeBSD only — `procctl(PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` disables
//!     ptrace/ktrace/debugging-sysctl introspection AND core dumping for this
//!     process, and
//!   - all three BSDs — `setrlimit(RLIMIT_CORE, {0, 0})` hard-zeros the core-dump
//!     size so a secret on heap/argv cannot land in a core file.
//!
//! macOS and Windows remain a documented no-op (no equivalent primitive wired).
//!
//! It does NOT hide cmdline from the SAME UID (a same-UID attacker already has
//! ptrace / `/proc/$PID/mem` access to the live process) — that residual is
//! accepted; this is the reliable, non-fragile companion to the `--*-stdin`
//! argv-leakage advisories (`secret_advisory`). Best-effort: a
//! `prctl` failure is ignored. The in-place argv-overwrite alternative was
//! deliberately declined (glibc/musl/static-linking-fragile + racy); see the
//! `argv-overwrite-after-parse` FOLLOWUP closure.

/// Deny other-UID `/proc/$PID` reads + core dumps for this process.
/// Linux + the three BSDs; a documented no-op on macOS/Windows.
pub fn set_non_dumpable() {
    #[cfg(target_os = "linux")]
    unsafe {
        // SAFETY: prctl(PR_SET_DUMPABLE, 0) takes no pointers; always sound.
        let _ = libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }

    // BSD parity: prctl(PR_SET_DUMPABLE) is Linux-only. The closest analog +
    // a portable backstop:
    //   (i)  FreeBSD only — procctl(PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE):
    //        disables ptrace/ktrace/debugging-sysctl/hwpmc/dtrace introspection
    //        AND core dumping for THIS process (re-enabled on execve, which we
    //        never call). OpenBSD/NetBSD have no procctl — they get parity via:
    //   (ii) ALL three BSDs — setrlimit(RLIMIT_CORE, {0,0}): hard-zeros the
    //        core-dump size regardless of the kern.corefile / kern.coredump
    //        sysctl, so a secret on heap/argv cannot land in a core file.
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    unsafe {
        // (i) FreeBSD trace-disable (best-effort; ignored on failure).
        #[cfg(target_os = "freebsd")]
        {
            // SAFETY: procctl with P_PID/id=0 (calling process) and a pointer
            // to a valid c_int `data`. The kernel reads sizeof(int) at `data`;
            // `ctl` outlives the call. No aliasing. Return value ignored.
            let mut ctl: libc::c_int = libc::PROC_TRACE_CTL_DISABLE;
            let _ = libc::procctl(
                libc::P_PID,
                0,
                libc::PROC_TRACE_CTL,
                &mut ctl as *mut libc::c_int as *mut libc::c_void,
            );
        }
        // (ii) Portable core-dump hard-zero on all three BSDs.
        // SAFETY: setrlimit reads sizeof(rlimit) at the pointer; `lim` is a
        // fully-initialized stack value that outlives the call. The `0` field
        // literals coerce to whichever signedness `rlim_t` has on the target
        // (i64 on freebsdlike, u64 on netbsdlike) — the arm is signedness-agnostic.
        let lim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let _ = libc::setrlimit(libc::RLIMIT_CORE, &lim);
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

    // BSD tests below are COMPILE-GATED ONLY and are NEVER executed by the CI
    // in these cycles (the only BSD CI leg is a `cargo check` compile-gate, not
    // a test-execution leg). Their runtime asserts are compile-checked for
    // symbol/type correctness but their RUNTIME truth is unverified by any
    // automation here — they are documentation / future-native-VM scaffolding,
    // not a live runtime gate.
    #[cfg(target_os = "freebsd")]
    #[test]
    fn set_non_dumpable_disables_trace_on_freebsd() {
        super::set_non_dumpable();
        let mut status: libc::c_int = 0;
        // SAFETY: procctl with P_PID/id=0 (calling process) and a pointer to a
        // valid c_int `status`. The kernel writes sizeof(int) at `status`.
        unsafe {
            libc::procctl(
                libc::P_PID,
                0,
                libc::PROC_TRACE_STATUS,
                &mut status as *mut libc::c_int as *mut libc::c_void,
            );
        }
        // PROC_TRACE_STATUS writes *data = -1 when tracing is DISABLED, 0 when
        // enabled-but-no-debugger, the debugger PID when a debugger is attached
        // (procctl(2) man page). After PROC_TRACE_CTL_DISABLE, -1 is the correct
        // value in the normal case (precondition: no external tracer attached).
        assert_eq!(status, -1); // -1 == tracing disabled (per procctl(2) man page)
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    #[test]
    fn set_non_dumpable_zeros_core_limit_on_bsd() {
        super::set_non_dumpable();
        // SAFETY: getrlimit writes sizeof(rlimit) into the zeroed stack value.
        let mut lim = unsafe { std::mem::zeroed::<libc::rlimit>() };
        unsafe {
            libc::getrlimit(libc::RLIMIT_CORE, &mut lim);
        }
        // Compare against the `0` integer literal — no signedness assumption
        // (`rlim_t` is i64 on FreeBSD, u64 on NetBSD/OpenBSD).
        assert_eq!(lim.rlim_cur, 0); // 0 literal — no signedness assumption
    }
}
