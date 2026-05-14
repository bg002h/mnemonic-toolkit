//! `mnemonic-toolkit` library surface.
//!
//! The toolkit's primary artifact is the `mnemonic` binary (see
//! `crates/mnemonic-toolkit/src/main.rs`). This library exposes only the
//! `mlock` module so that integration tests and the binary itself can
//! `use mnemonic_toolkit::mlock::*`. All other binary modules stay private
//! to `main.rs`.
//!
//! See `design/SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2 for the locked
//! crate-shape decision (Option C: hybrid lib + bin).

pub mod mlock;
