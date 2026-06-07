# Phase 2 (GREEN) Review — self-check-ms1-iteration

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: af690ca76b39a4b67`). Static review; operator ran the gates GREEN.
> The I1 (missing G-A guard) was folded after this review — see operator note.

---

## VERDICT: 0 Critical / 1 Important / 1 Minor

## Verified Clean
1. **Check correctness** (`self_check_bundle` ms1 block): length safety-belt (guards indexing, no panic); per-slot parity (`ms.is_empty() != expected.is_none()` → catches @0-only regression + watch-only false-populate); watch-only skip; `ms_codec::decode` (self-describing, no wire-language); entropy round-trip `payload.as_bytes() != expected_bytes`. Verified `payload.as_bytes()` (ms-codec 0.4.0 `payload.rs:102-107`) returns the raw entropy for BOTH `Entr(data)` and `Mnem{entropy,..}` → Mnem round-trip genuine. Error `card` labels sensible.
2. **Oracle threading at all 4 sites** (414 `resolved`; 1619/1666/1922 `resolved_slots`): each builds `Vec<Option<&[u8]>>` via `.entropy.as_deref().map(|v| v.as_slice())`. `Option<Zeroizing<Vec<u8>>>::as_deref()` → `Option<&Vec<u8>>` → `Option<&[u8]>`; borrow outlives the call. WIF → None → no false reject; import-json seeded → entropy set from envelope (line 1782) → applied. ✓
3. **No false-positive across shapes** (full single-sig, multisig, watch-only, descriptor-mode [site 2 always watch-only], import-json, wif, ms1-mnem). Full suite 0 failed incl. `bundle_import_json_self_check_round_trip_passes`, `ms1_mnem_self_check_round_trips`, `bundle_wif_slot_self_check_passes`.
4. **RED→GREEN integrity**: synthesize-then-mutate (md1 real, R0 M1); RED `self_check_detects_at0_only_ms1_regression` failed pre-fix / passes post-fix; `self_check_detects_wrong_entropy_ms1` discriminating. Non-vacuous.
5. **No surface change** (no flag/subcommand/value → no schema_mirror/manual mirror); `self_check_bundle` BIN-crate pub. `minimal_bundle_args()` names all 17 BundleArgs fields (exhaustive).

## Important (I1) — G-A discriminating guard absent
R0 round-2 GREEN was conditioned on G-A (seeded import-json + `--self-check`) being added. G-B (wif) is present (`cli_self_check.rs:45`); G-A was not. G-A is the only shape where a non-empty `ms1[i]` arrives via the import-json path with NO `--slot` arg — the only test that catches the C1a false-reject and guards against a future site-4 oracle regression (the existing `bundle_import_json_self_check_round_trip_passes` uses an all-empty-ms1 envelope, which a regressed `args.slot` oracle still passes). Fix is test-only.

## Minor (M1) — descriptor-mode site lacks a dedicated self-check integration test
`bundle_run_concrete_descriptor` is always watch-only; structurally identical to the existing all-empty import-json case. Not SPEC-mandated. Non-blocking.

## Bottom line
Shipping code is sound. Phase 2 NOT cleared for Phase 3 until G-A is added (test-only, same branch).

---

## Operator note (I1 folded) — 2026-06-06
G-A added: `cli_bundle_import_json.rs::bundle_import_json_seeded_ms1_self_check_passes` — builds a seeded envelope via `import-wallet --blob - --format bsms --ms1 MS1_TEST_1 --json` (ms1[0]!=""), writes it to a tempdir, then `bundle --import-json <seeded> --self-check` → asserts exit 0. Mirrors Cell 11's seeded-envelope construction. Discriminating: the populated ms1[0] arrives with NO `--slot` arg, so the pre-fix `args.slot` oracle would have expected `ms1[0]==""` → false-reject; the corrected `resolved_slots[0].entropy`-from-envelope oracle passes. Run GREEN. M1 (descriptor-mode dedicated test) accepted as non-blocking.

Gates re-run after G-A: full suite 0 failed; clippy --all-targets 0. **Phase 2 GREEN (0C/0I) — cleared for Phase 3 (release, PATCH v0.47.4).**
