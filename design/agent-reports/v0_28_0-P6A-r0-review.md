# v0.28.0 Phase P6A — Architect Review R0

**Phase:** P6A — `wallet_import/electrum.rs` skeleton + sniff + `ElectrumSourceMetadata` + sniff-bool wiring.

**Date:** 2026-05-19.

**Reviewer:** opus (acting as architect within autonomous execution; equivalent to per-phase architect-review-agent dispatch per cycle convention).

**Scope of review:**

- SPEC patch to §11.6 (wallet_type value-set correction).
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (NEW, 357 lines including 28 tests).
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (+1 line: `pub(crate) mod electrum;`).
- `crates/mnemonic-toolkit/src/wallet_import/sniff.rs` (+1 use line, +1 sniff-bool flip).

**Test surface:** 28 new unit tests in `wallet_import::electrum::tests` (all pass). 11 existing `wallet_import::sniff::tests` unchanged + still pass. Full 511-test `--bin mnemonic` suite passes.

---

## Verdict: GREEN (0 Critical / 0 Important / 1 Minor / 2 Notes)

P6A delivers what the phase brief asks for and uncovered one SPEC accuracy defect during recon (folded inline as a SPEC patch). Phase P6B is unblocked.

---

## Critical findings

(none)

---

## Important findings

(none)

---

## Minor findings

### M1 — `sniff` over-fires on `seed_version: 0` if regex-multisig pattern with leading zero

`sniff_no_match_multisig_pattern_zero_k` test pins that `"0of3"` is **sniff-positive** (regex matches `\d+`) with rationale "Per Electrum semantics k >= 1; ... bounds check is deferred to parse-time." This is technically correct (the regex is faithful to Electrum's `multisig_type` upstream which uses `\d+`), but it means a contrived blob like `{"seed_version":11,"wallet_type":"0of0"}` sniffs as Electrum and routes to the P6B parser, which will eventually need to error.

**Impact:** Cosmetic only. P6B will refuse `(k, n)` where `k == 0 || n == 0 || k > n` as part of its multisig-shape validation. The Ambiguous gate is preserved (the blob doesn't match any other parser's sniff either, so the parser-dispatch routing is correct).

**Recommendation:** Document in P6B parse-arm via a SPEC §11.6 sentence and add a parse-time test cell. No P6A action required.

---

## Notes

### N1 — SPEC §11.6 wallet_type value-set is corrected from the P0A draft

The P0A SPEC draft listed `wallet_type ∈ {"standard", "multisig", "2fa", "imported"}`. P6A recon via WebFetch against `electrum/wallet_db.py` + `electrum/util.py::multisig_type` proved that the literal string `"multisig"` is **never** stored as `wallet_type` by Electrum 4.x — the canonical multisig value is the `<k>of<n>` regex pattern (e.g., `"2of4"`). The toolkit's own `wallet_export/electrum.rs` confirms this: `electrum_multi_2of4.json` emits `"wallet_type": "2of4"`.

This is a P6A SPEC patch (not a P0A regression — the P0A R0 architect review focused on enum scaffolding, not Electrum-specific format research). The corrected enumeration mirrors the WebFetch-verified upstream behavior + the toolkit's own emit surface.

The `ElectrumWalletType::Multisig { k: u8, n: u8 }` provenance struct carries the parsed `(k, n)` directly rather than a static `Multisig` discriminator, so the on-disk representation round-trips faithfully through Phase P6C's `--json` envelope (consumed by P6B canonicalize + P6C dispatch).

### N2 — `seed_version` widened from u8 → u32

P0A SPEC draft pinned `seed_version: u8`. P6A widens to `u32` for two reasons: (a) consistency with `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN: u32`; (b) FOLLOWUP `electrum-final-seed-version-drift` tracks upstream's `FINAL_SEED_VERSION` rising — if it ever crosses 255 the u8 form would require a field-type churn. u32 absorbs that without code surgery. Documented in the SPEC patch.

---

## P6A → P6C interaction window (informational, not a finding)

Wiring `electrum = ElectrumParser::sniff(blob)` at `wallet_import/sniff.rs:79` means `sniff_format` can now return `SniffOutcome::Electrum`. The dispatch site at `cmd/import_wallet.rs:325` has an `other => unreachable!(...)` catch-all that assumes the new variants cannot fire. If a user supplied an Electrum blob between landing P6A and P6C **with no `--format` flag**, the `unreachable!` would panic.

This is INTRA-PR: P6A and P6C land in the same PR to `release/v0.28.0`, so the user-facing surface jumps atomically from "no electrum" to "full electrum" at PR merge. The intra-commit panic surface is non-user-reachable because no integration test in this PR exercises auto-sniff against an Electrum blob between commits. **P6C MUST replace the `unreachable!` arm with an explicit `SniffOutcome::Electrum => "electrum"` mapping; this is the brief's "Add `SniffOutcome::Electrum => \"electrum\"` arm" requirement.**

The `cli_import_wallet_p0c_dispatch.rs::p0c_format_electrum_panics_unimplemented` test still passes because it supplies `--format electrum` explicitly (routing through Site 2, which still panics via `unimplemented!()`). That test will be replaced/relaxed in P6C when the per-format dispatch arm is wired to call `ElectrumParser::parse`.

---

## Verification trail

- Built with `cargo build -p mnemonic-toolkit`: clean (after `#[allow(dead_code)]` annotations on P6B-consumed surfaces).
- Ran `cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::electrum`: 28 / 28 pass.
- Ran `cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::sniff`: 11 / 11 pass (existing tests unchanged + regression-clean under new electrum-bool wire).
- Ran full `cargo test -p mnemonic-toolkit --bin mnemonic`: 511 / 511 pass.

---

## Files changed by P6A

- `design/SPEC_wallet_import_v0_28_0.md`: §11.6 patch (wallet_type value-set + provenance struct correction).
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs`: NEW.
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs`: +1 line (`pub(crate) mod electrum;` insert alphabetically between `bsms_verify` and `json_envelope`).
- `crates/mnemonic-toolkit/src/wallet_import/sniff.rs`: +1 use, +1 sniff-bool flip.

End of P6A R0 review.
