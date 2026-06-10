# R0 round-2 architect review — PLAN_minors_m1_m2_m3_m13 (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). Source b822fb9. Verdict: GREEN (0 Critical / 0 Important / 4 minor — folded before Phase 1). Review verbatim below.

---

## Critical

None.

## Important

None.

## Fold-verification

**I-1 (slot-exact xpub binding) — FOLDED-OK.** Plan Part 1 now specifies slot-exact `(i as u8, xpub_to_65(&card_xpub))` membership in `desc.tlv.pubkeys`, Single = `(0, …)`, `pubkeys: None` as a distinct defensive failure, the cross-slot-swap rationale recorded, and the `verify_bundle.rs:1630-1659` precedent cited. Soundness chain re-verified against b822fb9 source:
- `synthesize.rs:590-604` (`synthesize_descriptor`) builds `pubkeys` via `cosigners.iter().enumerate().map(|(i, c)| (i as u8, xpub_to_65(&c.xpub)))` and the per-cosigner mk1 cards at `:620-640` iterate the SAME `cosigners` vector in the same order — slot index = pubkeys index.
- `synthesize_unified` builds `pubkeys` identically from `slots` (`:817-826`) then **delegates the whole card-emission back-half to `synthesize_descriptor(&descriptor, slots, …)`** (`:845`) — order identity holds by construction.
- All four `self_check_bundle` call sites confirmed downstream of synthesis: `cmd/bundle.rs:435` (every `BundleMode` arm routes through `synthesize_unified`, match at `:411-424`), `:1643` (after `:1587`), `:1690` (after `:1669`), `:1946` (import-json re-synthesizes at `:1914`; the `:1714-1716` pipeline-doc citation is accurate).
- `pubkeys: None` claim re-verified: the only `pubkeys: Some/None` construction sites are `synthesize.rs:149` (singlesig, exactly `[(0, xpub_65)]`), `:434` (self-multisig, `(i, xpub_65)` per slot), `:607`, `:833` — all `Some`. The `template.rs:518/:600` hits are inside `#[cfg(test)]` (`template.rs:286`). Defensive-only framing is correct.
- Current code confirmed to lack any xpub comparison in BOTH branches (Single `:2089-2114`, Multi `:2115-2141`); stub checks at `:2095`, `:2126-2139` as described. `xpub_to_65` is `pub` at `synthesize.rs:115` with the stated 65-byte layout.

**I-2 (cli_self_check.rs:5 doc fix + [obs] line) — FOLDED-OK.** Part 4 now corrects the module doc and claims the `[obs]` line. Verified: `tests/cli_self_check.rs:5` names `{bip84,wsh-sortedmulti}-mainnet-0-false-true.txt`; `:13` reads only the bip84 fixture; `wsh-sortedmulti-mainnet-0-false-true.txt` is in the 25-file delete set (26 fixtures total in `tests/vectors/v0_2/`, re-counted).

**I-3 (manual advisory prose + FULL lint) — FOLDED-OK.** Phase 3 adds inspect + repair advisory prose and runs the full manual lint. All four anchors re-verified live: `41-mnemonic.md:3001` = `## mnemonic inspect`, `:3105-3109` = its `### Advisories` table, `:2731` = `## mnemonic repair` (which has its own `### Advisories` table at `:2857` — a natural row anchor the plan doesn't name but the implementer will find), `:1501` = the per-occurrence wording precedent.

**I-4 (FOLLOWUPS promote + index lines) — FOLDED-OK.** One combined promoted entry per convention (precedent verified: `FOLLOWUPS.md:55` is the shared `### mk1-csi… (I10) + n1-vs-nge2…` entry), FIVE index lines — all five verified present and open: `:19` (inspect-repair), `:20` (localize), `:21` (orphaned-v0_2-multisig-goldens), `:23` (self-check-no-mk1-xpub-binding), `:33` (`[obs]` header-claims line) — plus the `### orphaned-v0_2-md1-vectors-no-harness` entry at `:157` (verified at that line).

**m-1 (stderr threading + pre-expansion) — FOLDED-OK.** Plan states the signature gains a stderr writer with both callers verified in scope (`cmd/inspect.rs:88-105` and `cmd/repair.rs:101-111` both have `stderr: &mut E`; both call the shared `repair::resolve_groups` — sole intake confirmed), and pins firing on RAW pre-expansion values. `expand_dashes` calls verified at exactly `repair.rs:339-341`.

**m-2 (unconditional flag fire + positional-only probe) — FOLDED-OK.** Firing rule now: unconditional on every non-`-` `--ms1` flag value; HRP-probe positionals only. Indel rationale verified — the strict typed-flag HRP gate is `if !relax_hrp_for_indel` at `repair.rs:284` (plan says `:285` — off-by-one, see Minor below).

**m-3 (positional label + per-occurrence) — FOLDED-OK.** `positional ms1` label + per-occurrence, against the verified `secret_in_argv_warning<W: Write>(stderr, flag, alternative)` at `secret_advisory.rs:40-45` (exact).

**m-4 (lint evidence hardening) — FOLDED-OK.** Append `"secret_in_argv_warning"` to the two Route rows — verified at exactly `tests/lint_argv_secret_flags.rs:92-93`, both with `source_file: "src/repair.rs"`, so the new anchor lands in the searched file.

**m-5 (bin-crate red-run + cross-slot cell + guards) — FOLDED-OK.** `mod self_check_ms1_tests` verified at `cmd/bundle.rs:2299-2300`; `--bin mnemonic` red-run stated; Multi cross-slot-swap cell present; guards cite the existing `cli_self_check.rs` cells; `mk_codec::encode_with_chunk_set_id` exposure re-confirmed (used in `synthesize.rs` live + test code).

**m-6 (two README paths) — FOLDED-OK.** Root `README.md:13` (`<!-- toolkit-version: 0.53.1 -->` verified at that line) + `crates/mnemonic-toolkit/README.md` (marker at `:9`); `scripts/install.sh:32` self-pin (`mnemonic-toolkit-v0.53.1`) verified.

**m-7 (gate.rs test placement) — FOLDED-OK in substance, stale anchor.** The cell goes in gate.rs's existing `#[cfg(test)]` module since `localize` is private — correct. But the cited line is wrong (see Minor m-A). All other M2 anchors re-verified exact: `localize` at `descriptor_builder/gate.rs:438-453` with the catch-all `Err(_) => None` and the `INVARIANT (M3)` comment at `:446-451`; narrowed twin `localize_parse_failure` `:405-418` (its `Err(miniscript::Error::NonTopLevel(_)) => None` arm proves the variant exists in the pinned miniscript); sole caller `:353` with `unwrap_or_else(|| "root")` root-fallback.

## Minor

**m-A — gate.rs `#[cfg(test)]` anchor is `:617`, not `:783+`.** The file is 1035 lines with exactly ONE `#[cfg(test)]` (grep count 1, at `descriptor_builder/gate.rs:617`). The `:783+` figure originated in my round-1 review and was folded verbatim without re-grep (the CLAUDE.md grep-at-write-time rule applies to folds too). Harmless — "the existing `#[cfg(test)]` module" is unambiguous — but fix the number.

**m-B — `--ms1` is single-occurrence; the "two inline values" per-occurrence TDD cell must mix sources.** `CardArgs::ms1()` returns `Option<&String>` (`src/repair.rs:212`) and `InspectArgs.ms1: Option<String>` (`cmd/inspect.rs:36`) — clap rejects a second `--ms1`. The per-occurrence cell therefore needs `--ms1 <inline>` + a positional `ms1…` value (or two positionals), not two flags. One clause in Part 2's TDD line prevents the implementer writing a clap-rejected cell.

**m-C — `:285` → `:284` off-by-one.** The indel-relaxation gate `if !relax_hrp_for_indel {` is at `repair.rs:284`.

**m-D — pin the positional advisory's `alternative` string.** `secret_in_argv_warning` requires an `alternative` ("pipe via {alternative}…"). For `--ms1` it's obviously `--ms1 -`; for a positional ms1 the only real stdin route is ALSO `--ms1 -` (positionals cannot be `-` — `classify_hrp_prefix` rejects non-HRP input, per the `count_dashes` doc at `repair.rs:218-221`). Say so, so the Phase-3 manual rows and the impl agree on exact wording first try.

## Verdict

**GREEN — 0 Critical / 0 Important / 4 minor.** All 11 round-1 findings folded correctly with no fold-drift; every load-bearing claim newly introduced by the folds re-verified against b822fb9 source (slot-exact soundness chain end-to-end, advisory-helper signature, `resolve_groups` signature/expansion/relaxation points, manual anchors, lint rows, FOLLOWUPS lines, README/install.sh markers). The four minors are anchor/wording fixes an implementer can absorb without a further review round — fold them and proceed to Phase 1.
