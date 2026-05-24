# R2 ARCHITECT REVIEW — `BRAINSTORM_v0_37_0_from_import_json_template_reemit.md`

**Round:** R2 (post-R1-fold convergence check)
**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus), continuation of R0/R1
**Spec SHA basis:** `36e6bfa`
**Verdict:** RED (0 Critical / 1 Important)

Files re-read: `cmd/export_wallet.rs` (both `run` `:236` and `run_from_import_json` `:540` in full), `wallet_export/{mod,sparrow,coldcard,electrum,jade,bip388,green}.rs`, `template.rs`, `tests/cli_export_wallet_from_import_json.rs` (fixture map, ALL_SOURCES, TEMPLATE_ONLY_DESTS, REFUSAL_STDERR_PATTERNS, p11a/p11c, Cell 3), `bsms-2line-sortedmulti-2of3.txt`, chapter-45, `40-cli-reference/41-mnemonic.md`.

## Fold-verification of the four R1 items
- **I-R1-1 (bsms / ALL_SOURCES partition) — RESOLVED, exact.** bsms fixture body is `wsh(sortedmulti(2,…))#he0ej3xr` → `WshSortedMulti`. §5.1 now states all 8 sources → `bip84` (3 singlesig) or `wsh-sortedmulti` (5 multisig incl. bsms), no P2shMulti source; `p11a` keeps only singlesig→coldcard-multisig.
- **I-R1-2 (REFUSAL_STDERR_PATTERNS / Cell 3) — RESOLVED, exact.** Coldcard-multisig literal confirmed already at `tests/…:817`; P2shMulti literal routed to Cell 3 inline assertion `:114-117`.
- **M-R1-a — RESOLVED IN §3 ONLY; re-introduced in §6 Phase 2 (see I-R2-1).**
- **M-R1-c (account-0) — RESOLVED.** §5.3 states `--account` rejected (`:554`), fixtures account-0.

## CRITICAL
None.

## IMPORTANT

### I-R2-1 — M-R1-a fix contradicted by the §6 Phase 2 task list, which still names `:353` as a prose-update target
Spec `:146` (Phase 2) read "…+ `45:347`/`:353` prose + …", contradicting the §3 (`:110`) M-R1-a fold that establishes `45:353` (in the `45:352-357` taproot round-trip note) must be left unchanged. A Phase-2 implementer working the §6 checklist would edit the protected taproot note. Fold-introduced drift (the M-R1-a fix landed in §3 but was not mirrored to the parallel Phase-2 list). **Remedy:** `:146` → "`45:347` prose (leave `45:352-357` unchanged)". Confidence 85.

## MINOR
- **M-R2-a** — §2.6 mislabels `:493-511` as "collect_missing path"; both `:493-511` and `:713-735` are emit-dispatch blocks (`_ => Err` at `:510`/`:730`); the collect_missing dispatches at `:469`/`:687` carry no coldcard-multisig guard. Descriptive only.
- **M-R2-b** — §5.1 anchors `:841`/`:611`/`:892` point at the `fn` line rather than the `#[test]` attribute one line above; valid locators, non-blocking.

## Convergence consistency check (§0 / §5.1 / §5.2 / §5.3)
Mutually consistent except I-R2-1. Spot-verified the succeed/refuse split is real: singlesig sources SUCCEED for coldcard/electrum/sparrow, REFUSE for coldcard-multisig (`:730`) and jade (`jade.rs:56-62`); multisig sources succeed across all five. §2.3 partition = exactly the `inputs.template.ok_or_else`-refusers; green reads `script_type` only. `CliTemplate` confirmed 10 variants, no bare-`sh(multi)` → §2.2 `Err` arm required.

## VERDICT
**VERDICT: RED (0C/1I)** — only the un-propagated M-R1-a fix in §6 Phase 2 (I-R2-1). Mechanical one-line fix. Fold and re-dispatch for R3.
