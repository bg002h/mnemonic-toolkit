# v0.28.0 Phase P6B — Architect Review R0

**Phase:** P6B — per-wallet_type parser body + `canonicalize_electrum` real body + 6 fixtures.

**Date:** 2026-05-19.

**Reviewer:** opus (acting as architect within autonomous execution).

**Scope of review:**

- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` — parse() body replaces P6A skeleton (~ +380 lines including 7 new tests).
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs::canonicalize_electrum` — skeleton replaced with real body; 4 new tests + 1 updated skeleton-list test.
- 6 NEW fixtures under `crates/mnemonic-toolkit/tests/fixtures/wallet_import/`:
  - `electrum-standard-bip84-mainnet.json` (zpub singlesig)
  - `electrum-standard-bip49-mainnet.json` (ypub singlesig → sh(wpkh))
  - `electrum-multisig-2of3-wsh.json` (Zpub triplet → wsh(sortedmulti))
  - `electrum-2fa-refused.json`
  - `electrum-imported-refused.json`
  - `electrum-encrypted-refused.json`

**Test surface:** 16 NEW unit tests added (10 parse cells + 6 fixture round-trips); total electrum module tests now 44. 4 NEW canonicalize cells in roundtrip module + 1 updated test. 525 total bin tests pass; full toolkit suite green.

---

## Verdict: GREEN (0 Critical / 0 Important / 2 Minor / 2 Notes)

P6B delivers the brief's required per-wallet_type parser + canonicalize + 6 fixtures. The parser correctly handles standard (BIP-44 / BIP-49 / BIP-84 / BIP-86) + multisig (wsh / sh(wsh) / sh) + 2fa / imported / encrypted refusals. P6C is unblocked.

---

## Critical findings

(none)

---

## Important findings

(none)

---

## Minor findings

### M1 — Singlesig BIP-86 (tr) path is wired but no fixture exercises it

`singlesig_wrapper_from_variant_and_purpose` handles `(None, 86)` → `SinglesigWrapper::Tr`, and `wrap_singlesig` renders `tr(<inner>)`. No fixture exercises the BIP-86 path. The wallet_export side has no BIP-86 Electrum fixture either (Electrum 4.x's taproot singlesig support is post-FINAL_SEED_VERSION).

**Impact:** Cosmetic — the path is reachable via the inverse of the documented neutral xpub + purpose=86 combination, but no real-world Electrum wallet currently uses it. Documented in the wrapper function's doc-comment.

**Recommendation:** Defer to a v0.28+ FOLLOWUP if Electrum ships taproot singlesig support; OR add a single hand-crafted neutral-xpub fixture in P6C. No P6B action required.

### M2 — `ImportProvenance::Electrum` placeholder fold

P6B emits `ImportProvenance::Bsms(None)` as a placeholder because the `Electrum(ElectrumSourceMetadata)` variant doesn't exist on `ImportProvenance` until P6C lands. The let-binding `_placeholder_provenance_inputs = (seed_version, &wt_enum, &dropped_fields)` preserves the captured fields so P6C's diff is a single-arm replacement.

**Impact:** None at runtime (the placeholder is structurally identical to `BsmsParser::parse`'s 2-line excerpt return, which carries no audit). The CLI dispatch site at `cmd/import_wallet.rs` doesn't currently reach the electrum parser anyway (P0C `unimplemented!()` arm fires first). This is a P6B → P6C handoff convention, not a user-visible defect.

**Recommendation:** P6C MUST replace the placeholder + populate the real metadata; if it doesn't, the envelope's `source_metadata` field for electrum imports would be `None` (silently degenerate). The let-binding's name makes the placeholder discoverable via grep.

---

## Notes

### N1 — Canonicalize is BTreeMap-stable + drops 13 runtime-state fields

`canonicalize_electrum` re-emits via `BTreeMap<String, Value>` for byte-stable output (alphabetical key order). The dropped-field list mirrors `wallet_import::electrum::collect_dropped_fields` exactly (13 names: `addr_history`, `addresses`, `channels`, `channel_backups`, `fiat_value`, `labels`, `spent_outpoints`, `stored_height`, `transactions`, `tx_fees`, `txi`, `txo`, `verified_tx3`). Two callers, single source-of-truth list — drift between the parser's stderr NOTICE and the canonicalize's silent drop would surface as an integration test failure at P6C's `--json` envelope round-trip cells.

If future Electrum versions add new runtime-state fields not in this list, the canonicalize will preserve them (not drop), and the next round-trip comparison will be `byte_exact=false`. That's the correct fail-safe behavior; the list grows as new fields are discovered.

### N2 — Multisig path supports wsh / sh(wsh) / sh wrappers via SLIP-132 prefix discrimination

`multisig_wrapper_from_variant` maps:
- `Zpub` / `Vpub` → `wsh(sortedmulti)` (BIP-48 wsh native segwit)
- `Ypub` / `Upub` → `sh(wsh(sortedmulti))` (BIP-48 wrapped segwit)
- `xpub` / `tpub` → `sh(sortedmulti)` (legacy BIP-45 / hand-edited)

Heterogeneous cosigner variants are rejected with a `must share a variant` template (test pinned). This mirrors `wallet_export/electrum.rs::emit_electrum_multisig_json`'s uniform-variant emit behavior (Electrum 4.x emits a single SLIP-132 variant for all cosigners; mixed-prefix multisig is unrepresentable upstream).

P2tr-multi is NOT supported (Electrum 4.x lacks libsecp-taproot support per the export-side refusal at `wallet_export/electrum.rs:60`; the ingest side never reaches that path because Electrum can't emit it).

---

## Verification trail

- `cargo build -p mnemonic-toolkit`: clean.
- `cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::electrum`: 44 / 44 pass.
- `cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::roundtrip`: pass (4 new electrum cells + 1 updated skeleton-list cell).
- `cargo test -p mnemonic-toolkit --bin mnemonic`: 525 / 525 pass.
- `cargo test -p mnemonic-toolkit`: full suite green (incl. integration tests).

---

## Files changed by P6B

- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs`: parse() body replaces P6A skeleton; +380 LOC including 16 new tests (10 parse cells + 6 fixture round-trips).
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs`: `canonicalize_electrum` replaces skeleton with real body; 4 new tests + 1 updated `skeleton_canonicalize_helpers_accept_empty_blob` (electrum removed from the skeleton list) + 1 new empty-blob shape test.
- 6 NEW fixtures under `crates/mnemonic-toolkit/tests/fixtures/wallet_import/`.

End of P6B R0 review.
