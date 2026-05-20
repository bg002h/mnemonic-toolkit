# v0.28.0 P2B-v2 — R0 architect review

**Branch:** `v0.28.0/p2-specter-v2-bg`
**Base:** P2A commit `6e22208`
**Scope:** Specter parse impl + `canonicalize_specter` (real body in `roundtrip.rs`, replacing skeleton) + 4 fixtures.

## Verdict

GREEN — 0 Critical / 0 Important / 0 Minor.

## Critical

(none)

## Important

(none — R0 caught one issue mid-implementation:)

### R0-fold: BIP-380 checksum strip required pre-pipeline

**Issue:** Initial parse() impl fed `descriptor_str` directly to
`concrete_keys_to_placeholders`. The downstream `parse_descriptor` ran
`MsDescriptor::from_str` on the placeholder form which rejected the
original `#csum` (correct for the concrete-keys form, invalid for the
placeholder form). 12 of 12 happy-path cells failed with "expected
77xs5dg6" / similar.

**Fold:** Added an explicit `miniscript::descriptor::checksum::verify_checksum`
call BEFORE `concrete_keys_to_placeholders`, mirroring the BSMS pattern
at `wallet_import/bsms.rs:195-207`. The returned body sans `#csum` feeds
the pipeline; the placeholder form has no original-checksum baggage and
miniscript re-validates the canonical post-placeholder form on its own
inside `parse_descriptor`. After fold, all 12 happy-path cells passed.

## Minor

(none)

## Verification

- `cargo build -p mnemonic-toolkit`: clean
- `cargo clippy -p mnemonic-toolkit --all-targets`: clean
- 45 specter unit tests pass (24 sniff from P2A + 21 new parse/fixture cells)
- 55 roundtrip tests pass (50 prior + 5 new canonicalize_specter cells)
- No regressions across the rest of the toolkit

## Notes

- Parse contract: Specter's `descriptor` field is the FULL concrete-keys form
  (`wpkh([fp/path]xpub/<0;1>/*)`, etc.) — NOT the `@N/**` placeholder form
  Sparrow uses. So no substitution needed; the parse reduces to feeding the
  descriptor through the BSMS+BitcoinCore pipeline (post BIP-380 checksum strip).
- `devices` array supports BOTH shapes:
  - Modern object form `{"type": "<vendor>", "label": "<name>"}` — both fields extracted
  - Legacy string form `"<vendor>"` (toolkit's own `wallet_export/specter.rs:55` emitter shape) — normalized to `{type: <vendor>, label: ""}`
  - Anything else (number, array, null) → ImportWalletParse with explicit error
- `canonicalize_specter` is BTreeMap-backed (alphabetical top-level keys); preserves the 4 load-bearing fields verbatim (no nested key reordering since `descriptor` is opaque string + `devices` is content-preserving).
- Threshold extraction regex extended over Sparrow's to include `multi_a` + `sortedmulti_a` (taproot multi variants); Specter exports taproot single-sig directly (descriptor-passthrough works because the concrete `[fp/path]xpub` form is already in the wire shape).
- 4 fixtures committed:
  - `specter-singlesig-p2wpkh.json` — minimal singlesig
  - `specter-multisig-2of3-p2wsh-sortedmulti.json` — 2-of-3 multisig 3 cosigners
  - `specter-blockheight-zero.json` — `blockheight: 0` + legacy `devices: ["unknown"]` shape
  - `specter-descriptor-with-checksum.json` — descriptor with `#csum` suffix + object-form device with empty label
- The `parse_descriptor_without_origin_refused` test uses a valid BIP-380 checksum (`nczup5a0`) so the failure surfaces at the origin-extraction step (not the checksum verify), proving the no-keys error path works.
