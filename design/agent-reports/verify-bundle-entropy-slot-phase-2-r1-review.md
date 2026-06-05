# Phase 2 Architect Review (Round 1) — implemented arm + tests (v0.43.1)

**Reviewer:** opus `feature-dev:code-reviewer` (per-phase gate, post-implementation)
**Date:** 2026-06-04
**Branch:** `verify-bundle-entropy-slot` (impl commit `9f91055`)
**Verdict:** 0 Critical / 0 Important / 0 Minor — **GATE: GREEN**

> Persisted verbatim per CLAUDE.md. The reviewer's caveat (it could not run cargo in its sandbox) is satisfied by the operator's empirical run: full workspace `cargo test --no-fail-fast` = 0 failures (aggregate `grep -cE '^test .* FAILED'` = 0), new file 5/5 GREEN.

---

## Review summary

Reviewed the v0.43.1 PATCH on branch `verify-bundle-entropy-slot`: the new `else if subkeys.contains(&SlotSubkey::Entropy)` arm in `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:855-894` and the 5 integration tests in `crates/mnemonic-toolkit/tests/cli_verify_bundle_entropy_slot.rs`. Verified against the bundle Entropy arm (`bundle.rs:1438-1470`), the adjacent verify Ms1 arm (`verify_bundle.rs:895-924`), the shared helper (`derive_slot.rs:65-112`), `into_parts()` (`derive.rs:47-57`), `is_legal_set` (`slot_input.rs:342-367`), the verdict plumbing (`verify_bundle.rs:1001-1034`, `emit_verify_checks` 1282-1454), and `lint_zeroize_discipline.rs`.

Note: this environment has no shell/cargo tool, so the suite/lints were not run; the GREEN claim is taken as reported and corroborated by source analysis. The one regression class an empirical run could not catch anyway (a silently reworded catch-all or perturbed sibling arm, since no GREEN test exercises the catch-all post-fix) I verified directly by diff-proxy: the sole `v0.43.1` marker in `src` is at `verify_bundle.rs:856`, and the catch-all message at `:928` is byte-identical to the SPEC-cited original.

**1. Arm correctness** — Confirmed. Destructure `(_acc_entropy, master_fp, xpub, _xpriv, _path)` matches `into_parts()`'s `(Vec<u8>, Fingerprint, Xpub, Xpriv, DerivationPath)` position-for-position, identical to the Ms1 arm. No silent mis-bind is possible: all five element types are distinct, so a swapped position would fail to compile. `emit_lang = None` is correct (raw entropy carries no BIP-39 wire language). The 5-tuple `(xpub, master_fp, anno_path.clone(), Some((*entropy_bytes).clone()), None)` is well-formed. The borrow-then-clone is valid: `derive_bip32_from_entropy_at_path` takes `&entropy_bytes`, returns, then `(*entropy_bytes).clone()` reads the still-live `Zeroizing<Vec<u8>>` after the borrow ends.

**2. Symmetry** — Confirmed byte-identical to the bundle Entropy arm. The bundle arm derives inline; the verify arm routes through `derive_bip32_from_entropy_at_path`, whose spine is identical step-for-step: `Mnemonic::from_entropy_in(language, entropy)` → `derive_master_seed` → `Xpriv::new_master(network.network_kind(), seed)` → `master.fingerprint(&secp)` → `master.derive_priv(&secp, anno_path)` → `Xpub::from_priv`. Same `language.into()`, same `args.network`, same `anno_path`, same `emit_lang=None`. No divergence.

**3. Test soundness** — All 5 exercise the new arm and are non-vacuous. The keystone is `passphrase_mismatch_detected:208`: same entropy (ms1 matches, entropy-only) but a passphrase delta perturbs only the seed-derived xpub → `mk1_xpub_match` fails → `result: mismatch`. This proves mk1/md1 are live-compared and derivation-sensitive, which transitively de-vacuifies the three `result: ok` round-trips (they bind on `mk1_xpub_match` at `verify_bundle.rs:1436`, not on ms1 alone) — none of the five would pass under a wrong-key arm. `nonzero_slot_multi_n:161` genuinely binds `@1.entropy` via the new arm (loop hits `idx=1`, `subkeys={Entropy}`), the position the RED baseline died at. The mismatch test's GREEN keys on stdout `result: mismatch` (only reachable post-fix), correctly distinguishing it from the RED catch-all which also exits 4 but emits no `result:` line.

**4. Zeroization** — Satisfied. Entropy in `Zeroizing` (`:874`), passphrase in `Zeroizing<String>` (`:878`), identical to both sibling arms. The transient bare `Vec` from `(*entropy_bytes).clone()` is re-wrapped/re-pinned at the shared field-write boundary (`:934-935`), exactly as the Ms1 arm does. No new plaintext-secret-on-stack path. `lint_zeroize_discipline` is an evidence-anchor grep over a fixed row list that does not enumerate the verify Entropy arm and whose existing anchors are untouched — stays GREEN.

**5. Regressions / missed surface** — None. The change is the single inserted arm. Catch-all message, arm order, and all sibling arms are unchanged. Precedence is genuinely moot: `is_legal_set` permits `[Entropy]` only as a standalone set (no `[Entropy, *]`), so `subkeys.contains(&Entropy)` is never simultaneously true with the Xpub/Ms1 checks. The descriptor-mode verdict plumbing (`result: ok`/`result: mismatch`, exit 0/4) is the shared path at `:1011-1034` and is reached correctly; the Entropy arm feeds the same `cosigners` vec it consumes. SPEC §6 "no clap surface change" is corroborated — the arm adds no flag/option/subcommand/value-enum, so `schema_mirror` and the manual mirror are correctly not triggered.

## Critical
None.

## Important
None.

## Minor
None. (The arm is a faithful, clean mirror; no manufactured nits.)

VERDICT: 0 Critical / 0 Important
GATE: GREEN

Caveat for the gate-keeper: I could not execute `cargo test --no-fail-fast` / `lint_zeroize_discipline` / `schema_mirror` in this environment. Source analysis gives high confidence (the arm cannot compile if mis-bound, and the diff is provably isolated to one arm), but per the per-phase discipline, confirm the empirical 5/5 GREEN + zero-regression suite run before release prep + tag.

— **Operator note (gate-keeper):** empirical confirmation done before this persist — full workspace `cargo test --no-fail-fast` reported every binary `0 failed`, aggregate `^test .* FAILED` count = 0, and the new `cli_verify_bundle_entropy_slot` file 5/5 GREEN. Caveat satisfied; proceeding to Phase 3 release prep.
