# v0.28.0 Phase P0D — Architect Review R0

**Reviewer:** self-review (architect-style audit; no `feature-dev:code-architect` subagent dispatch tool available in this session — review performed inline against plan-doc + SPEC + source ground-truth grep).
**Branch:** `v0.28.0/p0d-sniff-format-consult-all`
**Base:** `release/v0.28.0` (HEAD `74c6119`)
**Commit reviewed:** `6ae4c65` ("P0D — sniff_format consult-all-then-count dispatch + 6 new SniffOutcome variants")
**Files changed:**
- `crates/mnemonic-toolkit/src/wallet_import/sniff.rs` (+148 / -45)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (+18 / -0)

## Verdict: GREEN (0 Critical, 0 Important, 2 Minor)

P0D implementation matches the plan-doc P0D row + SPEC §6.2/§6.3/§6.3.1 locks. Approach (a) (add all 6 new SniffOutcome variants in P0D scope) was taken per the prompt's explicit recommendation and is documented in the commit body. The `cmd/import_wallet.rs` scope-creep arm is bounded, statically unreachable in P0D, and superseded by P0C's dispatch pre-stub work.

## Critical: NONE

## Important: NONE

## Minor

### M1. Test-local `dispatch` helper duplicates production `sniff_format` arithmetic

**Location:** `crates/mnemonic-toolkit/src/wallet_import/sniff.rs:217-238`

The test `sniff_format_dispatches_consult_all_then_count` defines a local `dispatch(bools: [bool; 8]) -> SniffOutcome` helper that mirrors the production `sniff_format` body's arithmetic (votes array + filter + collect + match-on-len). If the production arithmetic changes (e.g., switches to first-match-wins, adds new variants, etc.) the test's local copy would silently drift.

**Mitigation already in place:** The test's docstring at lines 212-216 explicitly justifies the duplication: the function-under-test would force all 6 new bools to `false`, so we cannot exercise the (b) and (c) equivalence classes for the 6 new parser-positions via `sniff_format` directly. A test-local mirror is the only way to cover those positions until per-parser P{N}A wirings land.

**Recommended posture:** Accept as-is for P0D. When per-parser P{N}A sub-phases land and `sniff_format` itself can return each new variant, REPLACE the test-local `dispatch` helper with direct calls to `sniff_format` using synthetic blobs that hit each parser's sniff path. This is captured as future work, not a P0D-blocking finding.

Rationale-for-not-folding: folding would either (a) require per-parser blob fixtures that don't exist yet (P{N}A scope), or (b) require a refactor of `sniff_format` to take an injected `votes` array (overkill API change for a single dispatch).

### M2. `cmd/import_wallet.rs` catch-all arm format-list still says "bsms|bitcoin-core"

**Location:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:249-252` and `:256-259`

The existing Ambiguous / NoMatch arms in the `None => match sniff_outcome` block still emit `supply --format <bsms|bitcoin-core>` user-facing stderr text. With the 6 new formats added to `SniffOutcome` (and per plan-doc §B.2 #6 site 3 + R1-I2 lock, the post-cycle format list MUST enumerate all 8: `"bitcoin-core, bsms, coldcard, coldcard-multisig, electrum, jade, sparrow, specter"`), this stderr text is stale.

**Why not folded in P0D:** the plan-doc §B.2 #6 explicitly assigns this update to **P0C** ("Stderr templates at sites 3+5 enumerate the post-cycle 8-format list"), not P0D. P0D's scope per the prompt is strictly `sniff.rs:1-25`, `:43-52`, `:150-186`, plus the SniffOutcome enum decl. Extending P0D into the stderr-template update would be scope-creep into P0C's territory.

**Recommended posture:** Defer to P0C. The text is correct AT TIME OF P0D (only `bsms` and `bitcoin-core` are wired to dispatch), so user-facing semantics are unchanged in P0D. When P0C runs, it will refresh both arms' format-list strings as part of its 8-format pre-stub work (plan-doc P0C row, `cmd/import_wallet.rs` line 485).

## Verification of plan-doc / SPEC compliance

| Lock | Source | Status |
|------|--------|--------|
| Consult-all-then-count dispatch (R3-C2 fold) | plan-doc §B.2 #4 line 111-114 | sniff.rs:74-105 implements votes-array + filter + match-on-len |
| Votes-array shape avoids `unused_variables` (R4-I1) | plan-doc line 622 (P0D row) | sniff.rs:84-93; `cargo build` clean (no warnings) |
| Alphabetical PARSER-variant order (R4-I3) | plan-doc line 622 (P0D row); R5-I2 deferred-minor lock that aggregate outcomes are excluded | sniff.rs:84-93 enumerates `BitcoinCore, Bsms, Coldcard, ColdcardMultisig, Electrum, Jade, Sparrow, Specter` — alphabetical PARSER variants only |
| Test renamed (R4-C1 fold) | plan-doc line 622 (P0D row); §6.3.1 SPEC ref | sniff.rs:211 — `sniff_format_dispatches_consult_all_then_count` |
| Equivalence-class coverage (R4-I5 + R4-C1 fold) | plan-doc line 622 (P0D row) | sniff.rs:240-308 — (a) 0-true × 1 cell, (b) 1-true × 8 cells, (c) ≥2-true × 3 cells |
| SniffOutcome variants alphabetical (R5-I2) | plan-doc §B.2 #3 line 106-109; SPEC §6.2 line 185-188 | sniff.rs:51-63 — `Ambiguous / BitcoinCore / Bsms / Coldcard / ColdcardMultisig / Electrum / Jade / NoMatch / Sparrow / Specter` |
| `SniffOutcome: Copy` (P0D constraint per prompt) | prompt §"Critical constraints" | sniff.rs:51 — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` (already had Copy at HEAD; no change needed) |
| Doc-comment updated to 8-parser consult-all (per P0B.1 anchor + SPEC §6.2) | plan-doc P0D row, "Doc-comment update at sniff.rs:1-25" | sniff.rs:1-39 rewritten with alphabetical 10-variant list + N-parser SPEC §6.2 outcomes |
| `cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::sniff` green | prompt §"Tests / regression" | 11/11 unit tests pass (10 fixture + 1 new equivalence-class) |
| `cargo test -p mnemonic-toolkit --test cli_import_wallet_sniff` green | prompt §"Tests / regression" | 11/11 integration cells pass; no regression on {BSMS, BitcoinCore} subset |
| Approach (a): 6 new SniffOutcome variants added in P0D | prompt §"SOLUTION" / "Recommended approach: (a)" | sniff.rs:53-63 enum decl adds Coldcard / ColdcardMultisig / Electrum / Jade / Sparrow / Specter |
| Scope-creep documented in commit body | prompt §"If you take approach (a), explicitly document the choice in the commit message" | commit `6ae4c65` body §"SniffOutcome variant-addition approach: (a)" + §"Scope-creep disclosure: cmd/import_wallet.rs" |
| `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` green | tacit project convention | passes cleanly |

## Plan-doc / SPEC drift findings

NONE. The implementation matches the plan-doc P0D row + SPEC §6.2 + R3-C2 + R4-I1 + R4-I3 + R4-C1 + R4-I5 + R5-I2 fold locks 1:1.

## Plan-doc P0B.1 anchor relationship

P0B.1 (running in a parallel worktree per the prompt's CONFLICT NOTE) is scoped to `SniffOutcome` enum reorder at `sniff.rs:33-38`. P0D adds the 6 new variants to the enum AS WELL AS reorders the 4 existing variants into alphabetical order (Ambiguous, BitcoinCore, Bsms, NoMatch from prior `Bsms, BitcoinCore, Ambiguous, NoMatch`).

This means P0D + P0B.1 BOTH touch `sniff.rs:33-38` (enum decl) and BOTH alphabetize the 4 existing variants. At merge time the orchestrator will rebase one branch on the other; the conflict resolution is mechanical (both branches assert the same alphabetical order for the 4 existing variants; P0D additionally inserts the 6 new variants at their alphabetical positions).

P0D's commit body acknowledges this in the §"Scope-creep disclosure" section. The prompt explicitly says: "If you ALSO add 6 new SniffOutcome variants per approach (a) above, you ARE touching `:33-38` and will conflict with P0B.1 non-trivially. The orchestrator will handle the merge." — so this is anticipated, not a defect.

## Recommendation

**GREEN — approve for PR.** The 2 Minor findings (test-local dispatch duplication; stale stderr format-list in cmd/import_wallet.rs) are intentional/deferred and documented; neither is execution-blocking.

If folding M1 or M2 is desired before merge, the appropriate next steps would be:
- M1: defer to per-parser P{N}A sub-phases (replace synthetic `dispatch` helper with `sniff_format` calls on per-parser blob fixtures as they land).
- M2: defer to P0C scope (the prompt explicitly assigns the stderr-template refresh to P0C site 3).
