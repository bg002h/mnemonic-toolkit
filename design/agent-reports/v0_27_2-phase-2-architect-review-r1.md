# v0.27.2 Phase 2 architect review — R1

**Reviewer:** sonnet (Minor-fold verification; opus R0 already at -r0.md)
**Branch:** `release/v0.27.2` at Minor-fold commit `5398233`
**Date:** 2026-05-19
**Verdict:** GREEN (0 Critical / 0 Important / 0 Minor)

## Scope

R0 returned GREEN with 0C/0I/3 Minors. Fold commit `5398233` applied all 3:

- **M1** — Filed `pr-26-import-provenance-three-variant-cleanup` FOLLOWUP at `design/FOLLOWUPS.md:2524` with `Status: open`, `Tier: v0.28+`. Structure matches sibling entries (Surfaced / Where / What / Why deferred / Status / Tier).
- **M2** — `bsms.rs:8` + `bitcoin_core.rs:25` docstrings updated to reference accessor methods on `ParsedImport` (backed by `ImportProvenance::Bsms(Some(...))` / `ImportProvenance::BitcoinCore(...)`).
- **M3** — `provenance_tests` cells alphabetized:
  1. `provenance_accessors_return_references_not_owned`
  2. `provenance_bitcoin_core_variant_yields_none_bsms_audit_and_some_source_metadata`
  3. `provenance_bsms_no_audit_variant_yields_none_bsms_audit`
  4. `provenance_bsms_variant_yields_some_bsms_audit_and_none_source_metadata`

## Verification

- `cargo test --bin mnemonic provenance_tests`: **4/4 PASS**.
- Commit `5398233` touches exactly 4 files (bsms.rs, bitcoin_core.rs, mod.rs, FOLLOWUPS.md).
- No semantic mutations to `ImportProvenance` logic; diff is minimal and correct.
- No new `#[allow]` suppressions introduced.

## Verdict

**GREEN. Phase 2 fully closes.** Phase 4 dispatch is unblocked.
