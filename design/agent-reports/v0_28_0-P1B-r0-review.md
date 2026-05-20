# v0.28.0 — Phase P1B (Sparrow parse impl + canonicalize_sparrow + fixtures) — R0 architect review

**Branch:** `v0.28.0/p1-sparrow-v2`
**Base:** `release/v0.28.0` @ `d7a2859` (post-Wave-1-cascade)
**Scope:** Sparrow parse impl in `wallet_import/sparrow.rs` (replacing the P1A `unimplemented!("P1B")` body) + `canonicalize_sparrow` real impl in `wallet_import/roundtrip.rs` (replacing the skeleton) + 5 on-disk fixtures.
**Date:** 2026-05-19
**Reviewer:** self-architect-review (R0)

## Status: GREEN

## Methodology

Plan-doc anchor: `/home/bcg/.claude/plans/unified-meandering-sundae.md:492` —
> **P1B** | Sparrow parse impl: decode `keystores[]`, extract `defaultPolicy.miniscript.script`, populate `SparrowSourceMetadata`. PLUS new `canonicalize_sparrow` helper in `wallet_import/roundtrip.rs`. Parse unit tests + ~5 fixtures. | `wallet_import/sparrow.rs`, `wallet_import/roundtrip.rs`, `tests/fixtures/wallet_import/sparrow-*.json` (×5) | ~180 src + ~250 tests + 5 fixture files

SPEC §11.1: parse contract steps lockstepped with the implementation.

## Source-grep verification (current branch HEAD)

### Parse impl — 10-step SPEC §11.1 walk

`crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (210 LOC parse impl + ~200 LOC tests + ~150 LOC helpers):

1. **Step 1 (JSON parse + object check):** lines 212-228.
2. **Step 2 (envelope extraction):** lines 230-279. Extracts `name`, `policyType`, `scriptType`, `defaultPolicy.miniscript.script`, `keystores[]`.
3. **Step 3 (policyType ↔ N consistency):** lines 281-296. SINGLE ⇒ N=1; MULTI ⇒ N≥2.
4. **Step 4 (per-keystore parse):** lines 298-302 + `parse_keystore` helper. Extracts `keyDerivation.masterFingerprint`, `keyDerivation.derivation`, `extendedPublicKey`. Master fingerprint format-validated (8 hex chars).
5. **Step 6 early (taproot refusal):** lines 304-314. `script_template.contains("tr(")` short-circuits with explicit `ImportWalletParse("taproot scripts are not yet supported ...")`. Cycle-followup `sparrow-taproot-descriptor-passthrough-import-support` filed.
6. **Step 5 (`@i/**` substitution):** lines 316-343. Longest-first iteration (sort by `i.to_string().len()` Reverse) prevents `@1` colliding with `@10`-prefix. Stray-`@N` leftover regex acts as sanity guard.
7. **Step 7 (pipeline):** lines 356-376. Feeds through `concrete_keys_to_placeholders` + `parse_descriptor::parse_descriptor`. Error-text prefix rewriting transforms `bsms:` → `sparrow:` for parity.
8. **Step 8 (ResolvedSlot vec):** lines 378-395. Uses `extract_origin_components` + `network_from_origins` + `build_slot_fields`. `validate_watch_only_resolved` invariant enforced.
9. **Threshold extraction:** lines 397-399. Local `extract_threshold_local` regex matches `multi|sortedmulti|thresh\(K,`.
10. **Step 9 (dropped-fields NOTICE):** lines 401-416. Per SPEC §2.4 stderr template.
11. **Step 10 (ParsedImport wrap):** lines 418-446. `ImportProvenance::Sparrow(SparrowSourceMetadata { ... })` constructor + `original_descriptor` via `recompute_descriptor_checksum`.

### canonicalize_sparrow

`crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` (lines 410-589, ~165 LOC):

- BTreeMap-backed alphabetical key ordering at serialize time.
- Preserved top-level keys: `name`, `network`, `policyType`, `scriptType`, `defaultPolicy`, `keystores`. Other keys (birthDate, gapLimit, mixConfig, ...) DROPPED.
- `defaultPolicy.miniscript.script` preserved verbatim; default `name: "Default"` if absent.
- `keystores[].extendedPublicKey` SLIP-132-normalized (best-effort; verbatim fallback on normalize failure).
- Trailing `\n` appended for pretty-printer parity.

### Fixtures (5 files, all under `tests/fixtures/wallet_import/`)

1. `sparrow-singlesig-p2wpkh.json` — minimal SINGLE P2WPKH (bip84/0'/0'/0' xpub6Bner... `5436d724`).
2. `sparrow-multisig-2of3-p2wsh-sortedmulti.json` — MULTI 2-of-3 P2WSH sortedmulti (3 cosigners, BIP-48 paths).
3. `sparrow-multisig-2of3-p2wsh-multi-ordered.json` — MULTI 2-of-3 P2WSH ordered `multi(...)` variant.
4. `sparrow-singlesig-p2sh-p2wpkh.json` — SINGLE bip49 P2SH-P2WPKH.
5. `sparrow-malformed-missing-script.json` — defaultPolicy.miniscript is empty `{}` (parse refusal).

### ImportProvenance accessor — sparrow_source_metadata

`crates/mnemonic-toolkit/src/wallet_import/mod.rs:121-135` — new `sparrow_source_metadata()` accessor returns `Some(&SparrowSourceMetadata)` only for the `Sparrow` variant. Required at P1B to reach the variant payload; otherwise Rust's dead-code analysis flags the tuple field. `#[allow(dead_code)]` covers the P1B → P1C interim until the JSON envelope wires the accessor.

### Removed at P1B

- `wallet_import/roundtrip.rs:1217-1227`: skeleton-cell `canonicalize_sparrow_skeleton_returns_not_yet_implemented` deleted.
- `wallet_import/roundtrip.rs:1258`: `("sparrow", canonicalize_sparrow(b""))` entry removed from the matrix `skeleton_canonicalize_helpers_accept_empty_blob` cell; updated comment to cite P1B parity with P4B.
- `wallet_import/sparrow.rs`: P1A `parse_panics_until_p1b` + `build_provenance_yields_sparrow_variant` cells dropped (no longer load-bearing once parse is real).

## Test surface (P1B cells)

`cargo test -p mnemonic-toolkit --bin mnemonic wallet_import::sparrow` — 27 cells:

- 12 sniff cells (3 positive + 9 negative) — preserved from P1A.
- 1 `SparrowPolicyType::from_str` matrix.
- 14 parse cells:
  - 4 happy-path (SINGLE wpkh / MULTI 2-of-3 sortedmulti / testnet coin-type / zpub SLIP-132 normalization).
  - 4 refusal (taproot / SINGLE-with-multi-keystores / MULTI-with-1-keystore / malformed fingerprint).
  - 1 dropped-fields NOTICE stderr cell.
  - 5 fixture-driven cells (one per fixture above).

`cargo test -p mnemonic-toolkit --bin mnemonic canonicalize_sparrow` — 6 cells:

- single_preserves_required_fields (alphabetical-order assertion).
- idempotent (re-canonicalize byte-equality).
- drops_non_preserved_top_level_fields (birthDate / gapLimit / mixConfig).
- multi_preserves_keystore_ordering (3 cosigners in declaration order).
- malformed_json_typed_error.
- bare_array_typed_error.

Full suite `cargo test -p mnemonic-toolkit`: bin target now 570 passing (up from 553 at P1A baseline; net +17 = +14 new parse cells + 6 canonicalize cells - 3 removed scaffolding cells). 0 FAILED. `cargo clippy --all-targets -- -D warnings` clean.

## Findings

### Critical

**None.**

### Important

**None.**

### Minor

#### M1 — `sparrow_source_metadata` accessor is `#[allow(dead_code)]` at P1B

The accessor exists to satisfy Rust's dead-code analysis on the `ImportProvenance::Sparrow(_)` variant payload. At P1B no production code calls it; P1C wires the JSON envelope to consume it. Mirrors the `source_metadata` / `bsms_audit` accessors' shape; the `#[allow(dead_code)]` is a P1B → P1C bridging convention rather than a defect.

#### M2 — `parse_keystore` accepts case-insensitive `masterFingerprint`

Sparrow's emit convention is lowercase 8-hex; the parser leniently accepts any 8-hex (case-insensitive). This is a defensive choice consistent with the BSMS parser's input lenience. Not a defect.

### Folds applied this round

None (R0 GREEN).

## Sign-off

P1B scope per plan-doc row 492 is fully implemented:
- ✓ Sparrow parse impl (decode `keystores[]`, extract `defaultPolicy.miniscript.script`, populate `SparrowSourceMetadata`)
- ✓ `canonicalize_sparrow` helper in `wallet_import/roundtrip.rs`
- ✓ Parse unit tests + fixtures (5 fixtures + 27 unit cells + 6 canonicalize cells)
- ✓ Cycle-followup `sparrow-taproot-descriptor-passthrough-import-support` filed
- ✓ Provenance accessor `sparrow_source_metadata` added for variant-payload reachability

Per CLAUDE.md "Per-phase reviewer-loop until 0 critical / 0 important" — R0 hits 0C/0I. P1C re-dispatches for CLI dispatch flip + integration cells.
