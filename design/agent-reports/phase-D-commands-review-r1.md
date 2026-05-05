# Phase D — command modules Review — r1

**Date:** 2026-05-05
**Commit under review:** `7e8f050` (parent: `738437d`)
**Reviewer:** opus phase-review

## Verdict

1 critical / 1 important / 0 low / 0 nits — both fixed inline post-r1 at commit `4b90b5e`. Cleared to advance to Phase E release prep.

## Critical (FIXED inline post-r1)

### C-1: Multisig check names missing `[i]` index suffix — SPEC §5.4 violated

**Files:** `crates/mnemonic-toolkit/src/format.rs:136` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_multisig`

SPEC §5.4 mandates indexed names (`mk1_decode[0]`, `mk1_xpub_match[0]`, etc.) for multisig per-cosigner check slots. Phase D's `run_multisig` emitted plain non-indexed names (every `mk1_decode` for N cosigners shared the same `name` string). Compounded: `VerifyCheck.name: &'static str` blocked `format!("mk1_decode[{}]", i)`.

**Fix (applied at `4b90b5e`):**
- `VerifyCheck.name: &'static str` → `String`. 73 construction sites updated to use `.into()` for plain literals or `format!("X[{}]", i)` for per-cosigner indexed names.
- `run_multisig` per-cosigner slots emit `format!("X[{}]", i)`; singleton checks (`ms1_entropy_match`, `md1_decode`, `md1_wallet_policy`) keep plain names per SPEC §5.4.
- Single-sig paths (`run_full`, `run_watch_only`, `watch_only_checks`) keep plain names — v0.1 integration test `verify_bundle_json_emits_9_checks_in_spec_order` still passes.
- Smoke test for N=3 multisig: 21 = 3 + 6·3 checks emitted in SPEC order.

## Important (FIXED inline post-r1)

### I-1: Dead `passphrase` binding in `run_multisig` ms1 entropy check

**File:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_multisig` (was lines 996-999)

`let passphrase = args.passphrase.clone().unwrap_or_default();` followed by `let _ = passphrase;` — code smell suggesting passphrase affects entropy verification (it doesn't; BIP-39 entropy is pre-passphrase by definition).

**Fix (applied at `4b90b5e`):** removed the binding + suppression; replaced with a one-line comment explaining BIP-39 entropy is passphrase-independent.

## Verified

- `BundleJson` field reshape (D.4): `origin_path: Option<String>` + NEW `origin_paths: Option<Vec<String>>` + `master_fingerprint: Option<String>`. Field order matches SPEC §5.3.
- Single-sig emit populates `origin_path: Some(...)`, `origin_paths: None`, `master_fingerprint: Some(...)`. Multisig shared-path emit populates `origin_path: Some(shared)`, `origin_paths: None`. Multisig divergent populates `origin_path: None`, `origin_paths: Some(...)`. Multisig + privacy → `master_fingerprint: None` unconditionally.
- Existing `cli_json_envelopes` test passes (Option<String> serializes as JSON string when Some).
- `watch_only_checks` extended with `privacy_preserving: bool`; emits `mk1_fingerprint_match: skipped` with detail `"privacy-preserving mode; fingerprint suppressed"`. Both single-sig + multisig paths honor the flag.
- `run_multisig` emits exactly `3 + 6N` checks in SPEC §5.4 order. For N=2: 15. For N=3: 21.
- Cosigner association via xpub-vs-`tlv.pubkeys` lookup; positional fallback for self-multisig (all xpubs identical).
- Self-multisig CSI-collision edge case: chunk grouping yields one group instead of N → existing group-count check fires `BundleMismatch`. SPEC-acknowledged self-multisig wart; no code action needed.
- v0.1 single-sig `run_full` + `run_watch_only` 9-element schema unchanged (plain names preserved).
- Phase E scope discipline: no fixture matrix / Cargo.toml metadata / CHANGELOG / SPEC §9.4 / tag work in Phase D.

## Smoke checks (post-fixup)

- `cargo test -p mnemonic-toolkit`: 95 passing.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean.
- `cargo fmt --check -p mnemonic-toolkit`: clean.
- v0.1 wire-bit-identical regression: 16/16 PASS.
- N=3 multisig verify smoke test: 21 indexed checks (`mk1_decode[0]`, `[1]`, `[2]`, etc.) emitted in SPEC order.
