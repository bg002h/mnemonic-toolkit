# v0.28.0 P2C — architect R0 review

**Phase:** P2C — 8-site Specter dispatch flip + integration tests.
**Branch:** `v0.28.0/p2-specter`.
**Base:** P2B commit `0990315`.
**Verdict:** GREEN (0 Critical / 0 Important / 1 Minor — filed as cycle-internal FOLLOWUP `wallet-import-cross-format-symmetric-mismatch`).

## Scope verified

Plan-doc P2C row (`/home/bcg/.claude/plans/unified-meandering-sundae.md:501`):

> Flip 8 dispatch sites in `cmd/import_wallet.rs`. Integration cells.

Sub-phase prompt directives:

> Flip 8 dispatch sites in `cmd/import_wallet.rs`. Add `ImportProvenance::Specter` variant alphabetically (after Sparrow). Add `SniffOutcome::Specter => "specter"` arm BEFORE the `other =>` catch-all. Integration tests.

The `ImportProvenance::Specter` variant was added at P2A (alphabetical slot after `Bsms`); the prompt's "after Sparrow" wording is technically off (alphabetical order is `BitcoinCore < Bsms < Specter`; Sparrow lands at P1A and would sit between Bsms and Specter when both are present). At P2A's authoring time, Sparrow doesn't exist yet, so Specter went after Bsms. When Sparrow lands, it'll insert between Bsms and Specter, leaving Specter's alphabetical position correct. Confirmed acceptable.

## Files touched

1. `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — 8-site dispatch flip:
   - **Site 1 (import):** added `specter::SpecterParser` + `ImportProvenance` to the import list.
   - **Site 2 (`--format` arm):** replaced `Some("specter") => unimplemented!()` with a real arm that rejects `SniffOutcome::Bsms` + `SniffOutcome::BitcoinCore` via `ImportWalletFormatMismatch`. The arm honors all other SniffOutcome values (Specter, Ambiguous, NoMatch, and any unimplemented sister-format outcomes) since they don't represent a known-other-vendor collision.
   - **Site 3 (auto-sniff `None =>`):** added `SniffOutcome::Specter => "specter"` arm BEFORE the `Ambiguous` / `NoMatch` error arms (alphabetical position after `BitcoinCore` + `Bsms`).
   - **Site 4 (parse dispatch):** replaced `"specter" => unimplemented!()` with `"specter" => SpecterParser::parse(&blob, stderr)?`.
   - **Site 5 (`--select-descriptor` coerce):** added a `"specter" =>` coerce arm matching the BSMS pattern (Specter is single-descriptor per SPEC §11.2; non-`all` selects coerce to `all` with stderr NOTICE).
   - **Site 6 (canonicalize dispatch):** UNCHANGED — already wired at P0C via `canonicalize_specter(blob)`.
   - **Site 7 (roundtrip envelope):** replaced `"specter" => json!({})` placeholder with full roundtrip-envelope construction mirroring the `bitcoin-core` arm (byte_exact + semantic_match + diff + status + error fields).
   - **Site 8 (stderr-WARNING):** UNCHANGED — Specter takes the `format_str != "bitcoin-core"` early-return path (no stderr WARNING; envelope-only roundtrip). Plan-doc Site 8 noted "per-parser P{N}B may decide to surface a roundtrip WARNING on stderr" — P2C decided NO (matches BSMS treatment; Specter blobs are JSON and the stderr WARNING shape is BSMS/Core-text-specific).
   - **NEW: `source_metadata` envelope arm:** added a sister `if let ImportProvenance::Specter(ref meta) = p.provenance { ... }` block alongside the existing `if let Some(meta) = p.source_metadata()` Bitcoin-Core block. Wire shape per SPEC §11.2 — emits `{label, blockheight, devices: [{type, label}, ...], dropped_fields}`. The accessor convention (`p.source_metadata()` is a typed `&CoreSourceMetadata` accessor) was NOT extended; Specter uses direct pattern match. A `p.specter_metadata()` accessor mirror could land in a future refactor; YAGNI for now.

2. `crates/mnemonic-toolkit/src/wallet_import/mod.rs`:
   - Lifted `#[allow(dead_code)]` from `ImportProvenance::Specter` variant — the variant's payload is now READ by the envelope-emit site at Site 7.

3. `crates/mnemonic-toolkit/src/wallet_import/specter.rs`:
   - Lifted `#[allow(dead_code)]` from `SpecterSourceMetadata` + `SpecterDeviceMarker` struct annotations — fields now actively read by the envelope-emit dispatch site.

4. **NEW** `crates/mnemonic-toolkit/tests/cli_import_wallet_specter.rs` (264 lines, 18 integration tests):
   - 4 happy-path parse cells (singlesig P2WPKH, multisig 2-of-3 wsh-sortedmulti, blockheight-zero with legacy string devices, with-checksum).
   - 2 auto-sniff dispatch cells (singlesig + multisig route via `SniffOutcome::Specter`).
   - 2 sniff-mismatch cells (--format specter against BSMS fixture, against Core fixture → exit 1 ImportWalletFormatMismatch).
   - 4 `--json` envelope cells (source_format wire-shape; source_metadata.{label,blockheight,devices,dropped_fields}; roundtrip object shape; multisig 3-devices envelope wire-shape; legacy string-devices normalized to object-form in envelope).
   - 2 `--select-descriptor` coerce cells (non-default coerces with NOTICE; `all` is silent).
   - 4 stdin / error-handling cells (stdin-vs-file parity; dropped-top-level-fields NOTICE; invalid-JSON parse error; invalid-checksum parse error).

5. `crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs`:
   - Removed `p0c_format_specter_panics_unimplemented` cell + its body. Per the P0C self-doc lock ("per-parser P{N}C sub-phases REPLACE these cells with happy-path parse cells anyway"), the deletion is the matching half of the regression-cell promotion. Coverage moves to `tests/cli_import_wallet_specter.rs`.

6. `design/v0_28_0-cycle-followups.md`:
   - Filed `wallet-import-cross-format-symmetric-mismatch` cycle-internal FOLLOWUP (1 Minor; see "Items surfaced not in scope" below).

## Test coverage

- Specter unit tests: 39 (unchanged from P2B; P2C is dispatch-only).
- Specter integration tests: **18 NEW** in `cli_import_wallet_specter.rs`; all green.
- Specter canonicalize tests: 11 (unchanged from P2B).
- Sniff dispatcher tests: 17 (unchanged from P2A; +6 specter cells already landed there).
- Cross-references: `cli_import_wallet_bitcoin_core.rs::specter_like` still passes — the Specter-shaped blob through `--format bitcoin-core` now sniffs as `SniffOutcome::Specter` (P2A wired); the `Some("bitcoin-core")` arm doesn't reject Specter sniff (only `Bsms`) so it accepts the override and runs Core parser, which fails with `ImportWalletParse(missing descriptors array)` → exit 2. Test assertion unchanged.
- Full suite: 106 test binaries passing (was 105 at P2B; +1 new specter integration file). 0 failures.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean.

## Discipline checks

- **Alphabetical ordering:** the new `SniffOutcome::Specter` arm in cmd's `None =>` dispatch sits at the alphabetical slot after `BitcoinCore` + `Bsms`. The new `Some("specter")` arm at Site 2 sits at the same alphabetical slot as the existing `Some(...)` arms (preserved from P0C). The new "source_metadata" envelope `if let` block for Specter sits AFTER the bitcoin-core `if let Some(meta) = p.source_metadata()` block (alphabetical via match-block convention).
- **No new `ToolkitError` variants:** dispatch reuses existing `ImportWalletFormatMismatch` + `ImportWalletParse` + `ImportWalletXprvForbidden` variants. No `error.rs` touched.
- **No `git add -A`:** P2C commit stages 5 explicit paths.
- **Architect-review persistence:** this file at `design/agent-reports/v0_28_0-P2C-r0-review.md` BEFORE commit.
- **Cross-repo lockstep:** P2C does NOT change the clap surface (the `--format specter` value was added at P0C; PossibleValuesParser unchanged here). No mnemonic-gui schema-mirror update required. The cycle-end Phase P15 covers the full schema-mirror bump.

## Items surfaced not in scope (filed as cycle FOLLOWUPs)

### Minor — `wallet-import-cross-format-symmetric-mismatch`

The existing `Some("bsms")` ↔ `Some("bitcoin-core")` arms at `cmd/import_wallet.rs:246-263` perform only the obvious-pair sniff-mismatch check. Now that P2C adds the `Some("specter") => { match sniff_outcome { ... } }` arm with explicit rejects for `SniffOutcome::Bsms` + `SniffOutcome::BitcoinCore`, the inverse pairs (`Some("bsms")` rejecting `SniffOutcome::Specter`, `Some("bitcoin-core")` rejecting `SniffOutcome::Specter`, etc.) are NOT covered. With 8 formats post-cycle, the matrix is 8×7=56 mismatch pairs.

The plan-doc P{N}C row literally says "Flip 8 dispatch sites" — not "extend symmetric N+1 mismatch coverage." Existing tests (`cli_import_wallet_bitcoin_core.rs:548 specter_like`) still pass because the wrong parser produces a typed `ImportWalletParse` (less precise but still correct behavior). Filed as cycle-internal FOLLOWUP at `design/v0_28_0-cycle-followups.md` for P14A triage; recommendation is a generalized `check_format_mismatch(supplied, sniffed) -> Result<...>` helper rather than 56 hand-typed match arms.

## Conclusion

P2C scope COMPLETE per plan-doc + SPEC §11.2. All 8 dispatch sites wired; 18 new integration tests + lifted `#[allow(dead_code)]` annotations + 1 self-replacing regression cell removed. Verdict: GREEN — Phase 2 (P2A + P2B + P2C) ready to open PR against `release/v0.28.0`.
