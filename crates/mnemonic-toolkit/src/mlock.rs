//! POSIX `mlock(2)` page-pinning for OWNED secret-bearing heap buffers.
//!
//! Cycle B Phase 2 (mlock infrastructure). See
//! `design/SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2 for the locked
//! Fix-B design: slice-fn primitive only (no wrapper type).
//!
//! Module surface:
//! - [`pin_pages_for`] — pin the heap pages covering a `&[u8]`. Zero-length
//!   is a no-op (returns empty range; no syscall).
//! - [`PinnedPageRange`] — RAII handle; Drop munlocks.
//! - [`report_at_exit`] — emit a stderr summary iff any mlock failed.
//! - Test helpers ([`page_size_for_test`], [`failure_count_for_test`],
//!   [`first_errno_for_test`]) so integration tests in `tests/` can read the
//!   process-static state without depending on `#[cfg(test)]` (which is
//!   per-crate-not-per-build; see RFC 1604).
//!
//! **Caller contract for `pin_pages_for`:** the buffer's heap address MUST
//! remain stable for the lifetime of the returned `PinnedPageRange`. Vec
//! reallocation (`.push` / `.extend` / `.reserve`) invalidates the pin. Sites
//! 2/3/4 (per SPEC §4 P3a) use construct-and-pin idioms that satisfy this.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

// ============================================================================
// Page size accessor (cached)
// ============================================================================

fn page_size() -> usize {
    static PAGE_SIZE: OnceLock<usize> = OnceLock::new();
    *PAGE_SIZE.get_or_init(|| {
        #[cfg(miri)]
        {
            4096
        }
        #[cfg(not(miri))]
        {
            // SAFETY: `libc::sysconf` with `_SC_PAGESIZE` is a POSIX-mandated
            // call that returns a positive long on Linux and macOS. We coerce
            // to usize; a negative result (errno-signalling) falls back to
            // 4096 which is the universal Linux x86_64 default.
            let v = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
            if v > 0 {
                v as usize
            } else {
                4096
            }
        }
    })
}

// ============================================================================
// PinnedPageRange + pin_pages_for
// ============================================================================

/// Page range pinned by [`pin_pages_for`]. Drop munlocks the range.
pub struct PinnedPageRange {
    pub start: *const u8,
    pub page_count: usize,
}

impl PinnedPageRange {
    fn empty() -> Self {
        Self {
            start: std::ptr::null(),
            page_count: 0,
        }
    }
}

impl Drop for PinnedPageRange {
    fn drop(&mut self) {
        if self.page_count == 0 || self.start.is_null() {
            return;
        }
        let bytes = self.page_count * page_size();
        // SAFETY: `self.start` was returned by a prior successful mlock in
        // `pin_pages_for`; caller contract requires the underlying buffer to
        // remain valid until this Drop (no Vec reallocation). munlock does
        // not dereference the address; only kernel bookkeeping updates.
        let _ = unsafe { sys_munlock(self.start, bytes) };
        // munlock errno is ignored: there is no recovery path on cleanup.
    }
}

/// Pin the heap pages covering `buf`. Zero-length is a no-op (returns empty
/// range; no syscall is issued). See SPEC §2 row 1 for the page-rounding
/// formula.
pub fn pin_pages_for(buf: &[u8]) -> PinnedPageRange {
    if buf.is_empty() {
        return PinnedPageRange::empty();
    }
    let (start, page_count) = round_to_pages(buf.as_ptr() as usize, buf.len(), page_size());
    let bytes = page_count * page_size();
    let start_ptr = start as *const u8;

    mlock_state().record_attempt();

    // SAFETY: `start_ptr` is page-aligned (computed by round_to_pages's
    // round-down step); `bytes` is a multiple of page_size. mlock contract
    // requires page-aligned address and multiple-of-page length per POSIX;
    // both satisfied. We never dereference start_ptr; mlock only updates
    // kernel mlock bookkeeping.
    let result = unsafe { sys_mlock_attempt(start_ptr, bytes) };
    match result {
        Ok(()) => PinnedPageRange {
            start: start_ptr,
            page_count,
        },
        Err(errno) => {
            if errno == libc::EINVAL {
                // SPEC §2 row 6: EINVAL should be unreachable from the
                // slice-fn API by construction. Debug builds trip a
                // debug_assert; release builds soft-fail like other errnos.
                debug_assert!(
                    false,
                    "mlock returned EINVAL — should be unreachable from slice-fn API by construction (page_size={}, bytes={})",
                    page_size(),
                    bytes,
                );
            }
            mlock_state().record_failure(errno, buf.len());
            PinnedPageRange::empty()
        }
    }
}

fn round_to_pages(addr: usize, len: usize, page: usize) -> (usize, usize) {
    debug_assert!(page.is_power_of_two(), "page size must be a power of two");
    if len == 0 {
        return (0, 0);
    }
    let mask = page - 1;
    let start = addr & !mask;
    let end = (addr + len + mask) & !mask;
    (start, (end - start) / page)
}

// ============================================================================
// MlockState aggregator + report_at_exit
// ============================================================================

struct MlockState {
    attempts: AtomicUsize,
    failure_count: AtomicUsize,
    total_bytes_unlocked: AtomicUsize,
    first_errno: OnceLock<i32>,
}

impl MlockState {
    const fn new() -> Self {
        Self {
            attempts: AtomicUsize::new(0),
            failure_count: AtomicUsize::new(0),
            total_bytes_unlocked: AtomicUsize::new(0),
            first_errno: OnceLock::new(),
        }
    }

    fn record_attempt(&self) {
        self.attempts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_failure(&self, errno: i32, bytes: usize) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.total_bytes_unlocked.fetch_add(bytes, Ordering::Relaxed);
        let _ = self.first_errno.set(errno);
    }
}

static MLOCK_STATE: OnceLock<MlockState> = OnceLock::new();

fn mlock_state() -> &'static MlockState {
    MLOCK_STATE.get_or_init(MlockState::new)
}

/// Emit a stderr summary iff `failure_count > 0`. Called from `main()` in
/// both `mnemonic-toolkit` and `ms-cli`. Format pinned by SPEC §6 G2.5.
pub fn report_at_exit() {
    let Some(st) = MLOCK_STATE.get() else {
        return;
    };
    let failures = st.failure_count.load(Ordering::Relaxed);
    if failures == 0 {
        return;
    }
    let attempts = st.attempts.load(Ordering::Relaxed);
    let bytes = st.total_bytes_unlocked.load(Ordering::Relaxed);
    let errno_name = st
        .first_errno
        .get()
        .map(|&e| errno_to_name(e))
        .unwrap_or("?");
    eprintln!("warning: {failures} of {attempts} secret regions could not be locked");
    eprintln!("         (first errno: {errno_name}, {bytes} bytes total); secret");
    eprintln!("         data remains in heap and may be swappable.");
    eprintln!("hint:    set RLIMIT_MEMLOCK >= 64KiB or grant CAP_IPC_LOCK");
    eprintln!("         to eliminate this warning.");
}

fn errno_to_name(errno: i32) -> &'static str {
    match errno {
        libc::EPERM => "EPERM",
        libc::ENOMEM => "ENOMEM",
        libc::EAGAIN => "EAGAIN",
        libc::EINVAL => "EINVAL",
        libc::ENOTSUP => "ENOTSUP",
        _ => "UNKNOWN",
    }
}

// ============================================================================
// Test helpers (pub for integration-test reachability; informational use
// only — production code does not call these)
// ============================================================================

/// Returns the cached process page size. Integration tests use this to
/// compute page-aligned buffer sizes without hard-coding 4096 (macOS aarch64
/// uses 16384).
pub fn page_size_for_test() -> usize {
    page_size()
}

/// Returns the current `failure_count` (atomic load) for the process-static
/// mlock-state singleton.
pub fn failure_count_for_test() -> usize {
    MLOCK_STATE
        .get()
        .map(|s| s.failure_count.load(Ordering::Relaxed))
        .unwrap_or(0)
}

/// Returns the first-recorded errno value, if any.
pub fn first_errno_for_test() -> Option<i32> {
    MLOCK_STATE.get().and_then(|s| s.first_errno.get().copied())
}

// ============================================================================
// Syscall wrappers — production / cfg(test) / cfg(miri) variants
// ============================================================================

#[cfg(miri)]
unsafe fn sys_mlock_attempt(_addr: *const u8, _len: usize) -> Result<(), i32> {
    Ok(())
}

#[cfg(miri)]
unsafe fn sys_munlock(_addr: *const u8, _len: usize) -> i32 {
    0
}

#[cfg(all(not(miri), not(test)))]
unsafe fn sys_mlock_attempt(addr: *const u8, len: usize) -> Result<(), i32> {
    // SAFETY: addr is page-aligned + len is page-multiple per pin_pages_for
    // caller; mlock is a POSIX syscall with documented semantics.
    let rc = unsafe { libc::mlock(addr as *const libc::c_void, len) };
    if rc == 0 {
        Ok(())
    } else {
        Err(last_os_errno())
    }
}

#[cfg(all(not(miri), not(test)))]
unsafe fn sys_munlock(addr: *const u8, len: usize) -> i32 {
    // SAFETY: addr came from a successful prior mlock; len matches.
    unsafe { libc::munlock(addr as *const libc::c_void, len) }
}

#[cfg(all(not(miri), test))]
unsafe fn sys_mlock_attempt(addr: *const u8, len: usize) -> Result<(), i32> {
    match fail_mode::current() {
        fail_mode::FailMode::Off => {
            // SAFETY: same as production path; addr is page-aligned + len is
            // page-multiple per pin_pages_for caller.
            let rc = unsafe { libc::mlock(addr as *const libc::c_void, len) };
            if rc == 0 {
                Ok(())
            } else {
                Err(last_os_errno())
            }
        }
        fail_mode::FailMode::EPerm => Err(libc::EPERM),
        fail_mode::FailMode::ENoMem => Err(libc::ENOMEM),
        fail_mode::FailMode::EInval => Err(libc::EINVAL),
    }
}

#[cfg(all(not(miri), test))]
unsafe fn sys_munlock(addr: *const u8, len: usize) -> i32 {
    // SAFETY: addr came from a successful prior mlock; len matches.
    unsafe { libc::munlock(addr as *const libc::c_void, len) }
}

#[cfg(not(miri))]
fn last_os_errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

// ============================================================================
// cfg(test) env-var fault injection (per SPEC §4 P2 + §6 G2)
// ============================================================================

#[cfg(test)]
mod fail_mode {
    use std::sync::OnceLock;

    pub enum FailMode {
        Off,
        EPerm,
        ENoMem,
        EInval,
    }

    pub fn parse(s: &str) -> Option<FailMode> {
        match s {
            "off" => Some(FailMode::Off),
            "eperm" => Some(FailMode::EPerm),
            "enomem" => Some(FailMode::ENoMem),
            "einval" => Some(FailMode::EInval),
            _ => None,
        }
    }

    static FAIL_MODE: OnceLock<FailMode> = OnceLock::new();

    pub fn current() -> &'static FailMode {
        FAIL_MODE.get_or_init(|| {
            std::env::var("MNEMONIC_TEST_MLOCK_FAIL_MODE")
                .ok()
                .as_deref()
                .and_then(parse)
                .unwrap_or(FailMode::Off)
        })
    }
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_rounding_formula_single_page() {
        let page = 4096;
        let (start, count) = round_to_pages(0x1234, 64, page);
        assert_eq!(start, 0x1000, "round down to page boundary");
        assert_eq!(count, 1, "small buf fits in 1 page");
    }

    #[test]
    fn page_rounding_formula_multi_page() {
        let page = 4096;
        let (start, count) = round_to_pages(0x1000, 2 * page + 1, page);
        assert_eq!(start, 0x1000);
        assert_eq!(count, 3, "2*page+1 spans 3 pages when starting page-aligned");
    }

    #[test]
    fn page_rounding_formula_zero_length() {
        let page = 4096;
        let (start, count) = round_to_pages(0x1234, 0, page);
        assert_eq!(start, 0, "zero-length yields empty range");
        assert_eq!(count, 0);
    }

    #[test]
    fn page_rounding_formula_exactly_one_page_aligned() {
        let page = 4096;
        let (start, count) = round_to_pages(0x2000, page, page);
        assert_eq!(start, 0x2000, "already page-aligned");
        assert_eq!(count, 1, "exactly-one-page buf at aligned address yields 1");
    }

    #[test]
    fn mlockstate_record_failure_idempotent_on_first_errno() {
        let st = MlockState::new();
        st.record_failure(libc::EPERM, 64);
        st.record_failure(libc::ENOMEM, 128);
        assert_eq!(
            st.first_errno.get().copied(),
            Some(libc::EPERM),
            "first_errno is set once and stays",
        );
    }

    #[test]
    fn mlockstate_record_failure_monotonic_on_counters() {
        let st = MlockState::new();
        st.record_attempt();
        st.record_attempt();
        st.record_attempt();
        st.record_failure(libc::EPERM, 64);
        st.record_failure(libc::ENOMEM, 128);
        assert_eq!(st.attempts.load(Ordering::Relaxed), 3);
        assert_eq!(st.failure_count.load(Ordering::Relaxed), 2);
        assert_eq!(st.total_bytes_unlocked.load(Ordering::Relaxed), 192);
    }

    /// G4.a — Zeroize-on-Drop discipline composes with pin_pages_for.
    /// Verifies a pinned `Vec<u8>` can be zeroized in place (via the
    /// `zeroize` crate's `Zeroize` impl for Vec — which scrubs the data
    /// buffer then clears len to 0) without panicking, then the pin drops
    /// cleanly. Avoids post-zeroize reads (which would be UB-adjacent since
    /// `Vec::zeroize` calls `clear()` invalidating len-based indexing).
    #[test]
    fn g4_a_pin_and_zeroize_compose_without_panic() {
        use zeroize::Zeroize;
        let mut v: Vec<u8> = vec![0xAAu8; 64];
        assert_eq!(v[0], 0xAA);
        let pin = pin_pages_for(&v);
        assert_eq!(pin.page_count, 1, "64-byte buf pins exactly one page");
        v.zeroize();
        assert_eq!(v.len(), 0, "zeroize clears Vec len after scrubbing");
        drop(pin); // munlock after zeroize — strictest threat-model ordering
    }

    // ========================================================================
    // G2.x — fault-injection acceptance gates.
    //
    // These live as LIBRARY UNIT tests (not integration tests in tests/)
    // because `cfg(test)` is per-crate-not-per-build (RFC 1604; flagged by
    // Phase 2 R0 v1 I-R0-4): the `FAIL_MODE` injection hook in this module
    // is only reachable when the LIBRARY itself is compiled with cfg(test),
    // which only happens for the `--lib` test target.
    //
    // Each G2.x test is `#[ignore]`-gated. CI invokes them individually in
    // separate cargo processes so `OnceLock<FailMode>` initializes from a
    // fresh env-var read per process:
    //
    //   MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm cargo test --lib mlock::tests::g2_1 -- --include-ignored
    //   MNEMONIC_TEST_MLOCK_FAIL_MODE=einval cargo test --lib mlock::tests::g2_3 -- --include-ignored
    //   MNEMONIC_TEST_MLOCK_FAIL_MODE=off cargo test --lib mlock::tests::g2_4 -- --include-ignored
    //
    // Filtering to the specific test name guarantees the test target's
    // process runs exactly one mlock test, so the OnceLock doesn't get
    // polluted by a prior call from a different test.
    // ========================================================================

    /// G2.1 — eperm fault injection. Asserts `MlockState.failure_count` is
    /// incremented and `first_errno` is recorded as `EPERM` after exactly
    /// one `pin_pages_for` call under `FAIL_MODE=eperm`. Requires
    /// `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` set in the env BEFORE this
    /// test process starts.
    #[test]
    #[ignore = "subprocess: requires MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm in env"]
    fn g2_1_eperm_increments_failure_count() {
        let buf = vec![0u8; 64];
        let _pin = pin_pages_for(&buf);
        assert!(
            failure_count_for_test() > 0,
            "FAIL_MODE=eperm must increment failure_count via record_failure",
        );
        assert_eq!(
            first_errno_for_test(),
            Some(libc::EPERM),
            "first_errno must be EPERM after eperm injection",
        );
    }

    /// G2.3-debug — EINVAL fault injection. Debug builds trip `debug_assert!`
    /// inside `pin_pages_for` (SPEC §2 row 6: EINVAL should be unreachable
    /// from the slice-fn API by construction; debug builds panic, release
    /// builds soft-fail). Requires `MNEMONIC_TEST_MLOCK_FAIL_MODE=einval`
    /// set in the env BEFORE this test process starts.
    #[test]
    #[ignore = "subprocess: requires MNEMONIC_TEST_MLOCK_FAIL_MODE=einval in env"]
    #[should_panic(expected = "EINVAL")]
    fn g2_3_einval_debug_panics() {
        let buf = vec![0u8; 64];
        let _pin = pin_pages_for(&buf);
        // unreachable in debug builds — debug_assert! fires inside pin_pages_for
    }

    /// G2.4 — control: FAIL_MODE=off must NOT synthesize failures. After
    /// one `pin_pages_for` call with sufficient ulimit, `failure_count`
    /// remains 0. Requires `MNEMONIC_TEST_MLOCK_FAIL_MODE=off` (or unset)
    /// + Linux `ulimit -l >= 64KiB` (CI sets this) or macOS default.
    #[test]
    #[ignore = "subprocess: requires MNEMONIC_TEST_MLOCK_FAIL_MODE=off + sufficient ulimit"]
    fn g2_4_off_no_synthesized_failures() {
        let buf = vec![0u8; 64];
        let _pin = pin_pages_for(&buf);
        assert_eq!(
            failure_count_for_test(),
            0,
            "FAIL_MODE=off must not synthesize failures (test env requires ulimit -l >= 64KiB)",
        );
    }
}
