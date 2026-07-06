# Per-phase R0 — Cycle A Phase 2 (funds-proof regressions) — Round 1

**Reviewer:** opus architect. **Diff:** `888389ea..ea0d3019` — ONE new file `tests/cli_cycleA_phase2_funds_proof.rs` (366 ins / 0 del, 5 tests). FULL suite reproduced in worktree. Persisted verbatim per CLAUDE.md.
**Verdict:** GREEN (0C/0I).

## Suites (reproduced): `mnemonic-toolkit` **3583 passed / 0 failed / 16 ignored** (3578 + 5). `wc-codec` **100 passed / 0 failed** (reconciled — see below).

## Non-collateral: `--numstat` = 1 file, 366/0, under `tests/`. Zero src/fixture/existing-test/design change.

## Per-test non-vacuity
- **T1 `verify_bundle_concrete_..._rejects_before_card_comparison` SOUND.** Concrete `wpkh([fp/84'/0'/0']xpub/0/*)` → Concrete fork (`verify_bundle.rs:1352`) → `descriptor_concrete_to_resolved_slots` → `parse_descriptor` → residue reject → `DescriptorParse` (`pipeline.rs:418`) exit 2; `?` short-circuits BEFORE `verify_emit_from_expected` (card compare). Asserts `.failure()`+`code==2`+`multipath`+`<a;b>`. `<a;b>` unique to the residue message → non-vacuous. Locks I-B concrete=exit2.
- **T2 `..._at_n_template_..._rejects_exit_4` SOUND.** `wpkh(@0[fp/84'/0'/0']/0/*)` non-Concrete → `verify_bundle.rs:1375` `map_err(DescriptorReparseFailed)` exit 4; Display "descriptor re-parse failed during verify-bundle: {detail}" (`error.rs:801`), detail contains `multipath`. Asserts `code==4`+`re-parse failed`+`multipath`. Other side of the I-B fork.
- **T3 `collapsed_wrong_oracle_value_independently_confirmed` SOUND, non-circular.** Re-derives via `bitcoin`/`bip39` crates ONLY: `account_xpub/0`==`bc1q8vph849...` (wrong), `account_xpub/0/0`==`bc1qcr8te4...` (true), `assert_ne!`. External-fact verification of the SPEC §1 oracle.
- **T4 (crown jewel) `bundle_descriptor_multipath_restores_to_true_bip84_first_receive` GENUINE non-vacuous funds proof.** `<0;1>/*` card built via `bundle --descriptor`→`concrete_keys_to_placeholders`→`lex_placeholders` (the bug's pipeline); `restore --md1` alone emits to stdout (`restore.rs:699-753`). Asserts 3 exact/direct: (1) `contains(bc1qcr8te4...)` — DERIVED by restore (not echoed), cross-validated true `m/84'/0'/0'/0/0` by T3; (2) `contains("<0;1>/*")` — use-site preserved not collapsed; (3) `!contains(bc1q8vph849...)`. Mnemonic `abandon×11 about`. Direct first-receive-address proof.
- **T5 `..._cannot_encode_the_collapsed_wallet` SOUND.** `/0/*`→reject→`DescriptorParse` exit 2+`multipath`+`<a;b>`. Adds the address-oracle tie-in over a4/a5.

## Ruling — T4 "could it pass without the fix?": YES, CORRECT BY DESIGN. A valid `<0;1>/*` was captured correctly pre- AND post-fix; T4's role (SPEC §8 / plan 2b(i)) is the POSITIVE oracle half (correct wallet⇒correct address) + an over-rejection guard (new floor must NOT reject valid `<0;1>/*`). The fix-dependent NEGATIVE power is carried by T1/T2/T5 (pre-fix = exit-0 false-pass / silent collapse). The pairing is exactly SPEC §8's mandate. Suite-as-a-whole = complete end-to-end funds proof.

## wc-codec reconciled = 100/0 (definitive). Per-binary: lib 0, field 10, pad 5, pipeline 24, raid 13, regroup 8, rs 12, sync 23, wordmap 5, doctests 0 → Σ 100. Phase-1 R0's 100 CORRECT; Phase-2 impl's 40 = partial/filtered invocation under-count. No code delta (Cycle A doesn't touch wc-codec).

## MINOR (no action): M-1 T4 passes ±fix (intentional positive oracle + over-rejection guard; T1/2/5 are the regressions). M-2 message names both `/0/*`+`/**` so bare contains wouldn't discriminate — no cell relies on that (anchor on `<a;b>`/exit/exact-addr). M-3 T5 overlaps a4/a5, additive via address tie-in.

## VERDICT: GREEN (0C/0I). Phase 2 advances.
