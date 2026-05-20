# v0.28.0 — Phase P1A (Sparrow sniff skeleton) — R0 architect review

**Branch:** `v0.28.0/p1-sparrow-v2`
**Base:** `release/v0.28.0` @ `d7a2859`
**Scope:** sniff skeleton + `SparrowSourceMetadata` + `ImportProvenance::Sparrow` enum variant + `sniff_format` votes-slot flip + `cmd/import_wallet.rs` `SniffOutcome::Sparrow => "sparrow"` arm.
**Date:** 2026-05-19
**Reviewer:** self-architect-review (R0)

## Status: GREEN

## Methodology

Plan-doc anchors (`/home/bcg/.claude/plans/unified-meandering-sundae.md`):

- §S.1 (Sparrow normative scope) — lines 197-219
- §Phase P1A scope row at line 491 — "wallet_import/sparrow.rs skeleton: SparrowParser struct + WalletFormatParser::sniff impl + SparrowSourceMetadata struct. NO parse impl. Wires SniffOutcome::Sparrow variant. Sniff unit tests."

SPEC anchor (`design/SPEC_wallet_import_v0_28_0.md` §11.1, lines 289-319): sniff signature + provenance schema + canonicalize helper signature.

## Source-grep verification (against current branch HEAD)

### sniff.rs votes-array slot for Sparrow

`crates/mnemonic-toolkit/src/wallet_import/sniff.rs:83` flips from `false` placeholder to `SparrowParser::sniff(blob)`. Adjacent slots preserved as `false` for the other 5 still-placeholder parsers (Coldcard / Electrum / Jade / Specter; ColdcardMultisig was wired at P4A).

The `votes` array at `sniff.rs:86-94` retains its alphabetical-by-variant-name ordering anchor from P0D — `(sparrow, SniffOutcome::Sparrow)` sits at index 6 between `(jade, ...)` and `(specter, ...)`. No structural reorder.

### Module wiring

`crates/mnemonic-toolkit/src/wallet_import/mod.rs:34` adds `pub(crate) mod sparrow;` alphabetically after `sniff`.

### ImportProvenance variant alphabetical insertion

Pre-P1A enum at `mod.rs:64-85`: `BitcoinCore, Bsms, ColdcardMultisig`. P1A inserts `Sparrow(sparrow::SparrowSourceMetadata)` after `ColdcardMultisig` at the alphabetically-correct slot. The CLAUDE.md alphabetical-by-variant-name discipline is honored.

`bsms_audit` + `source_metadata` exhaustive match arms (`mod.rs:91-107`) are extended with `Self::Sparrow(_) => None` arms — exhaustiveness preserved at compile time.

### Sparrow parser file

`crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` is new (~330 LOC). Sniff impl follows SPEC §11.1 exactly:

- Step 1 (JSON-parse + top-level object check) — `sparrow.rs:122-128`.
- Step 2 (`policyType` ∈ {`SINGLE`, `MULTI`}) — `sparrow.rs:130-138`.
- Step 3 (`scriptType` string-typed) — `sparrow.rs:139-142`.
- Step 4 (`defaultPolicy.miniscript.script` nested string) — `sparrow.rs:143-153`.
- Step 5 (`keystores` non-empty array) — `sparrow.rs:154-163`.

`parse` is intentionally `unimplemented!("P1B: ...")` (`sparrow.rs:172`) — invocation panics with an unambiguous P1B marker. The pre-existing P0C regression cell `p0c_format_sparrow_panics_unimplemented` at `tests/cli_import_wallet_p0c_dispatch.rs:57` continues to pass against the cmd-side P1C-pending unimplemented dispatch arm (not the parser's own unimplemented body) — see "Test surface" below.

### cmd/import_wallet.rs auto-sniff dispatch arm

`SniffOutcome::Sparrow => "sparrow"` is inserted AFTER `ColdcardMultisig` and BEFORE the catch-all `other => unreachable!()` at `cmd/import_wallet.rs:321-336`. This matches the C/F learned-best-practice: each per-parser P{N}A sub-phase pre-emptively adds the auto-sniff arm BEFORE the catch-all so the unreachable contract stays intact when the SniffOutcome variant goes live. The explicit Some("sparrow") parse-side dispatch arm at `cmd/import_wallet.rs:301` remains `unimplemented!("P1C: format sparrow not yet wired")` — P1C flips that.

## Test surface (P1A cells)

15 cells under `wallet_import::sparrow::tests`:

- 3 positive-sniff (single P2WPKH, multi 2-of-3 P2WSH, P2TR singlesig).
- 9 negative-sniff (BSMS / Bitcoin Core / Specter / empty-keystores / missing-nested-script / unrecognized-policyType-value / bare-array / random-text / empty-blob).
- 1 panic guard for `parse` (`#[should_panic(expected = "P1B")]`).
- 1 `build_provenance` round-trip (asserts `ImportProvenance::Sparrow(_)` variant).
- 1 `SparrowPolicyType::from_str` matrix.

Plus 1 unchanged existing P0C cell at `tests/cli_import_wallet_p0c_dispatch.rs:57` (`p0c_format_sparrow_panics_unimplemented`) — passes because the cmd-side `Some("sparrow")` arm still panics until P1C.

`cargo test -p mnemonic-toolkit` post-P1A: 553 passed (lib + bin) + integration suites all green. Local total: 0 FAILED. `cargo clippy --all-targets -- -D warnings` clean.

## Findings

### Critical
**None.**

### Important
**None.**

### Minor

#### M1 — Sniff doc-comment claims "Vendor markers shared with no other format ⇒ no Ambiguous risk."

This is asserted in the plan-doc (`§S.1` line 199) and SPEC (`§11.1` line 297). The current P1A sniff is positive-marker on `policyType` + `scriptType` + `defaultPolicy` + `keystores`. The Bitcoin Core sniff at `bitcoin_core.rs:81-97` has `policyType` and `defaultPolicy` and `keystores` in `VENDOR_MARKER_KEYS` — a blob carrying both Core's `descriptors` AND Sparrow's markers would fail Core's sniff (vendor-marker absence check) but pass Sparrow's. No co-fire risk from Core. Other parsers (Specter / Coldcard / Electrum / Jade) are still placeholder `false` so no co-fire risk possible. **Confirmed: M1 is informational, not actionable.**

### Folds applied this round

None (R0 GREEN).

## Sign-off

P1A scope per plan-doc row 491 is fully implemented:
- ✓ `wallet_import/sparrow.rs` skeleton (SparrowParser struct + WalletFormatParser::sniff impl + SparrowSourceMetadata struct)
- ✓ NO parse impl (intentional `unimplemented!("P1B: ...")`)
- ✓ Wires `SniffOutcome::Sparrow` variant (`sniff.rs:83` + `cmd/import_wallet.rs:336`)
- ✓ Sniff unit tests (3 positive + 9 negative + 3 scaffolding = 15 cells)
- ✓ `ImportProvenance::Sparrow(SparrowSourceMetadata)` enum variant added alphabetically

Per CLAUDE.md "Per-phase reviewer-loop until 0 critical / 0 important" — R0 hits 0C/0I. No further rounds needed for P1A in isolation; P1B re-dispatches.
