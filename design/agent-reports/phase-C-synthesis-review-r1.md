# Phase C — synthesis expansion Review — r1

**Date:** 2026-05-05
**Commit under review:** `830beb5` (parent: `e36d55f`)
**Reviewer:** opus phase-review

## Verdict

1 critical / 2 important / 2 low / 0 nits — all 3 C/I fixed inline post-r1; cleared to advance to Phase D after fixup commit.

## Critical (FIXED inline post-r1)

### C-1: `EngravingMode::FullMultisig` hardcoded `passphrase: not used` — silent passphrase misstatement

**File:** `crates/mnemonic-toolkit/src/format.rs:220` + `crates/mnemonic-toolkit/src/cmd/bundle.rs:646-651`

`EngravingMode::FullMultisig` carried no passphrase boolean; the format arm always emitted `passphrase: not used`. A multisig full-mode invocation with `--passphrase` would silently misstate the passphrase usage, which would lead the engraver to omit the passphrase hazard from the physical backup. SPEC §5.2 multisig section requires the same conditional as single-sig.

**Fix (applied):** Added `passphrase_used: bool` to `EngravingMode::FullMultisig`; format arm now branches on this flag and emits the appropriate text. `bundle_multisig_full` passes `passphrase_used: !passphrase.is_empty()`. Verified via `cargo run --bin mnemonic bundle --passphrase TREZOR --template wsh-sortedmulti --threshold 2 --cosigner-count 3` — engraving card now correctly emits `passphrase: USED — not engraved on any card; record separately and never lose it.`

## Important (FIXED inline post-r1)

### I-1: `watch_only_checks` was private (`fn`); should be `pub(crate)` per PLAN C.4

**File:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:595`

PLAN C.4 specified `watch_only_checks` would be `pub(crate)` so `self_check_bundle` could call it directly. The Phase C impl declared it private and `self_check_bundle` reimplemented inline. Functionally correct for single-sig but creates two parallel check-logic code paths.

**Fix (applied):** changed signature to `pub(crate) fn watch_only_checks(...)`. `self_check_bundle`'s inline implementation kept for now (Phase D may consolidate or leave as-is); Phase D's `3+6N` enumeration work has the option to call `watch_only_checks` directly.

### I-2: `BundleJson.origin_path` empty for multisig (TODO marker added; full fix Phase D)

**File:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:808`

`emit_multisig` constructs `BundleJson { origin_path: String::new(), master_fingerprint: String::new(), ... }`. SPEC §5.3 v0.2 requires `origin_path` for shared paths (or `origin_paths: Vec<String>` for divergent). Per-cosigner paths are correctly populated under `multisig_info.cosigners[*].origin_path`; the top-level field is the JSON consumer entry-point that's still empty.

**Fix (applied):** Added explicit `TODO(Phase D)` comment block at the construction site flagging both `origin_path` (populate when shared) and top-level `master_fingerprint` (currently empty for multisig — per SPEC §5.3 it's `null` for multisig). Phase D's "JSON envelope construction polish" task (D.4) owns the populate work.

## Low (deferred to Phase D / FOLLOWUPS)

- **L-1**: `self_check_bundle` for `MkField::Multi` skips per-cosigner xpub/fingerprint/path checks — covered by Phase D's deferred `3+6N` schema expansion. Add to FOLLOWUPS at v0.2-nice-to-have tier (Phase D-resolve).
- **L-2**: `verify_bundle::run_multisig` uses non-indexed `mk1_decode` check names; SPEC §5.4 pins names as `mk1_decode[i]`. Same Phase D deferred item; same FOLLOWUPS handling.

Both lows are part of the implementer's explicit DONE_WITH_CONCERNS scope-narrowing — Phase C delivers happy path + mismatch detection, Phase D expands to per-cosigner-named schema.

## Verified

- `Bundle.mk1` reshape `Vec<String>` → `MkField` complete; single-sig wraps in `MkField::Single`; v0.1 wire-bit-identical regression preserved (16/16 PASS).
- `MkField::Single` byte-identical serde via `#[serde(untagged)]` — confirmed by Phase B unit test.
- `synthesize_multisig_full` validates K/N range; derives N xpubs (self-multisig identical xpubs); per-cosigner CSI via `derive_mk1_chunk_set_id(&stubs[i])`; Bundle.mk1 = `MkField::Multi(...)`; cross-binding `debug_assert!`s preserved.
- `synthesize_multisig_watch_only` per-cosigner network/xpub cross-check (§4.3); path/xpub depth consistency (§4.5); `Shared`/`Divergent` selection correct; per-cosigner KeyCard emission with `privacy_preserving` honored.
- `self_check_bundle` lives in `cmd/bundle.rs` per PLAN; failure → exit 4 `BundleMismatch { card: format!("self-check[{}]", ...), message }`.
- SELF-MULTISIG WARNING byte-exact text per SPEC §4.1 4-line block; emitted only for `cosigner_count > 1`; emitted BEFORE bundle stdout; not suppressed by `--no-engraving-card`.
- `EngravingMode::FullMultisig` + `WatchOnlyMultisig` text per SPEC §5.2; multi-cosigner paths collapse to "shared" when identical else listed per-cosigner; HARDWARE WALLET CAVEAT for tr-multi-a / tr-sortedmulti-a only; SELF-MULTISIG line in engraving card for full multisig N>1.
- `verify_bundle::run_multisig`: `chunk_set_id`-based grouping; group-count check (≠N → exit 4); stub-list consistency check (mismatch across cards → exit 4); per-cosigner depth check at watch-only entry.
- All sibling-API consumption correct: `mk_codec::encode_with_chunk_set_id`, `md_codec::compute_wallet_policy_id`, `md_codec::PathDeclPaths::Divergent`.
- Phase D/E scope discipline preserved (no fixture matrix, no Cargo.toml metadata bump, no CHANGELOG).

## Smoke checks (post-fixup)

- `cargo test -p mnemonic-toolkit`: 95 passed (76 unit + 19 integration).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean.
- `cargo fmt --check -p mnemonic-toolkit`: clean.
- v0.1 wire-bit-identical regression: 16/16 PASS.
- Multisig+passphrase: engraving card now correctly emits `passphrase: USED — not engraved on any card; record separately and never lose it.` (the C-1 fix verified end-to-end).
