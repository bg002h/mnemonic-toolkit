# v0.36.0 — Per-phase code review (Phases 3+4: decode-address + verify-message)

**Date:** 2026-05-23
**Reviewer:** opus (feature-dev:code-reviewer), per-phase (agentId a5318751bcf58dd6f)
**Scope:** `decode_address.rs` + `cmd/decode_address.rs` + `verify_message.rs` + `cmd/verify_message.rs` + `error.rs` (2 variants) + integration tests; reviewer also read the vendored `bip322 0.0.10` source.

---

## Critical

**C1 — `verify-message` can be CRASHED (panic / exit 101) by a crafted P2SH address + valid-uncompressed-pubkey BIP-322 witness.** (Confidence 90 → CONFIRMED by reproduction.)
`verify_bip322` isolated the crate with `.is_ok()`, which catches `Err` but NOT a panic. The pinned `bip322 0.0.10` panics at `verify.rs:168` `ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap())` when the P2SH arm (`verify.rs:87-94`, reached for ANY P2SH address with `witness.len()>1`) processes a witness whose item[1] is a valid 65-byte **uncompressed** pubkey (`wpubkey_hash()` → `Err(UncompressedPublicKeyError)` → `.unwrap()` panics). Reachable via `auto` (P2SH ≠ P2PKH → bip322) or `--format bip322`. All attacker-controlled public input. No `catch_unwind`/`panic=abort` → unwinds through `run`→`main`, crash exit 101. Violates the documented exit convention (malformed → clean exit-1).
**Reproduced:** `verify_message.rs` test `p2sh_uncompressed_pubkey_does_not_panic` with a real uncompressed pubkey + grind-to-71/72-byte DER sig → `panicked at bip322-0.0.10/src/verify.rs:168:55: called Result::unwrap() on an Err value: UncompressedPublicKeyError`. (My first trigger attempt used invalid bytes that the crate rejected earlier via `from_slice`/`from_der` — confirming the panic needs a *valid* uncompressed key + *parseable* DER sig.)

## Important
None.

## Minor
**M1 — No test for the P2SH adversarial path** (the gap that let C1 ship). Add a regression test.
**M2 — `format_requested` via `format!("{:?}",…)` Debug** (confidence 30, below threshold; noted only — a future multi-word variant would emit "bip322full"). Not actionable now.

## Confirmed correct (excerpt)
Address-type partition sound (legacy=P2pkh gate honest; P2WSH/P2A/future → bip322 → clean `UnsupportedAddress`→valid:false); `verify_legacy` digest+recovery correct; invalid-sig→valid:false (non-panic); decode-address panic-safe (assume_checked/script_pubkey/witness_program/is_valid_for_network all total; no unwrap); scriptPubKey hex correct vs BIP-173/350 vectors; error.rs alphabetical, exit 1, kind strings match, no details arm; exit-code mapping (main.rs:161-162 `Ok(code)=>ExitCode::from(code)`); message-handling asymmetry documented + not a hazard; secret taxonomy correctly omits all new flags (public-data); legacy self-gen vector legitimate (signs via secp, verifies via independent is_signed_by_address path); BIP-322 vectors are genuine mediawiki/crate vectors; manual lockstep complete; gui-schema auto-discovers via CommandFactory.

VERDICT: RED (1C/0I)

---

## Fold disposition (controller) — per-phase → re-review
**C1 FIXED:** `verify_bip322` now returns `Result<bool, ToolkitError>` and wraps the crate call in `std::panic::catch_unwind` (with scoped panic-hook silencing so the crate's panic text doesn't leak to stderr; hook restored immediately after — CLI is single-threaded; only the one regression unit test exercises the catch in the shared test binary). A caught panic maps to a clean `ToolkitError::VerifyMessage` ("…malformed or unsupported witness") → exit 1, no crash. Callers updated with `?`.
**M1 FIXED:** `p2sh_uncompressed_pubkey_does_not_panic` regression test added (reproduces the exact panic pre-fix; passes post-fix). Full suite 2348 pass (+1), clippy clean.
**M2:** acknowledged, deferred (below confidence threshold; no multi-word format variant exists).
Re-dispatching to reviewer to confirm the C1 fix.
