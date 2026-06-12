# PLAN — pin the taproot restore-refusal contracts (GAP 1-T1)

**Date:** 2026-06-12 · **Repo:** mnemonic-toolkit · **SemVer:** NO-BUMP (test-only)
**Source SHA:** mnemonic-toolkit `origin/master` = `2f03eb0`. **Recon:** `cycle-prep-recon-taproot-coverage-gaps.md` (T1). **FOLLOWUPs (already filed in the hygiene pass):** `restore-general-and-multi-leaf-taproot-roundtrip`, `upstream-miniscript-taptree-depth2-display-asymmetry` (toolkit companion).

## 1. Problem (probe-grounded)

`bundle --descriptor` emits a faithful md1 card for taproot policies BEYOND single-leaf NUMS-`multi_a`/`sortedmulti_a`, but `restore --md1` reconstructs ONLY those two — every other `Tag::Tr` md1 hits a refusal arm in `src/cmd/restore.rs::taproot_template_and_internal_key`, and NONE of those arms is currently tested (recon-confirmed: zero `tests/` hits). This is the engrave-but-can't-mechanically-restore class; pinning the refusal CONTRACT prevents it from silently changing (e.g. a future edit turning a clean refusal into a wrong reconstruction or a panic).

**Probe-verified reachable-via-`bundle`→`restore` — THREE arms (R0-I1 corrected: all 3 are bundle-reachable):**
- **General taproot leaf** — `tr(NUMS,and_v(v:pk(K0),after(12000000)))` → bundle emits → `restore --md1` exit **2**, stderr substring `not a recognized multisig` (restore.rs:~710).
- **Multi-leaf taptree** — `tr(NUMS,{pk(K0),pk(K1)})` → bundle emits → `restore --md1` exit **2**, SAME message (a `TapTree`-tagged inner → same arm).
- **`is_nums:false` (cosigner-internal-key) tr** (restore.rs:~689) — `tr(K2,multi_a(2,K0,K1))` with a **DISTINCT** internal key K2 (∉ leaf set) → bundle exit **0** emits a card → `restore --md1` exit **2**, stderr substring `non-NUMS (cosigner) internal key`. (R0-I1: my earlier "bundle emits nothing" probe was tripped by a duplicate-key BIP-388 gate — `tr(K0,multi_a(2,K0,K1))` refuses at bundle with a distinct-key violation; with a distinct K2 it bundles + restore refuses via the :689 arm.)

**Genuinely NOT bundle-reachable (deferred):**
- **keypath-only `tr(NUMS)` with `tree:None`** — refuses at bundle's origin-annotation gate ("descriptor has neither @N placeholders nor [fp/path]-annotated keys"); the only arm needing a direct wire fixture. Deferred to the `restore-general-and-multi-leaf-taproot-roundtrip` FOLLOWUP (T3). (Note: a keypath-only `tr(K0)` xpub card DOES bundle and hits the :689 arm with a slightly misleading "multisig"-worded message — R0-M3, noted as a FOLLOWUP touch-up, NOT reworded here since this cycle PINS that message as a contract.)

## 2. The fix (test-only)

New integration test file `tests/cli_restore_taproot_refusal.rs` (bin-spawning, mirrors `cli_restore_multisig.rs` style — `Command::cargo_bin("mnemonic")`, `--md1 <chunk>` PER chunk via a local `restore_args` helper). A small `bundle_md1(desc) -> (Vec<String> chunks, String emitted_descriptor)` helper runs `bundle --descriptor … --network mainnet --json` and pulls `.md1[]` + `.descriptor`.

1. `general_tr_leaf_bundles_faithfully_but_restore_refuses`: `bundle` `tr(NUMS,and_v(v:pk(K0),after(12000000)))` → assert `.md1` non-empty AND `.descriptor` == the input string EXACTLY (R0-M1: the literal `NUMS` token is PRESERVED on the wire — NO substitution — so assert strict equality; this documents the card is a FAITHFUL BACKUP) → `restore` → exit 2 + stderr contains `not a recognized multisig`. (Pins restore.rs:710 + the engrave-but-can't-restore contract.)
2. `multi_leaf_taptree_bundles_faithfully_but_restore_refuses`: same for `tr(NUMS,{pk(K0),pk(K1)})` — `.descriptor` exact-equality (R0-I2: probe-confirmed exact round-trip) + restore exit 2 + same message. (Pins the GAP-1 sub-4 multi-leaf contract incl. the wire-faithfulness leg.)
3. `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` (R0-I1): `tr(K2,multi_a(2,K0,K1))` (distinct internal key K2) → bundle emits → restore exit 2 + stderr contains `non-NUMS (cosigner) internal key`. (Pins restore.rs:689 — the third bundle-reachable arm.)

This 3-arm + faithful-backup set fully delivers the `restore-general-and-multi-leaf-taproot-roundtrip` FOLLOWUP's T1 scope-split (a) (3 refusal arms + multi-leaf wire round-trip). The verify/address legs are out of scope (verify-bundle needs cosigner mk1 cards; restore refuses before any address derivation — R0 agreed).

Keys (R0-M2 — use REAL existing bracketed-xpub literals, not the nonexistent `XPUB4_*`): the 3-cosigner trio at `cli_bundle_import_json.rs:312-314` — fps `73c5da0a`/`b8688df1`/`28645006`, all `[fp/87'/0'/0']xpub…/<0;1>/*`. Lift the three xpub+bracket literals as local `const`s (K0/K1/K2) in the new test file. Concrete watch-only `bundle --descriptor` (no seed).

## 3. Verification
`cargo test -p mnemonic-toolkit --test cli_restore_taproot_refusal` (bin-spawning, no env needed for the core cells; the optional `md inspect` cell gates on `MD_BIN`). Probe already confirmed the exact exit codes + messages. GREEN gate: the new file green + `cargo clippy --all-targets -- -D warnings` clean + fmt (only mlock.rs exempt-diff tolerated). No `src/` change → NO-BUMP.

## 4. Lockstep / SemVer
- NO-BUMP (one new test file). No clap surface change → no manual/GUI/schema_mirror. No md-codec change. The FOLLOWUPs are already filed (hygiene pass) — this cycle adds the tests + flips nothing (the refusal is the documented current behavior; faithful reconstruction stays the FOLLOWUP's T3).
- These refusal MESSAGES become a pinned contract — a future restore-walker change touching them must update these cells in lockstep (that's the point).

## 5. R0 questions — ANSWERED (R0 round 1, folded)
1. **Scope** → R0-I1 corrected: THREE arms are bundle-reachable (general leaf, multi-leaf, distinct-IK non-NUMS) — all three pinned. Only `tr(NUMS)` keypath-only (`tree:None`) needs a direct fixture → deferred to the FOLLOWUP T3. This fully delivers the FOLLOWUP's T1 scope (R0-I2).
2. **Brittleness** → pin stable SUBSTRINGS (`not a recognized multisig`, `non-NUMS (cosigner) internal key`), not full strings. (R0 agreed.)
3. **verify/address legs** → out of scope (verify-bundle needs cosigner mk1; restore refuses before address derivation). bundle-emits + faithful-`.descriptor` + restore-refuses is the complete contract. (R0 agreed.)

## 6. Also (folded from R0)
- **M3 / I2 FOLLOWUP touch-up:** add a one-line note to `restore-general-and-multi-leaf-taproot-roundtrip` that `tr(NUMS)` keypath-only (`tree:None`) is the sole fixture-requiring arm (T3), and that keypath-only `tr(<xpub>)` cards hit the :689 arm with a "multisig"-worded message that misleads for single-sig (diagnostics-accuracy nit, fix in T3 — not reworded here since this cycle pins the message).
- Exit code confirmed `ModeViolation => 2` (error.rs:541). Substring pinning confirmed. `restore_args` (per-chunk `--md1`) + `Command::cargo_bin("mnemonic")` are the house pattern.
