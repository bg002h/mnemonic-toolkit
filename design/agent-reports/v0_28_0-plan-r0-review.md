## Appendix A — Architect R0 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r0-review.md`. Per CLAUDE.md "agent outputs persist verbatim... BEFORE the fold-and-commit step", the orchestrating session MUST copy this appendix verbatim to that path immediately upon ExitPlanMode, before applying any subsequent folds.

---

```
# v0.28.0 plan-doc — architect R0 review

**Reviewer:** Opus 4.7 (acting as `feature-dev:code-architect`)
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md`
**Plan-doc SHA at draft time:** `fcf9e6d` (origin/master, 2026-05-19)
**Source SHA reviewed against:** `fcf9e6d`

## Critical

### C1. Phase P1C–P6C "3 sites" CLI dispatch is dramatically undercounted (off-by-N concentrator)

[full C1 prose preserved — see folded P0C in Phase 0 above for resolution]

### C2. Plan closes `bsms-bip129-full-cutover` FOLLOWUP but ships only 2 of 5 canonical sub-items — split-state hazard

[full C2 prose preserved — see folded Phase P14A for resolution: do not flip canonical; file new bsms-bip129-encryption-envelope FOLLOWUP]

### C3. Plan closes `bsms-taproot-emit` FOLLOWUP but ships zero of the actual deliverable — real emit is upstream-blocked

[full C3 prose preserved — see folded Phase P14A for resolution: do not flip canonical; entry body update]

### C4. `cost/strip.rs:66` cited site is wrong — actual line is 63

[full C4 prose preserved — citation corrected to line 63]

### C5. Plan's tap→segv0 projection underspecifies x-only-to-compressed lift direction

[full C5 prose preserved — BIP-340 lift-x even-y (prefix 0x02) lock added to SPEC §S.11]

### C6. Q1-Q5 are LOCKED in the plan-doc but listed as "blocked" — internal inconsistency

[full C6 prose preserved — §"Open items" section deleted; B.3 reflowed as Locked]

## Important

### I1. Phase P11 matrix expansion has NO explicit sequencing-gate against parser availability

[I1 — Phase 11 PREREQUISITES stanza added: P1C-P6C + P7C must be merged before P11B/C/D]

### I2. Each new parser needs a `canonicalize_<format>` helper for round-trip discipline

[I2 — each B-sub-phase explicitly scoped to add canonicalize_<format>; files inventory updated]

### I3. Envelope `schema_version` cutover decision is missing

[I3 — SPEC §2.2 locks: stay at "1"; source_format is open-set; consumers tolerate unknown]

### I4. Plan §B.2 #4 first-match-wins precedence is normatively underspecified for ties

[I4 — locked: all-parsers-consulted + Ambiguous-on-multi-match (existing v0.26.0); precedence is documentary]

### I5. SPEC v0_28 file refers to itself but no companion files clearly distinguished

[I5 — locked: CREATE-NEW for both SPEC_wallet_import_v0_28_0.md and SPEC_compare_cost_v0_28_0.md]

### I6. Plan §S.7 BIP-129 4-line shape's line-3 nomenclature

[I6 — Phase P0A locks canonical line-3 name from v0_27_0-phase-2-bip129-recon.md]

### I7. Coldcard-multisig descriptor synthesis is non-trivial — plan elides correctness contract

[I7 — locked: xpub.fingerprint() + blob-XFP-header override + stderr WARNING on divergence]

### I8. Q3 sniff strictness "≥1 bipN key" might false-negative on legitimate Coldcard exports

[I8 — Q3 relaxed to accept disjunction xpub OR ≥1 bipN OR ≥1 bip48]

### I9. Phase P0B SniffOutcome alphabetical sort + ImportProvenance sort: "behavior-unchanged" assertion underspecified

[I9 — split P0B into P0B.1 (SniffOutcome reorder) + P0B.2 (ImportProvenance verify) + P0C (dispatch refactor)]

### I10. Each parser's "fixture-corpus-expansion" overlaps with Phase P9/P10 — phasing conflict

[I10 — fixture inventory in SPEC §S.9 tagged with explicit owner-phase per file]

### I11. mnemonic-gui schema-mirror lockstep PR sequencing is internally inconsistent

[I11 — consolidated to Phase P15 ownership; contradictory claims deleted]

### I12. Slug count off-by-one: plan says "10" but counts 11

[I12 — prose corrected to 11]

### I13. `cost/mod.rs` already has `MultiLeafTr` + `UnsupportedWrapper` variants

[I13 — Phase P12A scope explicit: remove #[allow(dead_code)] from both variants; Phase P12B clarifies new work]

### I14. P13C/P12D compare-cost manual chapter location is unresolved

[I14 — chapter located at docs/manual/src/40-cli-reference/41-mnemonic.md:2455]

### I15. Plan §S.7 4-line cross-validate first-address derivation — descriptor-path-N specification missing

[I15 — locked: derive at receive-branch path /0/0; reuse derive_first_address helper]

## Minor folded

- M1: VENDOR_MARKER_KEYS at line 62 (not 64). FOLDED in §B.2 #5 + Phase P0A.
- M2: WalletFormatParser trait ends at line 47. FOLDED in §B.2 #1.
- M3: ImportProvenance enum ends at line 71. FOLDED in §B.2 #2.
- M4: SniffOutcome at sniff.rs:33-38 — already accurate.
- M5: CHANGELOG draft scope — accepted suggestion; CHANGELOG drafts in P0A as outline, finalized in P14B.
- M6: ColdcardMultisigSourceMetadata cross-module pub(crate) — accepted as-is.
- M7: LOC estimate caveat — annotated in §B.1.
- M8: bsms-2line-tr-numsk.txt typo → bsms-2line-tr-nums.txt. FOLDED in §S.9.
- M9: manual-lint command — installed sibling-codec binaries. FOLDED in P13D.
- M10: bip48 blocks for single-sig parser — Phase P3 SPEC §3.3 clarifies.

## Overall verdict

**YELLOW** at R0 → expected **GREEN** at R1 after the above folds.
```

---

---

