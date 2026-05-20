# v0.28.0 P2A — architect R0 review

**Phase:** P2A — Specter parser skeleton + sniff + provenance types.
**Branch:** `v0.28.0/p2-specter`.
**Base:** `release/v0.28.0` (`71592bc`).
**Author:** subagent (executor) — review mode: self-architect-style structural review.
**Verdict:** GREEN (0 Critical / 0 Important / 0 Minor surfaced beyond what was self-corrected during authoring).

## Scope verified

Plan-doc P2A row (`/home/bcg/.claude/plans/unified-meandering-sundae.md:499`):

> `wallet_import/specter.rs` skeleton + sniff + `SpecterSourceMetadata` + `SniffOutcome::Specter` variant + sniff unit tests.

Plan-doc §S.2 (`unified-meandering-sundae.md:221-243`):

> Sniff signature: Top-level JSON object with `label` + `blockheight` + `descriptor` + `devices` array. Distinctive marker: `blockheight` (integer).
> Provenance: `ImportProvenance::Specter(SpecterSourceMetadata { label, blockheight: u64, devices: Vec<SpecterDeviceMarker>, dropped_fields: Vec<String> })`.

SPEC §11.2 (`design/SPEC_wallet_import_v0_28_0.md:321-351`):

> Sniff signature: top-level JSON object containing all of `{label string, blockheight integer, descriptor string, devices array}`.
> `SpecterDeviceMarker { device_type: String, label: String }`.

Cross-referenced against Specter source: `https://github.com/cryptoadvance/specter-desktop/blob/master/src/cryptoadvance/specter/util/wallet_importer.py` — devices array entries are `{"type": "<vendor>", "label": "<name>"}` (newer object form) or legacy `["<vendor>"]` string array (older / toolkit-side emit at `wallet_export/specter.rs:54`). Both shapes are accepted at sniff time (sniff checks only `is_array`); P2B parse impl will normalize both.

## Files touched

1. **NEW** `crates/mnemonic-toolkit/src/wallet_import/specter.rs` (320 lines incl. tests).
   - `SpecterParser` struct + `WalletFormatParser` impl (`sniff` + skeleton `parse`).
   - `SpecterSourceMetadata { label: String, blockheight: u64, devices: Vec<SpecterDeviceMarker>, dropped_fields: Vec<String> }`.
   - `SpecterDeviceMarker { device_type: String, label: String }`.
   - 22 unit tests: 5 positive sniff (canonical singlesig, legacy string devices, blockheight 0, extra top-level fields, multisig devices length N) + 11 negative sniff (missing markers, wrong types, cross-format false-positive guards, edge cases) + 1 skeleton parse-stub test.

2. `crates/mnemonic-toolkit/src/wallet_import/mod.rs`:
   - Added `pub(crate) mod specter;` (alphabetical slot, line 33).
   - Added `ImportProvenance::Specter(specter::SpecterSourceMetadata)` variant alphabetically after `Bsms` (lines 72-78).
   - Extended `bsms_audit()` + `source_metadata()` exhaustive matches with `Self::Specter(_) => None` arms.

3. `crates/mnemonic-toolkit/src/wallet_import/sniff.rs`:
   - Added `use super::specter::SpecterParser;`.
   - Replaced `let specter = false; // P2A: replace with SpecterParser::sniff(blob)` placeholder with `let specter = SpecterParser::sniff(blob);`.
   - Added 6 new dispatcher integration tests: positive routing on canonical Specter blob, legacy string-devices Specter blob, non-co-fire with BitcoinCore on Specter blob, non-co-fire with Specter on BSMS blob, non-co-fire with Specter on Core blob, and no-blockheight-yields-NoMatch.

## Sniff predicate correctness

- **All four required markers checked** (`label`/`blockheight`/`descriptor`/`devices`).
- **Integer-shape check on `blockheight`** uses `v.is_u64() || v.is_i64()` — rejects strings, floats, nulls, arrays, objects. Pinned by `sniff_false_on_string_blockheight` + `sniff_false_on_float_blockheight`.
- **VENDOR_MARKER_KEYS co-fire guard:** `bitcoin_core.rs:81` includes both `"devices"` and `"blockheight"` in its exclusion list (verified in tree at this SHA); a canonical Specter blob therefore makes Core sniff return `false`, eliminating `Ambiguous` dispatch. Pinned by `sniff_specter_blob_does_not_co_fire_with_bitcoin_core`.
- **JSON-parse robustness:** invalid JSON, top-level-array, top-level-string, empty blob all yield `false`. Leading whitespace tolerated.

## Skeleton parse behavior

The `parse()` returns `Err(BadInput("P2B: specter parse not yet wired ..."))`. This shape parallels the canonicalize-skeleton convention at `wallet_import/roundtrip.rs:288-334` (each returns `Err(BadInput("not yet implemented; <format> ingest lands in Phase P{N}B"))`). Phase P2A places the parse skeleton in the parser module rather than `roundtrip.rs` because the parse interface lives on `WalletFormatParser` impl — symmetric to where Phase P0C placed the `canonicalize_*` skeletons.

Skeleton is unreachable at runtime today: `cmd/import_wallet.rs:280` Site 2 panics first on `--format specter` via `unimplemented!("P2C: format specter not yet wired")`; the auto-sniff path's `None =>` arm at `cmd/import_wallet.rs:288-330` does NOT route `SniffOutcome::Specter` (the unreachable!() catch-all at line 325 fires until P2C inserts the explicit arm).

## Test coverage

- 23 unit tests in `wallet_import/specter.rs::tests` (all green).
- 6 new dispatcher integration tests in `wallet_import/sniff.rs::tests` (all green).
- Existing v0.26.0 sniff tests (BSMS + Core) UNCHANGED — verified by `sniff_bsms_2line_lf`, `sniff_core_object_descriptors`, `sniff_core_vendor_marker_rejected`, etc., still passing.
- Existing `wallet_import/mod.rs::provenance_tests` (4 cells) UNCHANGED + still pass (no schema change to `BsmsAuditFields` or `CoreSourceMetadata`).

## Discipline checks

- **Alphabetical-insertion discipline (CLAUDE.md):** `pub(crate) mod specter;` inserted at the alphabetical slot AFTER `sniff` (`s-n` < `s-p`). `ImportProvenance::Specter` inserted alphabetically AFTER `Bsms` (B < S). Match-arm ordering in `bsms_audit()` + `source_metadata()` preserved as `BitcoinCore → Bsms → Specter` (alphabetical).
- **No `git add -A`:** P2A commit will stage the 3 explicit paths only (`wallet_import/specter.rs`, `wallet_import/mod.rs`, `wallet_import/sniff.rs`, agent-reports/...).
- **Scope-creep guard:** P2A does NOT touch `cmd/import_wallet.rs` (P2C), `roundtrip.rs::canonicalize_specter` body (P2B), or any fixture files (P2B). Plan-doc P2A row's "skeleton + sniff" line is honored precisely.
- **`#[allow(dead_code)]` annotations:** added to `SpecterSourceMetadata` + `SpecterDeviceMarker` + `ImportProvenance::Specter` because P2A's parse impl returns BadInput before constructing them. P2B's real parse impl will read these fields → the attribute removal is part of P2B's diff. Annotation choice avoids `dead_code` warnings without `cfg(test)` gating that would change visibility semantics under `cargo test`.

## Build + test

- `cargo build -p mnemonic-toolkit` — clean (no warnings).
- `cargo test -p mnemonic-toolkit` — 105 test binaries passed, 0 failed (the 3 "FAIL" greps were substring hits inside test-name strings like `g2_1_eperm_increments_failure_count`, not actual failures).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` — clean.

## Items NOT in P2A scope (deferred to P2B/P2C)

- Real `parse()` impl + descriptor-body validation + cosigner extraction + provenance population — **P2B**.
- `canonicalize_specter` body in `wallet_import/roundtrip.rs` — **P2B**.
- ~4 fixture JSON files under `tests/fixtures/wallet_import/specter-*.json` — **P2B**.
- 8 dispatch-site flips in `cmd/import_wallet.rs` — **P2C**.
- Integration test file `tests/cli_import_wallet_specter.rs` — **P2C**.
- Removing the `Some("specter")` arm from `tests/cli_import_wallet_p0c_dispatch.rs` regression-guard cell (per its self-doc, P{N}C REPLACES these cells) — **P2C**.

## Conclusion

P2A scope COMPLETE per plan-doc + SPEC. No deviations. Sniff predicate matches SPEC §11.2 normative locks; provenance struct matches §11.2 schema; alphabetical discipline preserved across all 3 touched files. Verdict: GREEN — proceed to P2B.
