# Phase 2 (Task 2) — code-quality / per-phase architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Subagent-driven two-stage review: spec-compliance =
> ✅ compliant (guard correct & precise, both RED cells, no scope creep, 15 passed); code-quality (this
> doc) = **APPROVE with one Minor gap, 0C/0I/1m**. Dispatched via Agent (feature-dev:code-reviewer,
> inherited Opus 4.8). Diff BASE `2bebb99` → HEAD `a8fe9bd`. The 1 Minor (missing SortedMultiA @-in-both
> refusal cell) folded before Task 3. NOTE: the impl caught a real plan/spec defect (degenerate 2-of-2 is
> a coincidental k>n catch, genuine exit-0 RED needs n≥3); spec/plan corrected in `db5a700`.

---

## Review: Task 2 — `@-in-both` structural guard for non-NUMS taproot restore

**Reviewing:** `git diff 2bebb99 a8fe9bd`. Files: `crates/mnemonic-toolkit/src/cmd/restore.rs`, `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`.

## Strengths

**Guard completeness — confirmed.** Placed in both `Tag::MultiA` and `Tag::SortedMultiA` arms of `classify_taproot_restore`, before `Ok(TaprootRestore::Template(...))`. These are the only arms that produce a `Template` variant — the only consumer of the dangerous `Cosigner(idx)` shortcut (`pipeline.rs:134-154`). The general arm produces `GeneralFaithful` and never calls the shortcut. Guard at exactly the right chokepoints.

**`Body::MultiKeys` is the ONLY valid body for `Tag::MultiA`/`Tag::SortedMultiA`.** md-codec 0.35.3 `tree.rs:244-254`: the decoder hardcodes `Body::MultiKeys` for all four multi-family tags. No decode path produces a `MultiA`-tagged node with another body. The inner `if let Body::MultiKeys { indices, .. }` is exhaustive for any node reaching the Template arms.

**No cast risk.** `Cosigner(u8)`, `key_index: u8`, `indices: Vec<u8>` — `idx == *i` is u8==u8.

**NUMS path correctly excluded.** Outer `if let Cosigner(i)` → `Nums` trunks return `Ok(())`.

**GeneralFaithful arm correctly excluded.** The `_` arm produces `GeneralFaithful` → `faithful_multisig_descriptor` → `md_codec::to_miniscript` (`to_miniscript.rs:161-164` `is_nums:false → lookup_key`), which reads the ACTUAL tree — cannot emit the "all-others" wrong-leaf. Guard not needed there; absence correct.

**Display-fidelity bypass correctly characterised.** `pipeline.rs:28-31` + 143-154 confirm the Template path `from_str(rendered).to_string()` — output is its own re-print; a `multi_a(2,@1,@2)` parses & re-prints identically. Structural guard is the only net.

**n=3 is genuinely the critical RED.** For n=2, dropping the trunk leaves 2-of-1 → miniscript k>n (coincidental). For n=3, `multi_a(2,@1,@2)` is a valid 2-of-2 → exit 0, silently-different wallet. Test structure (primary n=3 + secondary 2-of-2) correct; doc comments accurate.

**`build_at_in_both_descriptor` sound.** `is_wallet_policy()` gate cleared by non-empty `tlv.pubkeys`. Guard keys on INDICES not bytes, so identical filler is irrelevant. `key_index:0` in Tr body + leaf indices including 0 → @-in-both condition.

**Doc comment quality excellent.** Explains WHY structural (not Display), cites pipeline lines, the n=2 degenerate case, the FOLLOWUP slug. A model for safety-critical documentation.

## Issues

### Critical — None.
### Important — None.

### Minor (confidence 83)
**`build_at_in_both_descriptor` tests `Tag::MultiA` only — the `SortedMultiA` arm is not covered by a refusal test.** `cli_restore_taproot.rs:354-366, 431-468`. The builder hardcodes `tag: Tag::MultiA`. The guard is identical in both arms, but no refusal test exercises the `SortedMultiA` variant — a future refactor removing `refuse_at_in_both` from only the SortedMultiA arm would not be caught. Fix: parametrize the builder with a `tag: md_codec::Tag` and add `at_in_both_sortedmulti_a_refuses_structurally` (n=3, `Tag::SortedMultiA`) asserting exit 2 + slug + "also a leaf key". Minor (guard code is identical + placement source-verified + N3 exercises the SortedMultiA non-refusal path), but worth closing before ship for the funds-safety crux.

## Load-bearing question answers
- **Catches EVERY reachable @-in-both?** Yes — any `MultiA`/`SortedMultiA` inner node has `Body::MultiKeys` (decoder-guaranteed); `Cosigner(i)` with `i ∈ indices` always fires.
- **Any @-in-both slips past?** No — the only Template entry is post-`refuse_at_in_both`-Ok; Ok only for Nums or distinct-trunk.
- **Precise (not over-broad)?** Yes — N1/N2/N3 distinct-trunk (K2 trunk ∉ {K0,K1} leaf) still reconstruct.
- **General arm same trap?** No — `GeneralFaithful` → `lookup_key` reads the actual tree; no shortcut.

## Assessment: **APPROVE with one Minor gap to close before ship.**
The guard is correct, complete, precisely placed, necessary. The funds-safety property is fully closed for the single-leaf `multi_a`/`sortedmulti_a` Template path: no reachable @-in-both md1 silently reconstructs a wrong multisig. NUMS exempt; general arm correctly unguarded; doc/test documentation exemplary. Close the SortedMultiA refusal-test gap before tagging.
