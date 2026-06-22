# R0 REVIEW (Round 2 — convergence) — `PLAN_w1_export_refusals_and_reconcile.md` (toolkit v0.70.1)

**Reviewer:** opus architect (independent R0, round 2). **Date:** 2026-06-22. **Source SHA:** HEAD = `1cea85ea`.
**Persisted verbatim per project discipline.**

## VERDICT: 0 Critical / 0 Important / 0 Minor

### Round-1 finding dispositions
- **I-1 (#4 DEFER justification) — RESOLVED.** §3 drops the false "scope creep / whole-descriptor decode" claim ("this is NOT sibling scope creep"), justifies DEFER on "no live defect" + bech32-wrong-for-custom-BCH(93,80,8), cites `md_codec::bch::bch_verify_regular`. Confirmed live: `bch_verify_regular` `pub fn` at `descriptor-mnemonic/crates/md-codec/src/bch.rs:89`; `polymod_run:53`, `hrp_expand:62`, `MD_REGULAR_CONST:17`, `GEN_REGULAR:7` all public. Multi-chunk vs whole-descriptor nuance recorded.
- **I-2 (#2 Green premise + test) — RESOLVED (all four).** §1 no longer asserts Green-imports-tr(KEY) as fact ("unverified… conservative disposition: discriminate"); discriminate framed as conservative; test renamed `cell_7_green_bip86_keypath_emission_unchanged` + reframed as behavior-pinning ("NOT a correctness assertion"); new FOLLOWUP `green-taproot-keypath-file-import-unverified` specified with escalation path + companion link.
- **M-1 (live citations) — RESOLVED.** H10 guard export_wallet.rs:124 ✓; variant error.rs:177 ✓; emitter `ok_or_else` electrum.rs:52/jade.rs:36/coldcard.rs:111 (plan :51/:34/:111 — coldcard exact, electrum/jade ±1, all in the right `emit()` bodies); test :1105 + fixture :951 exact.
- **M-2 (#3 as second arm) — RESOLVED.** §2 heading "a SECOND ARM of the existing H10 guard, not a new guard"; notes variant+guard shipped v0.62.0.
- **M-3 (green.rs import) — RESOLVED.** Build plan step 2 is now definite. Confirmed: green.rs:19 lacks `WalletScriptType` (electrum.rs:15 has it) — import genuinely required.
- **M-4 (live-RED sibling-pin-check) — RESOLVED.** §4 surfaces as separate item, NOT folded, with ship-gate-discipline language. Confirmed: manual.yml:86 `md-cli-v0.6.2` vs install.sh:35 canonical `v0.7.1` — gate live-RED.
- **M-5 (version-site path) — RESOLVED.** Build plan step 4 uses `scripts/install.sh:32` with the "not root" parenthetical. Confirmed install.sh:32 = `mnemonic-toolkit-v0.70.0`.

### New drift introduced by folds
None. Pure reframe/justification + one new FOLLOWUP; no change to the R0-verified #2 structural fix or #3 second-arm design. No contradiction with any round-1 Verified-correct item. §6 Q-rulings match item bodies; §7 R0-status accurate.

### New findings (fresh adversarial read)
None rising to Minor. One sub-Minor (non-blocking): electrum/jade `ok_or_else` citations off by one line (:51/:34 vs live :52/:36) — "~"-prefixed, inside the right `emit()` bodies, tolerated by the citation-decay convention. Not worth a re-fold.

**GATE: GREEN (0C/0I) — cleared for implementation.**
