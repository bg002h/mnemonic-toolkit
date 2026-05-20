# v0.28.0 Phase P6C — Architect Review R0

**Phase:** P6C — flip 8 dispatch sites + `ImportProvenance::Electrum` variant + `SniffOutcome::Electrum => "electrum"` arm + integration cells + Site 8 stderr-warning electrum opt-in.

**Date:** 2026-05-19.

**Reviewer:** opus (acting as architect within autonomous execution).

**Scope of review:**

- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — `ImportProvenance::Electrum` variant added alphabetically (BitcoinCore < Bsms < Electrum); new `electrum_metadata()` accessor; `bsms_audit()` + `source_metadata()` extended with `Electrum` arm. Provenance test matrix extended.
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` — P6B placeholder `ImportProvenance::Bsms(None)` replaced with real `ImportProvenance::Electrum(ElectrumSourceMetadata)`; `#[allow(dead_code)]` annotations removed.
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — Sites 2/4 (electrum dispatch + parser call), Site 3 (auto-sniff arm), Site 7 (`--json` envelope roundtrip arm), Site 8 (stderr-warning electrum opt-in), source_metadata envelope block for electrum. Multi-arm format-mismatch dispatch for `--format electrum` against all other concrete sniff verdicts.
- `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum.rs` — NEW (14 integration cells).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs` — `p0c_format_electrum_panics_unimplemented` replaced with `p6c_format_electrum_no_longer_panics_unimplemented` regression guard.

**Test surface:** 14 new integration tests in `cli_import_wallet_electrum.rs` (3 happy paths + 2 auto-sniff + 4 refusals + 3 envelope shape + 2 format-mismatch). 1 P0C test updated. Total bin tests now 530; integration suite green. Full toolkit suite passes.

---

## Verdict: GREEN (0 Critical / 0 Important / 1 Minor / 3 Notes)

P6C delivers all 8 dispatch-site flips + the integration surface. Electrum ingest is end-to-end functional: explicit `--format electrum`, auto-sniff routing, refusal templates, `--json` envelope with source_metadata, format-mismatch surface. Phase 6 cycle is closed.

---

## Critical findings

(none)

---

## Important findings

(none)

---

## Minor findings

### M1 — Site 8 emit_roundtrip_stderr_warning regression-pinned test text-format change

The pre-P6C Site 8 used a hardcoded `canonicalize_bitcoin_core failed:` template. My initial P6C refactor used `canonicalize_{format_str} failed:` which renders `canonicalize_bitcoin-core` (with hyphen, since `format_str` is `bitcoin-core`). This broke two pinned regression tests (`emit_roundtrip_stderr_warning_canonicalize_err_emits_warning`, `emit_roundtrip_stderr_warning_non_utf8_blob_emits_notice`).

Folded same-session: refactored to use a `canonicalize_label: &'static str` arm-bound to the canonical function name (`canonicalize_bitcoin_core` / `canonicalize_electrum`). Both tests pass; no user-facing text change.

**Lesson:** when extending an existing Site 8-style format-discrimination block, the `canonicalize_<format>` Rust function name (snake_case) is distinct from the CLI `--format <value>` (kebab-case). Future P{N}C sub-phases must replicate the `arm-bound canonicalize_label` pattern (don't string-template the format_str).

---

## Notes

### N1 — All 8 dispatch sites correctly flipped per plan-doc §B.2 #6

- **Site 1** (clap PossibleValuesParser): unchanged — `electrum` was already in the alphabetical set from P0C.
- **Site 2** (`Some("electrum") =>` arm in format_str match): full mismatch-discrimination block replaces `unimplemented!()`. All concrete sniff verdicts produce ImportWalletFormatMismatch with both supplied + sniffed labels.
- **Site 3** (auto-sniff `None =>` arm): new `SniffOutcome::Electrum => "electrum"` arm added; the existing `unreachable!` catch-all for not-yet-wired variants is preserved (statically reachable only for Coldcard/Sparrow/Specter/Jade/ColdcardMultisig — all still bool-false at P6C).
- **Site 4** (format_str → parser dispatch): `"electrum" => crate::wallet_import::electrum::ElectrumParser::parse(...)?` replaces the `unimplemented!()`.
- **Site 5** (select-descriptor coercer): unchanged — electrum singlesig + multisig produce 1 bundle each; default `_ => apply_select_descriptor` path is correct.
- **Site 6** (canonicalize dispatch): unchanged — was already wired to real `canonicalize_electrum` at P0C; the P6B fold replaced the skeleton body. No P6C change required.
- **Site 7** (`--json` envelope roundtrip arm): full byte_exact/semantic_match/diff/status block (mirrors bitcoin-core arm) replaces `json!({})`.
- **Site 8** (stderr-warning predicate): `if !matches!(format_str, "bitcoin-core" | "electrum")` replaces `if format_str != "bitcoin-core"`; canonicalize dispatch switched to arm-bound function selection.

### N2 — ImportProvenance::Electrum source_metadata block enumerates electrum-specific fields

`emit_json_envelope` now emits one of two `source_metadata` blocks (mutually exclusive per the enum invariant):
- Bitcoin Core: `{active, internal, range, dropped_fields, wallet_name}` (unchanged).
- Electrum: `{seed_version, wallet_type, wallet_name, dropped_fields}` (NEW; wallet_type renders as `"standard"` or `"<k>of<n>"` mirroring on-disk).

The two blocks intentionally have overlapping but disjoint field-shape (both have `wallet_name` + `dropped_fields`; bitcoin-core has `active/internal/range` not on electrum; electrum has `seed_version/wallet_type` not on bitcoin-core). Downstream consumers discriminate via the outer `source_format` field.

BSMS bundles (which use ImportProvenance::Bsms) have NO `source_metadata` block — the BSMS audit fields live under the separate `bsms_audit` key. This asymmetry is pre-existing v0.27.x behavior, unchanged by P6C.

### N3 — P6A → P6C panic window closed

The P6A R0 review flagged a P6A → P6C interaction window: with `electrum = ElectrumParser::sniff(blob)` wired at P6A but the `cmd/import_wallet.rs:325` `unreachable!` catch-all not yet updated, a user supplying an Electrum blob WITHOUT `--format` would have panicked.

P6C closes the window by adding `SniffOutcome::Electrum => "electrum"` BEFORE the catch-all. The auto-sniff path now routes Electrum verdicts correctly; the catch-all remains the structural guard for the still-bool-false variants (Coldcard/Sparrow/etc.). All cli_import_wallet_electrum.rs cells that exercise auto-sniff pass without invoking the catch-all.

---

## Verification trail

- `cargo build -p mnemonic-toolkit`: clean.
- `cargo test -p mnemonic-toolkit --test cli_import_wallet_electrum`: 14 / 14 pass.
- `cargo test -p mnemonic-toolkit --test cli_import_wallet_p0c_dispatch`: 10 / 10 pass (1 updated test).
- `cargo test -p mnemonic-toolkit --bin mnemonic`: 530 / 530 pass.
- `cargo test -p mnemonic-toolkit`: full integration suite green.
- Smoke tests: `cargo run -- import-wallet --format electrum --blob <fixture>` produces the expected text-summary output for all 3 happy-path fixtures + the 3 refusal templates fire byte-exactly per SPEC §11.6.1.

---

## Files changed by P6C

- `crates/mnemonic-toolkit/src/wallet_import/mod.rs`: ImportProvenance::Electrum variant + electrum_metadata() accessor + provenance test matrix extension (4 new cells in the matrix-invariant test).
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs`: ImportProvenance fold replaces P6B placeholder; #[allow(dead_code)] removed.
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`: Sites 2, 3, 4, 7, 8 flipped; source_metadata envelope block added for electrum; arm-bound canonicalize_label fix (M1).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum.rs`: NEW (14 integration cells).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs`: 1 cell renamed + body updated to regression-guard the unimplemented-panic removal.

End of P6C R0 review.

---

## Phase 6 cycle close

All 3 sub-phases shipped:
- P6A R0: GREEN (skeleton + sniff + SPEC patch).
- P6B R0: GREEN (real parser + canonicalize + 6 fixtures).
- P6C R0: GREEN (8 dispatch sites + provenance + integration cells).

Total P6 surface:
- 1 new module (`wallet_import/electrum.rs`, ~890 LOC including 44 unit tests).
- 6 new fixtures.
- 1 new integration test file (`cli_import_wallet_electrum.rs`, 14 cells).
- 1 ImportProvenance enum variant (Electrum) + 1 accessor (electrum_metadata).
- 1 canonicalize_electrum real body.
- 8 dispatch site flips in cmd/import_wallet.rs.
- 3 architect reviews persisted to design/agent-reports/.
- 1 SPEC patch (§11.6 wallet_type value-set correction + provenance struct).

Test surface delta: +14 integration tests + +49 unit tests = +63 tests net. Full toolkit suite green.
