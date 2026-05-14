//! `mnemonic-toolkit` library surface.
//!
//! The toolkit's primary artifact is the `mnemonic` binary (see
//! `crates/mnemonic-toolkit/src/main.rs`). This library exposes a focused
//! set of modules so integration tests and the binary itself can use
//! `mnemonic_toolkit::<module>::*`. All other binary modules stay private
//! to `main.rs`.
//!
//! See `design/SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2 for the locked
//! crate-shape decision (Option C: hybrid lib + bin).
//!
//! Exposed modules:
//! - `mlock` (Cycle B Phase 2): page-pinning primitives.
//! - `final_word` (v0.11.0 P1): BIP-39 final-word completer library.
//!   Defines a small, self-contained `FinalWordError` so the library
//!   surface does not pull in the binary-private `ToolkitError`. The CLI
//!   handler in `src/cmd/final_word.rs` (P2) converts `FinalWordError`
//!   into `ToolkitError` at the boundary.
//! - `seed_xor` (v0.12.0 P1): Coldcard-compatible BIP-39 ↔ BIP-39
//!   all-or-nothing XOR-based seed splitter. Defines a library-local
//!   `SeedXorError` per the same pattern as `final_word`. The CLI
//!   handler in `src/cmd/seed_xor.rs` (P2) converts `SeedXorError`
//!   into `ToolkitError` at the boundary.

pub mod final_word;
pub mod mlock;
pub mod seed_xor;
