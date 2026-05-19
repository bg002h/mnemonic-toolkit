## Appendix B — Architect R1 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r1-review.md`. Per CLAUDE.md "agent outputs persist verbatim... BEFORE the fold-and-commit step", the orchestrating session MUST copy this appendix verbatim to that path immediately upon ExitPlanMode, before applying any subsequent folds.

---

```
# v0.28.0 plan-doc — architect R1 review

**Reviewer:** Opus 4.7 (via feature-dev:code-architect)
**Plan-doc state:** post-R0-fold
**Source SHA reviewed against:** fcf9e6d

## R0 fold verification

[All 21 R0 findings folded correctly except I9 — see R1-C1 below.]

## R1 Critical (new findings)

### R1-C1. P0B.2 ImportProvenance "verify-only no-op" claim is wrong
[Folded — §B.2 #2 + Phase P0B.2 row now scope a real reorder Bsms→BitcoinCore]

### R1-C2. SniffOutcome alphabetical-sort P0B.1 affects `sniff_format` dispatch arms
[Folded — Phase P0B.1 row clarifies truth-table test mapping-not-arm-order assertion]

## R1 Important (new findings)

### R1-I1. Phase 11 PREREQUISITES stanza lacks gate-enforcement machinery
[Folded — Phase P11A scope adds executable start-gate self-check]

### R1-I2. P0C 8-site pre-stub `unimplemented!()` semantics
[Folded — Phase P0C scope clarifies arm placement BEFORE Some(other) fallback at :239-243]

### R1-I3. Coldcard-multisig xfp WARNING semantics — 2-case crispness underspecified
[Folded — §S.4 now carries 5-row truth table + WARNING template + xfp_header_disagreed metadata field]

### R1-I4. SPEC §S.11 BIP-340 lift-x unit test specification is brittle
[Folded — Phase P12A scope adds two-assertion pair: prefix-discipline (a) + parity-invariance smoke (b)]

### R1-I5. Phase P14A arithmetic + CHANGELOG phrasing audit
[Folded — Phase P14A item (4) reversed: KEEP dup stub OPEN; CHANGELOG per Keep-a-Changelog convention]

### R1-I6. `derive_first_address` reuse claim — module location wrong
[Folded — §S.7 corrected to `derive_address.rs:26`; consumers at `wallet_export/bsms.rs:36, 104` and `wallet_import/bsms.rs:225`]

### R1-I7. `build_tr_descriptor` location citation is wrong
[Folded — definition site cited as `cost/translate.rs:101`; call site at `cost/mod.rs:148`]

## R1 Minor folded

- R1-M1: Phase P15 reviewer-loop-vs-sequential steps distinguished.
- R1-M2: P9B tr-nums fixture ownership + integration cell explicit.
- R1-M3: P11A LOC est clarified — inline helper tests, not consumer tests.
- R1-M4: --format dropdown ordering verified against mnemonic-gui at P15.
- R1-M5: roundtrip.rs touched in P0C (skeletons) + P1B-P6B (impls); files inventory updated.
- R1-M6: §S.4 "investigation directive" reworded.
- R1-M7: mnemonic-gui next-version informative; verify against pinned-upstream.toml.

## Overall R1 verdict

**YELLOW** at R0 → folded → R1 dispatched → still YELLOW (2 new Critical + 7 Important all folded) → R2 dispatch pending.

Per R1 architect recommendation: "Do NOT call ExitPlanMode yet. Apply the 9 R2 folds above and re-dispatch R2 architect review."
```

---

---

