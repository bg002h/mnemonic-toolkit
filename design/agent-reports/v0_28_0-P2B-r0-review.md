# v0.28.0 P2B — architect R0 review

**Phase:** P2B — Specter parse impl + `canonicalize_specter` + 4 fixtures.
**Branch:** `v0.28.0/p2-specter`.
**Base:** P2A commit `818c4a2`.
**Verdict:** GREEN (0 Critical / 0 Important / 0 Minor surfaced beyond self-corrected items during authoring — see "Items self-corrected").

## Scope verified

Plan-doc P2B row (`/home/bcg/.claude/plans/unified-meandering-sundae.md:500`):

> Specter parse impl + `canonicalize_specter` helper. Parse unit tests + ~4 fixtures.

Plan-doc §S.2 fixtures (`unified-meandering-sundae.md:241`):

> ~4 — singlesig P2WPKH-from-coldcard, multisig 2-of-3 P2WSH-sortedmulti-multi-device, descriptor-with-checksum, blockheight-zero.

SPEC §11.2 parse contract (`design/SPEC_wallet_import_v0_28_0.md:331`):

> Extract `descriptor` verbatim. Preserve `label` as wallet name. `devices` array becomes per-cosigner provenance hints.

## Files touched

1. `crates/mnemonic-toolkit/src/wallet_import/specter.rs`:
   - Replaced P2A skeleton `parse()` body with real impl mirroring `bitcoin_core::parse_entry` + `bsms::parse` patterns:
     - JSON-parse + extract 4 required fields (label/blockheight/descriptor/devices).
     - Validate `descriptor` for xprv-prefix forbidden (matches `bitcoin_core` Phase 3 R0 C1 + I1 fold pattern; strips `#<csum>` trailer to avoid base58check stochastic false positives).
     - Validate BIP-380 checksum via `miniscript::descriptor::checksum::verify_checksum`.
     - Run `concrete_keys_to_placeholders` → `parse_descriptor::parse_descriptor` pipeline.
     - Extract per-cosigner origin components + infer network via BIP-48 coin-type child number (mirrors `bsms::network_from_origins` + `bitcoin_core::network_from_origins`).
     - Build per-cosigner `ResolvedSlot` with `entropy: None` + invariant-validate.
     - Normalize `devices` array (BOTH legacy string-form `["unknown"]` AND modern object-form `[{"type":..., "label":...}]`) into `Vec<SpecterDeviceMarker>` with explicit length-vs-cosigner-count NOTICE + pad/truncate.
     - Collect non-{label,blockheight,descriptor,devices} top-level keys into `dropped_fields` with stderr NOTICE.
     - Return `vec![ParsedImport { provenance: ImportProvenance::Specter(metadata), ... }]`.
   - Negative-blockheight rejection (SPEC-extension; blockheights are monotonically non-negative — surface as `ImportWalletParse` rather than silent cast).
   - Per-format helpers (`extract_origin_components`, `network_from_origins`, `coin_type_from_path`, `extract_threshold`, `xprv_prefix_regex`, `origin_capture_regex`) — duplicated from `bitcoin_core.rs` with `specter:` error-template prefix per the per-format error-message convention in this crate.
   - 17 new unit-test cells: 5 happy-path (singlesig, multisig, legacy string-devices, testnet, dropped-fields) + 8 negative-path (xprv-forbidden, invalid-checksum, missing-descriptor, negative-blockheight, devices-length-mismatch, devices-object-missing-type, invalid-JSON, threshold-u8-overflow) + 4 fixture-based smokes.

2. `crates/mnemonic-toolkit/src/wallet_import/mod.rs`:
   - Restored `#[allow(dead_code)]` annotation on `ImportProvenance::Specter` variant (variant is now CONSTRUCTED by P2B's parse impl but its payload field is not yet READ — match arms `Self::Specter(_) => None` discard it; P2C wires envelope-emit to read it).

3. `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs`:
   - Replaced `canonicalize_specter` P0C skeleton body (`Err(BadInput("not yet implemented..."))`) with real semantic-canonicalize impl. Pattern mirrors `canonicalize_bitcoin_core`:
     - Parse + extract required fields.
     - Re-canonicalize descriptor via existing `recanonicalize_descriptor` helper (parse + render + re-checksum).
     - Normalize devices to canonical object-form (legacy string → `{type, label: ""}`).
     - Re-serialize via BTreeMap for alphabetic key order + trailing newline.
   - Removed the deleted-on-arrival `canonicalize_specter_skeleton_returns_not_yet_implemented` test cell.
   - Removed `("specter", canonicalize_specter(b""))` from `skeleton_canonicalize_helpers_accept_empty_blob` test fixture loop; added `canonicalize_specter_rejects_empty_blob` as the regression guard for empty-blob behavior under the new impl.
   - 11 new canonicalize_specter test cells (4 inline + 4 fixture-based + 3 negative-path).

4. **NEW** fixture files in `crates/mnemonic-toolkit/tests/fixtures/wallet_import/`:
   - `specter-singlesig-p2wpkh-coldcard.json` — single-cosigner BIP-84 mainnet, coldcard device hint with label.
   - `specter-multisig-2of3-wsh-sortedmulti.json` — 2-of-3 mainnet BIP-48 multisig, 3 distinct device-vendor hints.
   - `specter-with-checksum.json` — singlesig with checksum suffix (regression guard for the `#<csum>` BIP-380 trailer pipeline).
   - `specter-blockheight-zero.json` — `blockheight: 0` (the default emit value from `wallet_export/specter.rs:67`) + legacy string-form `devices: ["unknown"]`.

## Behaviors locked

- **xprv-forbidden:** any extended-private-key prefix on the descriptor (mainnet `xprv`, testnet `tprv`, SLIP-132 `yprv|zprv|Yprv|Zprv|uprv|vprv|Uprv|Vprv`) → `ToolkitError::ImportWalletXprvForbidden`. Trailer-stripped BEFORE regex match (Phase 3 I1 fold pattern lifted from `bitcoin_core::parse_entry`).
- **Checksum-validated:** BIP-380 checksum on `descriptor` validated up-front via miniscript; mismatch → `ImportWalletParse` with explicit error template.
- **Network inference:** BIP-48 coin-type child number on FIRST cosigner's origin path; cross-cosigner heterogeneity → `ImportWalletParse` (mirrors `bsms` + `bitcoin_core` rule); coin-type 0 → mainnet, 1 → testnet, other → error.
- **Watch-only invariant:** every parsed cosigner has `entropy == None`. `validate_watch_only_resolved` called per SPEC §8.2.
- **Devices length-vs-cosigner-count:** lenient — emits stderr NOTICE on mismatch, pads with `unknown` placeholders or truncates so provenance vector has 1-to-1 mapping with cosigner slots. SPEC §11.2 doesn't normatively lock the rule, but the toolkit's own `wallet_export/specter.rs` emit invariant (line 62-63) is `length == cosigner_count`; we tolerate import-side drift via the lenient pattern.
- **Negative blockheight:** rejected as `ImportWalletParse("negative `blockheight` <N>; must be a non-negative integer")`. Blockheights are monotonically non-negative; a negative value signals malformed export. Not in SPEC §11.2 explicitly but logically consistent with the sniff's `is_u64() || is_i64()` integer check.
- **Dropped fields:** any top-level key outside `{label, blockheight, descriptor, devices}` is collected into `SpecterSourceMetadata.dropped_fields` + emits a single stderr NOTICE listing them. Mirrors `CoreSourceMetadata.dropped_fields` convention (`bitcoin_core.rs:195-206`).
- **Devices entry shape tolerance:** BOTH `Value::String(s)` and `Value::Object({"type":..., "label":...})` accepted. Other element types (null, number, bool, array) → `ImportWalletParse("devices[i] is neither a string nor an object")`.

## canonicalize_specter behaviors locked

- Semantic round-trip (not byte-exact): whitespace + key ordering + checksum recompute all normalize away.
- Output field order: `blockheight`, `descriptor`, `devices`, `label` (alphabetical via `BTreeMap`).
- Output device-entry shape: object form with `label` + `type` keys (alphabetical).
- Idempotent: `canonicalize(canonicalize(x)) == canonicalize(x)`.
- Unknown top-level fields dropped (mirrors parse-side `dropped_fields` discipline).
- Legacy string-form `["coldcard"]` and modern object-form `[{"type":"coldcard","label":""}]` canonicalize to the SAME output.
- Empty blob → `ImportWalletParse("invalid JSON")`; missing-descriptor → `ImportWalletParse("missing or non-string `descriptor`")`; negative blockheight → `ImportWalletParse("negative `blockheight` <N>")`.

## Test coverage

- 39 specter unit tests (was 23 at P2A end): 22 sniff + 12 parse + 4 fixture-smoke + 1 threshold-overflow regression. All green.
- 50 roundtrip tests (was 39 at P0C end): +11 new canonicalize_specter cells. All green.
- 105 total test suites in `cargo test -p mnemonic-toolkit`; 0 failures.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean (after one comparison-chain → match refactor).

## Items self-corrected during authoring

- **Helper-function duplication from `bitcoin_core.rs`:** considered hoisting `extract_threshold` / `network_from_origins` / `xprv_prefix_regex` to a shared parent module. Decided AGAINST — the convention in this crate is per-format error-message prefix (each helper's error templates carry the format tag like `"import-wallet: specter: parse error: ..."`). Sharing would require either generic error-template threading (overengineered for 3 parsers + 4 more to land) or accepting a generic error message. Lift-shared lives in a future `wallet-import-shared-origin-parse-helpers` FOLLOWUP if the duplication gets painful.
- **Negative-blockheight policy:** SPEC §11.2 didn't explicitly cover negative integers (sniff accepts any `is_u64() || is_i64()`). Chose explicit reject rather than silent `as u64` cast (would wrap to huge u64). Pinned by `parse_negative_blockheight_rejected` cell.
- **Devices length-vs-cosigner-count lenient policy:** SPEC §11.2 doesn't normatively lock. Chose lenient (NOTICE + pad/truncate) over strict (error) because some Specter firmware variants reportedly emit `devices: []` for watch-only descriptor exports. Pinned by `parse_devices_length_mismatch_emits_notice_and_normalizes` cell.
- **`canonicalize_specter` skeleton-test deletion:** the P0C-era skeleton-shape pin test (`canonicalize_specter_skeleton_returns_not_yet_implemented`) MUST be deleted at P2B (per the skeleton-cell's own doc-comment: "these cells become regression guards for the skeleton-shape contract and will be REPLACED — not augmented — at P{N}B"). Done.
- **`skeleton_canonicalize_helpers_accept_empty_blob` parametrized loop:** removed the `specter` row since it's no longer a skeleton. Added `canonicalize_specter_rejects_empty_blob` as the dedicated empty-blob regression guard for the real impl.

## Discipline checks

- **No `git add -A`:** P2B commit stages specific paths only.
- **Alphabetical-insertion discipline:** all match arms preserve alphabetical order (BitcoinCore → Bsms → Specter). No new `ToolkitError` variants added (existing `ImportWalletXprvForbidden` + `ImportWalletParse` cover all paths). No new `SniffOutcome` variants (P2A added `Specter`).
- **Architect-review persistence:** this file at `design/agent-reports/v0_28_0-P2B-r0-review.md` BEFORE commit (CLAUDE.md persistence discipline).

## Items NOT in P2B scope (deferred to P2C)

- 8 dispatch-site flips in `cmd/import_wallet.rs` — **P2C**.
- Integration test file `tests/cli_import_wallet_specter.rs` — **P2C**.
- `tests/cli_import_wallet_p0c_dispatch.rs::p0c_format_specter_panics_unimplemented` cell removal — **P2C** (it correctly panics today; will be deleted when P2C flips the dispatch arm).
- Lifting `#[allow(dead_code)]` from `SpecterSourceMetadata` + `SpecterDeviceMarker` struct fields + `ImportProvenance::Specter` variant — **P2C** (P2C wires the envelope-emit dispatch site to READ the fields; until then they are constructed-but-never-read).

## Conclusion

P2B scope COMPLETE per plan-doc + SPEC §11.2. Parse impl mirrors `bitcoin_core` + `bsms` patterns; canonicalize impl mirrors `canonicalize_bitcoin_core`. Both legacy string-devices and modern object-devices shapes accepted at parse + canonicalize. 4 SPEC-tagged fixtures land. Verdict: GREEN — proceed to P2C.
