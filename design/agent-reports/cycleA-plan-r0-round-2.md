# R0 GATE REVIEW — IMPLEMENTATION_PLAN_cycleA_descriptor_use_site_collapse.md — Round 2 (convergence)

**Reviewer:** opus architect. **Against:** `origin/master @ 8c8b9183` (local HEAD `2aab6da3` = SPEC-GREEN commit, source tree byte-identical). Read-only, adversarial. Converging against `cycleA-plan-r0-round-1.md` (0C/4I). Persisted verbatim per CLAUDE.md.
**Verdict:** GREEN (0C/0I).

## Fold-by-fold verification
**I-A (merge Phase 1+2 atomic) — LANDED.** Phase 1 is one ATOMIC phase: 1a writes ALL failing tests (8 reject shapes + 9 positive controls + 6 per-surface CLI rejects + Sparrow passthrough control) → 1b implements residue check → 1c migrates ALL incumbent cells → 1d gates on FULL `cargo test -p mnemonic-toolkit` + `wc-codec` GREEN, then per-phase R0, commit only on GREEN. No committed RED boundary. Renumber clean (no orphan "Phase 2 migration"/"Phase 5"; gates 1d/2c/3e/4d).

**I-B (per-path verify variant) — LANDED + CONFIRMED.** Plan asserts CONCRETE verify → `DescriptorParse`/exit 2 PRIMARY (Phase 2a), `@N`-template → `DescriptorReparseFailed`/exit 4 secondary. Source: `verify_bundle.rs:1349-1357` concrete fork calls `descriptor_concrete_to_resolved_slots(...)?` BEFORE card compare (false-pass closed); `pipeline.rs:417-418` re-wraps as `DescriptorParse`; `verify_bundle.rs:~1375` (template fork) → `DescriptorReparseFailed{detail}`; `error.rs:597-598`: `DescriptorParse=>2`, `DescriptorReparseFailed=>4`. SPEC §4 D2 corrected to match.

**I-C (`:898` flip-reject, keep fixture) — LANDED + CONFIRMED.** Plan flips `core_fixture_file_mainnet_receive_change_pair_parses` `bundles=2 .success()` → reject, KEEPS `core-mainnet-receive-change-pair.json` unchanged, out of swap list. Sweep re-bucketed to "ASSERT-REJECT / keep fixture (NOT Group B)". Source `bitcoin_core.rs:892-906`: legacy `/0/*`+`/1/*` non-multipath pair (both reject under Part 1); `:915` (`<0;1>/*`, distinct FP_A/FP_B) retained as STAYS-PASSING + future merge-negative-control. Fixture preserved as pair-merge follow-up input.

**I-D (`/**` disclosure) — LANDED + MECHANISM CONFIRMED.** `/**` disclosed in CHANGELOG §3b(c) AND manual §3a with `<0;1>/*` workaround; CLI-level `--format descriptor` `/**` reject test (1a) + unit reject; follow-up `bip389-double-star-shorthand-support` filed (higher-impact-than-pair-merge flag). Mechanism: `concrete_keys_to_placeholders` `push_str(&descriptor[last_end..])` (`pipeline.rs:391`) ⇒ `@0[fp…]/**` ⇒ wild eats `/*`, residue `*` ⇒ REJECT; no pre-lexer `/**`→`<0;1>` expansion. Sparrow excepted (self-expands before lexing).

**MINORs — all present.** M-a inline-literal swaps explicit. M-b SPEC §8 line actually edited (STAYS exit 1, supersedes "exit 2 not 1"). M-c Sparrow discharge + positive-control test. M-d MINOR bump in BOTH plan + SPEC. M-e a4/a5 both legs assert reject.

**No new drift.** Renumber introduced no orphan/mis-numbered gate; Group-B list no longer contains `:898`; `:915` STAYS-PASSING not moved; Phase-1 residue snippet unchanged and correct — byte-logical mirror of md-cli `template.rs:128-137`, placed after `.transpose()?` validator (`parse_descriptor.rs:177`) before `out.push` (`:183`), panic-safe, `i` in scope, "adapt don't add bare-origin group" retained.

## CRITICAL
None. Residue floor fail-closed; no path produces a card; every reject precedes card comparison/bundle build.

## IMPORTANT
None. All four round-1 Importants folded correctly + independently re-verified against source.

## MINOR (non-blocking)
- **m1.** SPEC §8 line still says the verify false-pass test asserts "(exit≠0, `DescriptorReparseFailed`)" — inconsistent with the corrected §4 D2 + Phase 2a (concrete → `DescriptorParse`/exit 2). Plan Phase 2a is authoritative + correct; recommend updating the SPEC §8 line on a future touch.
- **m2.** Two carried citation nits: plan cites `pipeline.rs:401` for the verbatim `/**` push (actual `push_str` is `:391`) and `parse_descriptor:852` for the `lex_placeholders` call (actual `:853`). Mechanism unaffected.
- **m3.** Phase-2 funds regressions are born-green guards (sanctioned consequence of the I-A atomic merge — the fix lands in Phase 1). Harmless; meaningfulness verifiable by a temporary revert. No change required.

## VERDICT
**GREEN (0C/0I).** All four round-1 Importants + five Minors folded correctly, each independently re-verified against origin/master source. No Critical/Important/new-drift. The three residual Minors are non-blocking tidy-ups. **Cleared to begin Phase 1 implementation** (atomic: tests → residue check → full migration → GREEN → per-phase R0).
