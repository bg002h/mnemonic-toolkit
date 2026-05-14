//! POSIX `mlock(2)` page-pinning for OWNED secret-bearing heap buffers.
//!
//! Cycle B Phase 2 (mlock infrastructure). See
//! `design/SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2 for the locked
//! Fix-B design: slice-fn primitive only (no wrapper type).
//!
//! Module surface (locked in P2.T1 R0 + post-Fix-B verify):
//! - `pub fn pin_pages_for(buf: &[u8]) -> PinnedPageRange` — pin the heap
//!   pages covering `buf`. Zero-length is a no-op.
//! - `pub struct PinnedPageRange { start, page_count }` with Drop = munlock.
//! - `pub fn report_at_exit()` — emit 2-line stderr summary iff
//!   `failure_count > 0`. Called from `main()`.
//! - Internal: `fn page_size()` cached in `OnceLock<usize>`;
//!   `MlockState` aggregator; `record_failure`; `cfg(test)` env-var
//!   fault injection; `cfg(miri)` syscall shims.

// TODO P2.T3 (GREEN): full implementation per SPEC §4 P2.
