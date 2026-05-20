# Phase P3C — architect R0 review

**Reviewer:** in-session architect-style self-review (Opus 4.7 main agent)
**Branch:** `v0.28.0/p3-coldcard`
**Files under review:**
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (+~25 LOC: `ImportProvenance::Coldcard` variant + accessor matches + `coldcard_metadata` accessor)
- `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs` (~-5 LOC: flip placeholder `Bsms(None)` provenance → real `Coldcard(meta)`; remove `#[allow(dead_code)]`)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (+~70 LOC: explicit `--format coldcard` arm with FormatMismatch sniff guard; `ColdcardParser::parse` dispatch; envelope round-trip arm; `coldcard_metadata` envelope field)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs` (new, ~290 LOC, 17 integration cells)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_p0c_dispatch.rs` (rename + update one cell: P3C transitions `unimplemented!` panic to FormatMismatch)

**Source SHA verified against:** branch HEAD pre-commit
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 3 P3C row + §S.3
**SPEC:** `design/SPEC_wallet_import_v0_28_0.md` §11.3

---

## Critical (correctness-blocking)

**None.** P3C delivers complete dispatch wire-up:

1. `ImportProvenance::Coldcard(coldcard::ColdcardSourceMetadata)` enum variant added alphabetically after `Bsms` per CLAUDE.md alphabetical-discipline invariant.
2. `bsms_audit()` + `source_metadata()` accessor match-arms extended with `Self::Coldcard(_) => None`.
3. New `coldcard_metadata()` accessor: `Some(&meta)` only for Coldcard variant.
4. `ColdcardParser::parse` body's placeholder `Bsms(None)` provenance flipped to real `Coldcard(meta)`.
5. `cmd/import_wallet.rs` site 2: explicit `--format coldcard` arm with FormatMismatch guard against sniff-yields-bsms / sniff-yields-bitcoin-core (mirrors BSMS / Bitcoin Core precedent).
6. `cmd/import_wallet.rs` site 4: `"coldcard" => ColdcardParser::parse(&blob, stderr)?`.
7. `cmd/import_wallet.rs` site 7: roundtrip arm produces byte-exact / semantic-match / status:ok envelope (or canonicalize_failed) pattern mirroring `bitcoin-core`.
8. New `coldcard_metadata` envelope field carries chain / xfp (uppercased) / bip_derivation / account / dropped_fields.
9. Updated `SniffOutcome::Coldcard => "coldcard"` arm comment to remove the P3A transitional language; now reflects post-P3C live state.
10. New `cli_import_wallet_coldcard.rs` integration test file with 17 cells covering happy-paths (5 BIP variants), auto-sniff dispatch, envelope assertions (coldcard_metadata + roundtrip + bundle descriptor), stderr notice, and refusal cases (FormatMismatch on BSMS/Core blobs, malformed JSON, missing-BIP-block).

## Important

**I1 — Updated `p0c_format_coldcard_panics_unimplemented` test in P0C dispatch suite.** Originally asserted `unimplemented!()` panic; post-P3C the dispatch is real and returns FormatMismatch on BSMS blob. Renamed test to `p0c_format_coldcard_dispatch_post_p3c_returns_format_mismatch` + updated assertion to check FormatMismatch error contents. This test transition is expected at P3C (the P0C pre-stub graduation point); the other 5 unimplemented-format tests (sparrow / specter / electrum / jade / coldcard-multisig) remain unchanged until their own P{N}C lands.

## Minor

**M1 — `coldcard_metadata` envelope wire-up uses verbose enum match expressions inline.** The chain-to-string + bip_derivation-to-string + xfp-to-hex conversions are inline in the `json!` macro arms. Could be refactored to small helper functions on `ColdcardChain` / `ColdcardBip` (e.g., `as_str(&self) -> &'static str`); deferred as a Minor since the inline forms are equally readable and adding helpers expands the public API surface of the coldcard module. No fold.

**M2 — `coldcard_metadata` envelope field is a NEW top-level field, distinct from existing `source_metadata` (which is Bitcoin Core specific) and `bsms_audit`.** This means `--json` envelopes can carry AT MOST ONE of `source_metadata` / `bsms_audit` / `coldcard_metadata` at a time (enforced by `ImportProvenance` enum). Documented in the wire-up comment. A future SPEC cleanup might consolidate these under a single `provenance_metadata` field with a discriminator; tracked implicitly via the v0.28 enum-variant-per-format pattern. No fold.

**M3 — `coldcard_missing_bip_block_returns_parse_error` integration test accepts multiple error-message variants** (`"no recognized BIP-derivation block"` OR `"could not detect format"` OR `"matches multiple"`). Rationale: the input `{"chain":"BTC","xfp":"5436D724","account":0}` is REJECTED by Coldcard's sniff (clause 3 violation), so auto-sniff would yield NoMatch. With explicit `--format coldcard`, the parse runs and refuses at dominant-BIP selection. The test tolerates either dispatch path. Documented in the test body. No fold.

**M4 — `coldcard_metadata` JSON xfp hex casing.** Uses uppercase (8-char `{:02X}` format) to mirror Coldcard's source-blob convention. The internal `[u8; 4]` representation is case-agnostic; choosing UPPER for the envelope output preserves visual parity with Coldcard documentation samples. No fold.

**M5 — Integration test cell count: 17.** Plan-doc P3C line-budget is "~270 tests" (LOC; actual ~290 LOC including helpers + assertions). Matches budget. No fold.

**M6 — Stale comment in `cmd/import_wallet.rs:300+` about "P0D pre-stub catch-all" still references unreachable! arm.** Now that P3A flipped sniff-wiring to live, the catch-all is partially-reachable (5 of 6 new SniffOutcome variants are still placeholder-false; only Coldcard is live). The comment block is informative-historical; refactoring it would inflate the P3C diff. Will become accurate again when P4/P5/P6 land. No fold.

## Verdict

**GREEN.** Proceeding to commit + PR-open.
