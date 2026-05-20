# v0.28.0 P1C architect review — R0 (inline self-review)

**Phase:** P1C — Sparrow CLI dispatch flip + envelope wiring + integration tests.
**Reviewer:** inline self-review (agent-aa74aea6602d044ab).
**Date:** 2026-05-19.
**Scope of review:** the files mutated by P1C:

- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — Site 2 dispatch (`Some("sparrow") => ...`), Site 4 parse arm (`"sparrow" => SparrowParser::parse(...)`), Site 7 roundtrip envelope (canonicalize_sparrow byte-exact diff), envelope `sparrow_source_metadata` insertion, header doc + `--format` help text refresh, SparrowParser import.
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` — removed `#[allow(dead_code)]` on `SparrowSourceMetadata` (fields consumed by envelope emitter now).
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — removed `#[allow(dead_code)]` on `sparrow_source_metadata()` (called by envelope emitter).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs` (NEW) — 14 integration cells (4 parse happy-path + 3 sniff + 5 envelope/roundtrip + 2 refusal).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs` — removed `p0c_format_sparrow_panics_unimplemented` (stale post-P1C per P0C file header's "replace on flip" contract).

**SPEC anchor:** `design/SPEC_wallet_import_v0_28_0.md` §11.1.
**Plan-doc anchor:** P1C row at line 493.

## Verdict

**GREEN.** 0 Critical, 0 Important, 0 Minor. Ready to commit.

## Critical findings

(none)

## Important findings

(none)

## Minor findings

(none — surfaced and folded inline):

- **(folded)** `sparrow_json_envelope_roundtrip_byte_exact_on_canonical_fixture` initially asserted `byte_exact: true` on the Sparrow-native-shape fixture. Sparrow's NATIVE field order (`name, network, policyType, scriptType, defaultPolicy, keystores`) is NOT alphabetical, so `canonicalize_sparrow`'s BTreeMap-driven alphabetical re-emit produces a different byte-shape → byte_exact=false is correct. Renamed cell to `sparrow_json_envelope_roundtrip_status_ok_on_well_formed_fixture` and updated assertions to pin (status=ok, semantic_match=true, byte_exact=false).

## Verifications run

- `cargo build -p mnemonic-toolkit` → success.
- `cargo test -p mnemonic-toolkit --test cli_import_wallet_sparrow` → 14/14 pass.
- `cargo test -p mnemonic-toolkit --test cli_import_wallet_p0c_dispatch` → 9/9 pass (was 10 pre-P1C; removed sparrow panic cell).
- `cargo test -p mnemonic-toolkit` (full suite) → 106 test suites GREEN; no regression.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

## CLI dispatch parity with BSMS/Core (plan-doc P1C architect R0 review focus)

| Site | Before P1C | After P1C |
|---|---|---|
| Site 1 — `--format` PossibleValuesParser | `["bitcoin-core","bsms","coldcard","coldcard-multisig","electrum","jade","sparrow","specter"]` | UNCHANGED (P0C already wired all 8 values) |
| Site 2 — `Some("sparrow") =>` | `unimplemented!("P1C: format sparrow not yet wired")` | full dispatch: BSMS+Core sniff-mismatch checks → `"sparrow"` format_str |
| Site 3 — Ambiguous + NoMatch stderr templates | enumerate all 8 formats | UNCHANGED (already lists sparrow) |
| Site 4 — `"sparrow" =>` parse arm | `unimplemented!("P1C: parse not yet wired")` | `SparrowParser::parse(&blob, stderr)?` |
| Site 5 — `--select-descriptor` BSMS coerce | sparrow falls through to default | UNCHANGED (Sparrow has no analogous coerce need) |
| Site 6 — canonicalize dispatch | `"sparrow" => canonicalize_sparrow(blob)` | UNCHANGED (P0C already imported; P1B installed real body) |
| Site 7 — roundtrip envelope shape | `"sparrow" => json!({})` | full byte_exact/semantic_match/diff/status object mirroring bitcoin-core arm |
| Site 8 — provenance envelope field | NONE (only `bsms_audit` + `source_metadata` emitted) | `sparrow_source_metadata` inserted via `p.provenance.sparrow_source_metadata()` accessor |

## Cross-format sniff-mismatch matrix (cycle-followup candidate)

The existing v0.26.0 dispatch only ImportWalletFormatMismatches `--format bsms` vs `BitcoinCore` sniff (and vice versa). P1C's new `Some("sparrow") =>` arm extends the mismatch matrix to include `BSMS` + `BitcoinCore` sniffs vs `--format sparrow`. The REVERSE — `--format bsms` mismatching a `Sparrow` sniff — is NOT wired (the BSMS arm at Site 2 only checks against `BitcoinCore` sniff). Same for `--format bitcoin-core` against Sparrow sniff.

Documented in `tests/cli_import_wallet_sparrow.rs::sparrow_with_bsms_format_exits_format_mismatch` — the test acknowledges both ImportWalletFormatMismatch and ImportWalletParse as acceptable outcomes for the same input, with a comment explaining the dispatch ordering. Tracking as cycle-followup:

- **NEW** `wallet-import-format-mismatch-matrix-completion` — cross-format mismatch matrix only covers v0.26.0's BSMS ↔ BitcoinCore axis. P1C+ per-parser flips should extend the matrix symmetrically (every `--format X` arm should mismatch against EVERY other parser's sniff). v0.28.0 cycle-followup.

## Envelope wire-shape contract (sparrow_source_metadata)

```json
{
  "source_format": "sparrow",
  "bundle": { ... },              // unchanged v0.27.0 BundleJson
  "roundtrip": {                  // P1C Site 7
    "byte_exact": <bool>,
    "semantic_match": <bool>,
    "diff": <string|null>,
    "status": "ok|canonicalize_failed",
    "error": <string?>            // only when status="canonicalize_failed"
  },
  "sparrow_source_metadata": {    // P1C Site 8
    "label": <string|null>,       // top-level `name` from blob
    "policy_type": "SINGLE|MULTI",
    "script_type": "<verbatim Sparrow scriptType>",
    "dropped_fields": [<string>]
  }
}
```

Per the existing per-parser-accessor convention (`bsms_audit` for BSMS, `source_metadata` for Core), the field name `sparrow_source_metadata` is distinct from `source_metadata` so wire-shape parsers can index on field-name to distinguish format. Each per-parser cycle adds its own provenance field; envelope readers must check for presence of EACH known field rather than a single discriminator key.

## GUI schema-mirror lockstep — NO update required

Per CLAUDE.md: any `--format` PossibleValuesParser CHANGE requires `mnemonic-gui/src/schema/mnemonic.rs` lockstep. P1C does NOT change the PossibleValuesParser (P0C already wired all 8 values). The only flag-shape touch is the doc-comment refresh (`/// Format override...`), which is help-text not flag-schema. GUI schema mirror is current.

## P0C cell-replacement-on-flip discipline

The header of `tests/cli_import_wallet_p0c_dispatch.rs` (lines 25-32) documents:

> per-parser P{N}C sub-phases REPLACE these cells with happy-path parse cells anyway, so over-pinning the panic text creates a delete-on-arrival regression-cell.

P1C honors this by removing `p0c_format_sparrow_panics_unimplemented` (the dispatch no longer panics) and adding the equivalent happy-path coverage in `tests/cli_import_wallet_sparrow.rs`. Net cell count: 14 (sparrow integration) − 1 (p0c removal) = +13.

## Cycle-followups logged

- **NEW** `wallet-import-format-mismatch-matrix-completion` (Tier v0.28+): each `--format X` arm should symmetrically mismatch against every parser's sniff, not just the v0.26.0 BSMS↔Core axis. Surfaced in P1C; future per-parser cycles can fold incrementally.
