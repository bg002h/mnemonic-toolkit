# Phase 1 (Task 1) — code-quality / per-phase architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Subagent-driven-development two-stage review:
> spec-compliance = ✅ compliant (all 6 points verified, no scope creep, 13 passed, NUMS goldens
> byte-identical); code-quality (this doc) = **Approve with recommendations, 0C/0I/3m**. Dispatched via
> Agent tool (feature-dev:code-reviewer, inherited session default Opus 4.8). Diff BASE `51966fc` → HEAD
> `9c7edeb`. Minors M1-M3 folded in the follow-up commit before Task 2.

---

## Review of Task 1: `restore-non-nums-tr-internal-key` branch, diff `51966fc → 9c7edeb`

**Reviewing:** changes to `crates/mnemonic-toolkit/src/cmd/restore.rs` and `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs` that lift the blanket non-NUMS taproot restore refusal.

### Strengths

**Correctness of the trunk-key read.** `Body::Tr.key_index` is declared `u8` in md-codec 0.35.3 (`tree.rs:54`), and `TaprootInternalKey::Cosigner` is `Cosigner(u8)` (`wallet_export/mod.rs:95`). The dereference `*key_index` at `restore.rs:710` is a clean copy with no cast — type-safe and correct.

**Match exhaustiveness.** The four arms of the `match &tree.body` block in `classify_taproot_restore` are: `is_nums:true + Some(inner)`, `is_nums:false + Some(inner)`, `tree: None`, and a catch-all for non-Tr bodies. Together they cover the full md-codec `Body::Tr` shape without gap, and the catch-all correctly refuses a non-Tr body arriving at a taproot classifier.

**NUMS path byte-identical.** The `is_nums:true` arm still maps to `TaprootInternalKey::Nums` unchanged. The call site `restore.rs:1215` correctly uses the variant carried from `classify_taproot_restore`, so both NUMS arms produce the same `TaprootInternalKey` that was hard-coded before. Pre-existing NUMS goldens are unaffected.

**GeneralFaithful route-around correctness.** For `is_nums:false`, the descriptor is produced by `faithful_multisig_descriptor` — which re-enters md-codec's `to_miniscript_descriptor`, and that function's `is_nums:false → lookup_key` branch (`to_miniscript.rs:161-164`) reads the actual key from the wire directly. The `TaprootInternalKey` carried in the `GeneralFaithful` variant is then only used for the `--format` dispatch path (`build_multisig_import_payload`), not for descriptor construction itself. This is correct.

**`@-in-both` funds-safety: reachability confirmed contained.** `bundle --descriptor "tr(K, multi_a(k, K, others))"` is rejected at intake by the BIP-388 distinct-key gate (`ToolkitError::Bip388DistinctKeyViolation`, exit 2 — confirmed via `error.rs:632`). The only path to a `@-in-both` md1 is direct hand-construction via md_codec's public API — an adversarial/manual operation, not a legitimately engraved card. The missing Task 2 guard leaves this hand-crafted path producing a silently-wrong reconstruction, but NO legitimately-engraved card reaches it. Correctly the planned RED baseline for Task 2.

**Test quality — N1/N2.** Both go through the full `bundle_md1 → restore_args` round-trip with a wire round-trip assert, non-empty md1 assertion, success assertion, AND golden descriptor + address pins. The trunk key renders as a depth-0 `xpub661My...` key bearing K2's `[28645006/87'/0'/0']` fingerprint/path (not the 50929b74... NUMS hex).

**Deleted refusal test.** `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` correctly removed per plan Step 6, with N2 covering the now-supported distinct-trunk shape as a success.

### Issues

#### Critical — None.
#### Important — None.

#### Minor

**M1: Test file module-level doc is stale (actively false after Task 1).** `cli_restore_taproot.rs:11-12` — "Still refused … non-NUMS (cosigner) internal key (`restore-multisig-taproot-reconstruction`)" is now FALSE. Confidence 85. Fix: delete/update that bullet (Task 4 docs sweep should include the test header). Recommend fixing before Task 2.

**M2: N3 omits the wire round-trip assertion present in N1 and N2.** `cli_restore_taproot.rs:224-237` discards `_emitted`. Confidence 82. Fix: add `assert_eq!(emitted, desc, "non-NUMS sortedmulti_a must round-trip on the wire");` after the `assert!(!md1.is_empty())`.

**M3: The shared golden address for N2 and N3 may surprise a future reader but is technically correct.** `cli_restore_taproot.rs:107-110` — identical because `sortedmulti_a` == `multi_a` script when {K0,K1} already sorted. Confidence 80. Fix: a one-line clarifying comment.

### `@-in-both` gap assessment
With Task 2 deferred, `restore --md1` in this commit will silently reconstruct a WRONG descriptor for any `@-in-both` md1 card. However, NO such card is legitimately producible by `bundle` (BIP-388 distinct-key gate refuses at intake, exit 2). The only producer is direct md_codec hand-construction (Task 2's RED-proof mechanism). Correctly characterized as a RED baseline, not a shipped funds-safety defect.

### Assessment: **Approve with recommendations**
Core implementation correct, type-safe, exhaustive; NUMS path provably unaffected; @-in-both gap deliberate and blocked from legitimate production by bundle's intake gate. Three Minor findings are all documentation/test-quality (M1 most important — fix before Task 2). None are blockers for advancing to Task 2.
