//! v0.41.0 — `--slot @N.ms1=` integration tests.
//!
//! Phase 1 scope: the canonical-mode descriptor gate. A secret-bearing slot
//! (`ms1` / `seedqr` / `phrase`) carrying an explicit `@N.path=` against a
//! CANONICAL descriptor (`wsh(sortedmulti(...))`, whose `canonical_origin`
//! supplies the per-shape default path) is refused with a
//! `SlotInputViolation{kind:"conflict"}` — exit 2 — because the canonical
//! descriptor already pins the origin path; an explicit per-`@N` path would
//! conflict with it.
//!
//! These fire on the `@0.<secret> + @0.path` subkey set BEFORE the per-cosigner
//! binding loop, so `@1` only needs to be well-formed (a valid xpub) to clear
//! the missing-`@1` check.

use assert_cmd::Command;
use predicates::prelude::*;

/// Canonical 2-of-2 sorted-multisig descriptor (canonical_origin maps it, so
/// it is NOT treated as non-canonical / explicit-origin).
const CANONICAL_DESC: &str = "wsh(sortedmulti(2,@0,@1))";

/// A well-formed mainnet xpub for the `@1` cosigner (from
/// `cli_export_wallet_coldcard.rs`). Keeps the command from being rejected for
/// a missing `@1` so the canonical gate on `@0` is what fires.
const VALID_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";

/// The canonical-mode conflict message (verbatim from
/// `cmd::bundle.rs` / `slot_input.rs`).
const CONFLICT_MSG: &str =
    "has both secret-bearing input and watch-only input; pick one per slot.";

/// `@0.ms1=<...> + @0.path=<...>` against a CANONICAL descriptor → exit 2,
/// SlotInputViolation conflict. The gate fires on the subkey set; the ms1
/// value need not decode (the gate precedes the binding loop).
#[test]
fn ms1_plus_path_canonical_descriptor_refused_exit2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            CANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            "@0.ms1=ms1stubvalue",
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={VALID_XPUB}"),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(CONFLICT_MSG));
}

/// `@0.seedqr=<...> + @0.path=<...>` against a CANONICAL descriptor → exit 2,
/// SlotInputViolation conflict.
///
/// Baseline note (plan Task 1.4 R0-I2): pre-fix the canonical gate only matched
/// `has_phrase && has_path`, so a `[Seedqr, Path]` set fell through to the
/// per-cosigner binding loop and surfaced as an exit-1 BadInput. The widened
/// `(has_phrase || has_seedqr || has_ms1) && has_path` gate normalizes it to
/// the exit-2 SlotInputViolation. Assert exit 2.
#[test]
fn seedqr_plus_path_canonical_descriptor_refused_exit2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            CANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            "@0.seedqr=000100020003000400050006000700080009001000110012",
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={VALID_XPUB}"),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(CONFLICT_MSG));
}
