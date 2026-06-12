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
//! - `slip39` (v0.13.0): Trezor SLIP-0039 hierarchical K-of-N Shamir
//!   Secret Sharing. Multi-module subdirectory. Defines a library-local
//!   `Slip39Error` per the same pattern as `seed_xor`. The CLI handler
//!   in `src/cmd/slip39.rs` (P2, future) converts `Slip39Error` into
//!   `ToolkitError` at the boundary.
//! - `seedqr` — SeedQR encode/decode subcommand (v0.30.0). Defines a
//!   small, self-contained `SeedqrError` so the library surface does
//!   not pull in the binary-private `ToolkitError`. The CLI handler in
//!   `src/cmd/seedqr.rs` (P2) converts `SeedqrError` into
//!   `ToolkitError::BadInput` at the boundary via
//!   `map_seedqr_error(e, action)`.
//! - `electrum_crypto` — Electrum field-level encryption decrypt + encrypt
//!   primitives (v0.31.0 / Cycle 6a). Implements Electrum's
//!   `pw_encode_bytes` / `pw_decode_bytes` (Format A field-level
//!   encryption: `sha256d(password) + AES-256-CBC + PKCS7 + base64`).
//!   Defines a library-local `ElectrumDecryptError` per the same
//!   pattern. The CLI handler in `src/cmd/import_wallet.rs` (Cycle 6b
//!   Phase 3) converts via a boundary mapper to `ToolkitError::BadInput`
//!   at orchestrator pre-decrypt time. Format B (whole-file storage
//!   encryption) is out of scope; tracked as FOLLOWUP
//!   `wallet-import-electrum-encrypted-storage-format-b`.
//! - `bsms_crypto` — BIP-129 encryption-envelope crypto primitives
//!   (v0.31.0 / Cycle 7a). Implements PBKDF2-SHA512 + AES-256-CTR +
//!   HMAC-SHA256 per BIP-129 §Encryption. Defines a library-local
//!   `BsmsCryptoError` per the same pattern. The CLI handler in
//!   `src/cmd/import_wallet.rs` (Cycle 7b Phase 3) will convert via
//!   a boundary mapper to `ToolkitError::BadInput` at orchestrator
//!   pre-decrypt time. Used standalone (no CLI consumer) until
//!   Cycle 7b ships the `--bsms-encryption-token` flag.
//! - `secret_taxonomy` (v0.14.0): public `pub const &[&str]` arrays of
//!   secret-class node / slot-subkey token strings. Mirrors the
//!   private `NodeType::is_secret_bearing` /
//!   `SlotSubkey::is_secret_bearing` predicates; downstream consumers
//!   (e.g., `mnemonic-gui`'s `persistence::redact_for_persistence`)
//!   import these instead of source-scraping at build time. Single
//!   source of truth enforced at toolkit test time via parity tests
//!   on `cmd::convert::NodeType` and `slot_input::SlotSubkey`.
//!   **Stability contract:** these slices form load-bearing public
//!   API for the GUI's persistence redaction. Renaming, reordering,
//!   or removing entries is a semver-minor event (pre-1.0 0.X-axis
//!   bump); adding entries is additive and minor-safe. Consumed by
//!   `mnemonic-gui` v0.4.0+.

pub mod bsms_crypto;
pub mod electrum_crypto;
pub mod final_word;
// `mlock` uses POSIX `libc::mlock` / `libc::munlock` / `libc::sysconf` /
// `_SC_PAGESIZE`. None of those symbols exist in `libc`'s Windows
// surface (Windows has `VirtualLock`; libc-rs's Windows surface is
// CRT-only). Pre-v0.14.0 the toolkit was binary-only and never
// compiled on Windows. v0.14.0 promoted `secret_taxonomy` to public
// lib API for `mnemonic-gui` consumption, which transitively required
// the entire lib to compile on every platform the GUI targets —
// including Windows. Cfg-gate keeps `mlock` available on Unix
// (its existing consumer surface) while letting the lib compile on
// Windows. Closes the architect-flagged Critical at GUI v0.4.0 CI.
#[cfg(unix)]
pub mod mlock;
pub mod secret_taxonomy;
// v0.34.7: process-level argv-hardening (PR_SET_DUMPABLE). Unconditional —
// the body is `#[cfg(target_os = "linux")]`-gated (no-op elsewhere), so this
// compiles on the GUI's Windows lib consumption.
pub mod process_hardening;
/// v0.24.0 Tranche B.1: authoritative `flag_is_secret` predicate consumed by
/// the gui-schema v5 envelope emitter (`cmd::gui_schema`). Mirror in
/// `mnemonic-gui/src/secrets.rs` for v0.5..v0.9 hand-coded-schema fallback;
/// GUI-side drift gate asserts the two lists agree.
pub mod secrets;
/// Serialize-transparent, zeroize-on-drop secret string for derived
/// private-key material emitted via `--json` / text (silent-payment, nostr).
pub mod secret_string;
pub mod seed_xor;
pub mod seedqr;
pub mod slip39;

// ---------------------------------------------------------------------------
// `cfg(fuzzing)`-ONLY mount of the `parse_descriptor` closure.
// ---------------------------------------------------------------------------
//
// The fuzz target at `fuzz/fuzz_targets/descriptor_parse.rs` drives the
// binary-private `parse_descriptor::parse_descriptor` (the toolkit's untrusted
// descriptor-string intake). That function and the 19 modules its transitive
// `crate::` paths reference are declared in `main.rs` (the bin crate), NOT in
// this lib — per the locked Option C crate shape
// (`SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2). A path-dep fuzz crate can
// only reach lib-crate items, so we mount the closure here UNDER `cfg(fuzzing)`.
//
// `cfg(fuzzing)` is set ONLY by cargo-fuzz (it passes `--cfg fuzzing` via
// RUSTFLAGS). In EVERY normal build — `cargo build`/`cargo test`, CI rust.yml,
// the shipped `mnemonic` binary — this whole block is ENTIRELY ABSENT, so it has
// zero effect on the shipped surface. The lone normal-build-visible change is
// the `[lints.rust] unexpected_cfgs` line in `Cargo.toml`, which only stops
// clippy `-D warnings` from flagging the otherwise-unknown `cfg(fuzzing)`.
//
// `extern crate self as mnemonic_toolkit;` (load-bearing): three shared
// bin+lib files in the closure (`derive.rs`, `synthesize.rs`, `derive_slot.rs`)
// reach `mlock` via the EXTERNAL-crate self-name path
// (`mnemonic_toolkit::mlock::…`) because `main.rs` reaches `mlock` only via that
// path. Aliasing self under `cfg(fuzzing)` makes the self-name resolve inside
// the lib too, with zero edits to the shared files.
//
// `error.rs` is mounted here, so `ToolkitError` becomes `pub` UNDER
// `cfg(fuzzing)` ONLY. It remains binary-private in every normal/shipped build,
// so the "binary-private by design" invariant is preserved (the mount never
// compiles in a normal build).
#[cfg(fuzzing)]
extern crate self as mnemonic_toolkit;

#[cfg(fuzzing)]
pub mod cost;
#[cfg(fuzzing)]
pub mod derive;
#[cfg(fuzzing)]
pub mod derive_address;
#[cfg(fuzzing)]
pub mod derive_slot;
#[cfg(fuzzing)]
pub mod error;
#[cfg(fuzzing)]
pub mod format;
#[cfg(fuzzing)]
pub mod friendly;
#[cfg(fuzzing)]
pub mod indel;
#[cfg(fuzzing)]
pub mod language;
#[cfg(fuzzing)]
pub mod network;
#[cfg(fuzzing)]
pub mod parse;
#[cfg(fuzzing)]
pub mod parse_descriptor;
#[cfg(fuzzing)]
pub mod repair;
#[cfg(fuzzing)]
pub mod secret_advisory;
#[cfg(fuzzing)]
pub mod slip0132;
#[cfg(fuzzing)]
pub mod slot_input;
#[cfg(fuzzing)]
pub mod synthesize;
#[cfg(fuzzing)]
pub mod template;
#[cfg(fuzzing)]
pub mod wallet_export;
