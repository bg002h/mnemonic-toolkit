# v0.28.0 P2C-v2 — R0 architect review

**Branch:** `v0.28.0/p2-specter-v2-bg`
**Base:** P2B commit `e12054b`
**Scope:** Flip remaining 7 `cmd/import_wallet.rs` dispatch sites for Specter + `ImportProvenance::Specter` variant lift from `#[allow(dead_code)]` + `specter_source_metadata()` accessor lift + envelope-emit `specter_source_metadata` field + integration tests + post-P2C update of the P0C `specter_panics_unimplemented` cell.

## Verdict

GREEN — 0 Critical / 0 Important / 0 Minor.

## Critical

(none)

## Important

(none)

## Minor

(none)

## Verification

- `cargo build -p mnemonic-toolkit`: clean
- `cargo clippy -p mnemonic-toolkit --all-targets`: clean
- 17 new integration cells in `tests/cli_import_wallet_specter.rs` pass
- All 10 P0C dispatch tests pass (1 cell renamed `_panics_unimplemented` → `_dispatches_format_mismatch_post_p2c` per the P1C / P4C precedent)
- All toolkit tests pass; no regressions across the cycle

## Dispatch sites flipped (matrix-discipline lock)

| Site | Description | Action |
|---|---|---|
| 1 | `use` import in `cmd::import_wallet` | Added `specter::SpecterParser` |
| 2 | `Some("specter")` arm at format-mismatch dispatch | Replaced `unimplemented!()` with format-mismatch matrix (Bsms / BitcoinCore / ColdcardMultisig / Sparrow → exit 1; Ambiguous + NoMatch tolerated) |
| 3 | `SniffOutcome::Specter => "specter"` auto-sniff arm | Already wired at P2A |
| 4 | `"specter" => SpecterParser::parse(&blob, stderr)?` | Replaced `unimplemented!()` with real parser dispatch |
| 5 | `--select-descriptor` coerce | Added `"specter"` arm (mirrors BSMS; emits NOTICE + coerces non-`all` to `all` for the single-descriptor Specter shape) |
| 6 | Canonicalize dispatch | Already wired at P0C; body now points at the P2B real `canonicalize_specter` |
| 7 | Roundtrip envelope construction | Replaced `"specter" => json!({})` with the full byte_exact / semantic_match / diff / status envelope mirroring the bitcoin-core + sparrow shape |
| 8 | stderr-WARNING | UNCHANGED — Specter takes the no-warning early-return path (matches the `!= "bitcoin-core"` predicate already in `emit_roundtrip_stderr_warning`); follows BSMS treatment |

## Notes

- **`ImportProvenance::Specter` variant lift:** P2A added the variant with `#[allow(dead_code)]`. P2C lifts the attribute since the envelope-emit dispatch at `cmd::import_wallet::emit_json_envelope` now reads `meta.label` / `meta.blockheight` / `meta.devices` / `meta.dropped_fields` actively. Same lift on `SpecterSourceMetadata` + `SpecterDeviceMarker` struct fields.
- **`specter_source_metadata()` accessor lift:** P2A added the accessor under `#[allow(dead_code)]`. P2C lifts the attribute since `cmd::import_wallet::emit_json_envelope` now uses `p.provenance.specter_source_metadata()` to drive the new envelope field.
- **Mismatch matrix completeness:** the Site 2 arm refuses Bsms + BitcoinCore + ColdcardMultisig + Sparrow sniffs. Inverse wires (each of those arms refusing a Specter sniff) are NOT added — same pattern as the P1C-introduced gap; logged at `wallet-import-format-mismatch-matrix-completion` in `design/v0_28_0-cycle-followups.md` (updated with P2C reference).
- **Envelope field wire-shape:** `specter_source_metadata.devices` is emitted as an array of `{type, label}` objects (consistent across both legacy string-form and modern object-form inputs — the parser normalizes upstream, the envelope just renders).
- **Select-descriptor coerce:** orchestrator-specified per-format coerce for Specter (single-descriptor by construction); NOTICE template mirrors BSMS at line 543. Tested by `specter_select_descriptor_non_all_emits_notice_and_coerces` integration cell.
- **P0C dispatch test update:** `p0c_format_specter_panics_unimplemented` was renamed `p0c_format_specter_dispatches_format_mismatch_post_p2c` to reflect post-P2C semantic (mirrors P1C `_post_p1c` + P4C `_post_p4c` naming precedents).
