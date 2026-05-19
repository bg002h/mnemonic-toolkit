# v0.27.0 Phase 5 R0 Architect Review

**Date:** 2026-05-19
**Reviewer:** opus-4-7 (feature-dev:code-reviewer)
**Phase:** v0.27.0 Phase 5 — `bundle --import-json` + `export-wallet --from-import-json` consumer wiring
**Verdict:** GREEN with 2 Important findings (both folded pre-commit)

## Scope

Reviewed working-tree diff against `47daeb0`:
- NEW `crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs` (~365 LOC) — typed envelope parser + helpers
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — module declaration
- `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `--import-json` + `--import-json-index` clap args + `bundle_run_from_import_json` dispatch
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — `--from-import-json` + `--from-import-json-index` clap args + `run_from_import_json` dispatch
- NEW `crates/mnemonic-toolkit/tests/cli_bundle_import_json.rs` (12 cells + 1 regression)
- NEW `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` (10 cells + 1 cross-phase integration)

## Findings by severity

- Critical: **0**
- Important: **2** (folded pre-commit)
- Minor: **2** (deferred to next-cycle / acceptable)

## Important findings

### I1 — `descriptor_body_no_csum` silent fallback masks checksum-mismatch (confidence 85)

`crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs:349-352`:
```rust
pub(crate) fn descriptor_body_no_csum(descriptor_with_csum: &str) -> &str {
    miniscript::descriptor::checksum::verify_checksum(descriptor_with_csum)
        .unwrap_or(descriptor_with_csum)
}
```

The sibling code in `wallet_import/bsms.rs:141-145` does `.map_err(...)?` — propagates verification failure as `ImportWalletParse`. Phase 5's `.unwrap_or(descriptor_with_csum)` silently returned the descriptor with `#<csum>` retained on failure. Failure mode: hand-crafted/edited envelopes where the user drops a character from the descriptor body but keeps the original `#<csum>` get a cryptic downstream parse error instead of a clean checksum-mismatch error.

**Fold:** Changed helper to return `Result<&str, ToolkitError>` + propagate via `?` at the two call sites (`bundle.rs` + `export_wallet.rs`). Matches the bsms.rs convention. Added regression cell `bundle_import_json_tampered_descriptor_emits_bip380_checksum_error` (Cell 13) that mutates a fingerprint byte in the fixture and asserts the clean BIP-380 error.

### I2 — Plan-doc §3.7.1 stale field count claim (confidence 100)

Actual `EmitInputs` struct at `wallet_export/mod.rs:336-382` has **16 fields**. Phase 5's construction at `export_wallet.rs:587-609` correctly constructs all 16. The plan-doc heading at §3.7.1 line 546 ("17-field contract") and lines 548+550 ("17 fields", "all 17") were off-by-one — the canonical table at lines 554-569 enumerates 16 entries which IS exhaustive. The Phase 4 holistic review M1 erroneously bumped 16→17 without verifying against the live struct; this Phase 5 R0 caught it.

**Fold:** Plan-doc §3.7.1 header + body + §4.5 references all flipped 17→16, with a "REVISED twice" note explaining the off-by-one history.

## Minor findings (deferred)

### M1 — `path_raw` reconstruction for empty origin path produces trailing slash

At `json_envelope.rs:261-268`, for `card.origin_path == m`, the trim-then-format produces `[deadbeef/]`. Production sites in `bsms.rs:303` produce `[deadbeef/<path>]` for non-empty paths; the empty-path edge case isn't covered. Low impact (BIP-48/BIP-87 origin paths are never empty; mk1's `KeyCard.origin_path` from the Phase 4 envelope always has depth ≥ 1). FOLLOWUP for v0.28+ if it ever surfaces.

### M2 — Plan §6.3 smoke step 4 references nonexistent `--ms1` on bundle subcommand

`BundleArgs` doesn't expose `--ms1` (that's import-wallet's surface); seed overlay on bundle is `--slot @N.phrase=` only. Plan-doc bug surfaced via user's review questions. Not a Phase 5 implementation issue. Phase 6 doc-fold task: rewrite smoke step 4 from `--ms1 "abandon ..."` to `--slot @0.phrase="abandon ..."`.

## Holistic dimension verifications

### Taproot rejection at typed-deser layer

Phase 5 does NOT reject `tr(...)` at the deser layer. Behavior on `--from-import-json` with taproot envelope:
- BSMS emitter rejects (existing P2tr check in `wallet_export/bsms.rs:71-76`)
- Bitcoin Core / BIP-388 accept (descriptor-passthrough — fine for plain `tr(xpub)`)
- Sparrow / Jade / Coldcard accept with `taproot_internal_key: None` — this silently loses the multi_a-vs-key-path internal-key designation for `tr(sortedmulti_a(...))` envelopes

**Severity Important (not Critical)** because:
- No v0.27.0 wire-format path produces a `tr(sortedmulti_a(...))` envelope from `import-wallet` (BSMS rejects taproot at import; Bitcoin Core listdescriptors with tr() is a corner case)
- Plan-doc explicitly defers this to FOLLOWUP `wallet-import-taproot-internal-key`
- No test exercises this path

**Recommendation:** File the FOLLOWUP at cycle close as planned; no Phase 5 fold required.

### Cross-cutting confirmations

- **clap-derive mutex enforcement.** Verified at `bundle.rs:130-135` (`conflicts_with_all = ["template", "descriptor", "descriptor_file"]`) and `export_wallet.rs:159-164` (`conflicts_with_all = ["template", "descriptor"]`). The `required_unless_present_any` at bundle `template` arg includes `"import_json"`. Four mutex cells exercise parse-time refusal. PASS.

- **verify-bundle round-trip exercised.** Cell 12 at `cli_bundle_import_json.rs:425-494` correctly exercises a non-trivial 2-of-3 wsh-sortedmulti synthesis via `skip_middle_3of3_envelope_json` fixture; asserts `result == "ok"`. PASS.

- **17 (sic — actual 16) EmitInputs field enumeration.** Per I2 above, actual is 16 fields; Phase 5 constructs all 16 correctly. PASS modulo plan-doc count fix.

- **`BundleJsonView` deser + `deserialize_mk_field_normalized` soundness.** Sound. Probes first array element to disambiguate flat vs nested mk1; rejects malformed (mixed-shape) inputs via the trailing `_ => Err` arm. Empty array yields empty `Vec<Vec<String>>`. Two unit tests cover both shapes. Mirror struct correctly carries all 14 BundleJson fields + the union-shape mk1.

### User-asked uncertainty items

- **`emit_args.descriptor = Some(...)` clone-then-inject hack** at `bundle.rs:1565-1572`. Acceptable pattern for Phase 5 scope. Comment documents the rationale. Refactoring `emit_unified` to take an explicit `descriptor_override` would touch ~5-6 wire-up sites for a single-call-site benefit. FOLLOWUP if a second consumer ever needs the same pattern.

- **`--ms1` on bundle subcommand is a gap.** Confirmed correctly out of scope — `BundleArgs` doesn't expose it; seed overlay on bundle is `--slot @N.phrase=` only. Acceptable per project's permissive-input/expressive-output philosophy.

- **`entropy = Some(decoded)` without re-deriving xpub from envelope-ms1 entries.** Acceptable. The envelope was emitted by `import-wallet --ms1` which already did the xpub-match check at emit time. Re-verifying here would be double-work. The `--slot @N.phrase=` overlay path DOES re-derive and verify (user-supplied input). Symmetric treatment: trust envelope-ms1, verify user-supplied seed.

- **Sparrow/Jade/Coldcard/Electrum refuse descriptor-mode → no upstream gate needed.** Confirmed acceptable. Cell 3 in `cli_export_wallet_from_import_json.rs` pins the per-emitter refusal at v0.27.0; per-format ignored-input contract applies.

## Top-priority follow-up actions (folded in this commit)

1. ✅ **I1 fix:** `descriptor_body_no_csum` returns `Result<&str, ToolkitError>` + propagation at both call sites + new regression cell.
2. ✅ **I2 fix:** Plan-doc §3.7.1 + §4.5 references corrected from 17 → 16.
3. **FOLLOWUP-to-file at cycle close:** `wallet-import-taproot-internal-key` (silent acceptance of `tr(sortedmulti_a(...))` via `--from-import-json` for non-BSMS emitters). Already planned per §4.6.

## Verdict

**GREEN.** Both Important findings folded pre-commit. 1536 tests pass (was 1535 + 1 new regression cell). Clippy clean.
