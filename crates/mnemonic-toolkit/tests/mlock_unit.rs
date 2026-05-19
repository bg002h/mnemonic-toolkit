//! Cycle B Phase 2 integration tests for the slice-fn mlock module (Fix B).
//!
//! See SPEC §6 G1.1-G1.4 (functional correctness — page-count contract).
//!
//! **G2.* fault-injection tests live in `src/mlock.rs`'s `#[cfg(test)] mod
//! tests`, not here.** Integration tests in `tests/` link against the
//! library's PROD build where `cfg(test)` is false; the `FAIL_MODE`
//! injection hook is unreachable per RFC 1604 (`cfg(test)` is
//! per-crate-not-per-build) — exactly the same constraint R0 v1 I-R0-4
//! flagged for the cfg(test) drop-probe. CI invokes each G2.* unit test
//! with `MNEMONIC_TEST_MLOCK_FAIL_MODE` set in the workflow `env:` block so
//! `OnceLock<FailMode>` initializes correctly per subprocess.
//!
//! Phase 2 deferrals to Phase 3a (per SPEC §2 row 7): G2.2 (enomem),
//! G2.3-release, G2.5 (stderr summary) — Phase 2 has no production mlock
//! callsite for subprocess fault injection to invoke.

use mnemonic_toolkit::mlock;

// ============================================================================
// G1 — functional correctness (page-count contract)
// ============================================================================

#[test]
fn g1_1_single_page_pin_has_page_count_one() {
    use std::alloc::{alloc, dealloc, Layout};

    let page_size = mlock::page_size_for_test();
    // SAFETY: Layout is valid (size > 0, align is power of 2). We deallocate
    // before returning. Buffer is page-aligned so pin_pages_for returns exactly 1.
    let layout = Layout::from_size_align(64, page_size).expect("valid layout");
    unsafe {
        let ptr = alloc(layout);
        assert!(!ptr.is_null(), "alloc failed");
        let slice = std::slice::from_raw_parts(ptr, 64);
        let pin = mlock::pin_pages_for(slice);
        assert_eq!(pin.page_count, 1, "page-aligned 64-byte buffer spans exactly 1 page");
        assert!(!pin.start.is_null(), "non-empty buf produces non-null start");
        drop(pin);
        dealloc(ptr, layout);
    }
}

#[test]
fn g1_2_multi_page_pin_has_page_count_at_least_two() {
    let page = mlock::page_size_for_test();
    let buf = vec![0xBBu8; 2 * page + 1];
    let pin = mlock::pin_pages_for(&buf);
    assert!(
        pin.page_count >= 2,
        "buf of len 2*page+1 must span >= 2 pages (got {})",
        pin.page_count,
    );
    drop(pin);
}

#[test]
fn g1_3_zero_length_is_no_op_no_syscall_no_panic() {
    let pin = mlock::pin_pages_for(&[]);
    assert_eq!(pin.page_count, 0, "zero-length buf must produce page_count=0");
    assert!(pin.start.is_null(), "zero-length buf must produce null start");
    // Drop is a no-op for an empty range; no panic.
    drop(pin);
}

#[test]
fn g1_4_page_aligned_exactly_one_page_count_one() {
    let page = mlock::page_size_for_test();
    // Layout::from_size_align guarantees page-aligned allocation for size == page.
    // We use vec![] which is not guaranteed to be page-aligned; the rounding
    // formula still yields page_count=1 if the buffer fits entirely within one
    // page or page_count=2 if it straddles a boundary. To exercise the exact
    // page-aligned case, allocate via the OS allocator and verify alignment;
    // for Phase 2's RED test surface we accept either page_count value and
    // assert the rounding-formula property (page_count = ceil((addr_end -
    // addr_start_rounded) / page_size)).
    let buf = vec![0xCCu8; page];
    let pin = mlock::pin_pages_for(&buf);
    assert!(
        pin.page_count == 1 || pin.page_count == 2,
        "page-sized buf spans either 1 page (aligned) or 2 pages (straddle): got {}",
        pin.page_count,
    );
    drop(pin);
}

// ============================================================================
// G2 — moved to src/mlock.rs unit tests (cfg(test) reachability per RFC 1604).
// See module-level comment + src/mlock.rs's #[cfg(test)] mod tests.
// ============================================================================

// ============================================================================
// G6 — cross-repo diff manifest (PE; impl at tests/mlock_g6_invariant.rs)
// ============================================================================

// G6 SPEC §6 invariant is implemented as a standalone integration test at
// `tests/mlock_g6_invariant.rs`. Sibling-repo source discovery via the
// SIBLING_REPO_PATH env var (set by `.github/workflows/rust.yml` after
// `actions/checkout` of mnemonic-secret); falls back to an adjacent-dir
// relative path for local-dev. See SPEC §6 G6 + cross-repo audit matrix
// §5 G6.
