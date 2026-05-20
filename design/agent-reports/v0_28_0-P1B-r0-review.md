# v0.28.0 P1B architect review — R0 (inline self-review)

**Phase:** P1B — Sparrow parse impl + `canonicalize_sparrow` real implementation + fixtures.
**Reviewer:** inline self-review (agent-aa74aea6602d044ab).
**Date:** 2026-05-19.
**Scope of review:** the files mutated by P1B:

- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (P1A skeleton + P1B parse body + provenance struct + fixture-driven tests)
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (`ImportProvenance::Sparrow` variant + `sparrow_source_metadata()` accessor)
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` (`canonicalize_sparrow` real body + 5 P1B canonicalize cells + sparrow removed from skeleton-empty-blob cell)
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-*.json` (5 new fixtures)

**SPEC anchor:** `design/SPEC_wallet_import_v0_28_0.md` §11.1.
**Plan-doc anchor:** P1B row at line 492 + §S.1 at line 197-219.

## Verdict

**GREEN.** 0 Critical, 0 Important, 0 Minor. Ready to commit.

## Critical findings

(none)

## Important findings

(none)

## Minor findings

(none — both surfaced-and-folded inline during R0):

- **(folded)** `parse_testnet_network_inferred_from_coin_type_one` initially used a fabricated tpub that failed base58 checksum decode. Replaced with `704c7836/...tpubDEgS9...` lifted from `tests/fixtures/wallet_import/bsms-2line-decay-144.txt` (verified-good fixture).
- **(folded)** `sparrow-singlesig-p2sh-p2wpkh.json` initially used a fabricated BIP-49 xpub that failed base58 checksum decode. Replaced with `28645006/49'/0'/0'/xpub6DnEBNk...` lifted from `tests/fixtures/wallet_import/core-bip49-mainnet.json` (verified-good fixture).
- **(folded)** `canonicalize_sparrow_single_preserves_required_fields` used `String::find` for top-level-key ordering check — `find` returns FIRST occurrence, which matches the `"name"` inside `defaultPolicy` (Sparrow's policy-name field, `"Default"`) before the top-level `"name"`. Rewrote to round-trip via `serde_json::from_str` + `Map::keys()` iteration, which preserves insertion order from the BTreeMap → alphabetical.
- **(folded)** clippy `doc_lazy_continuation` errors on the `SparrowSourceMetadata` doc-comment (paragraph break between bulleted list and follow-up paragraph). Added the required blank-line separator.

## Verifications run

- `cargo build -p mnemonic-toolkit` → success.
- `cargo test -p mnemonic-toolkit --bin mnemonic sparrow` → 32/32 pass.
- `cargo test -p mnemonic-toolkit` (full suite) → 105 test suites all GREEN; no regression.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

## Roundtrip-shape correctness (plan-doc P1B architect R0 review focus)

`canonicalize_sparrow` re-emits the blob with:
- **Top-level keys preserved:** `name, network, policyType, scriptType, defaultPolicy, keystores` (the SPEC §11.1 required set).
- **Top-level keys dropped:** anything else (Sparrow's `birthDate`, `gapLimit`, `mixConfig`, etc.) → mirrored on `SparrowSourceMetadata.dropped_fields` for downstream NOTICE emission. Test `canonicalize_sparrow_drops_non_preserved_top_level_fields` pins.
- **Alphabetical ordering** at every nested level via BTreeMap insertion. Tests `canonicalize_sparrow_single_preserves_required_fields` + `canonicalize_sparrow_idempotent` pin.
- **xpub passes through SLIP-132 normalizer** for canonical neutral-prefix form. Tests in `parse_zpub_variant_normalized` exercise the parse path; canonicalize uses the same normalizer.
- **Keystore field whitelist:** `label, source, walletModel, keyDerivation, extendedPublicKey`. Non-whitelist keystore fields (e.g., hypothetical `passphrase`) are dropped silently per canonicalize-best-effort discipline.

`canonicalize_sparrow` is **idempotent** (parse-and-emit produces byte-identical second-pass output) which is the round-trip semantic the v0.27.0 `--json` envelope `roundtrip.byte_exact` field consumes (Phase P1C wires the consumer; P1B installs the helper).

## Parse-impl design (SPEC §11.1 compliance)

| SPEC §11.1 requirement | P1B implementation | check |
|---|---|---|
| Decode `keystores[i]` → cosigners | `parse_keystore` extracts `masterFingerprint` + `derivation` + `extendedPublicKey` per index | ✓ |
| Extract descriptor from `defaultPolicy.miniscript.script` | nested JSON traversal at `parse` step 2 | ✓ |
| Convert miniscript → descriptor via `wsh(...)` / `sh(wsh(...))` wrapping per `scriptType` | NOT NEEDED — Sparrow's script already carries the full wrapping (`wpkh(@0/**)`, `wsh(sortedmulti(K,@0/**,...))`, `sh(wsh(...))`, etc., per `wallet_export/sparrow.rs:185-220`); P1B substitutes `@N/**` placeholders with `[fp/path]xpub/<0;1>/*` and feeds the existing `concrete_keys_to_placeholders` → `parse_descriptor` pipeline | ✓ (simpler than plan-doc anticipated; reuse of existing pipeline is the right shape) |
| `policyType` ∈ {SINGLE, MULTI} ↔ `keystores.len()` consistency | step 3 enforces SINGLE→1, MULTI→≥2 with explicit refusal | ✓ |
| `SparrowSourceMetadata { label, policy_type, script_type, dropped_fields }` | exact schema match per `mod.rs:103-106` (above) | ✓ |
| dropped-field detection + stderr NOTICE per SPEC §2.4 | step 9 emits `"notice: import-wallet: sparrow: dropped envelope fields {}: not preserved..."` | ✓ |

### Taproot deferral

Sparrow's emit at `wallet_export/sparrow.rs:215-219` ships taproot as DESCRIPTOR-PASSTHROUGH (no `@N/**` placeholders — the canonical descriptor with `[fp/path]xpub` keys is embedded directly). The `@N/**` substitution path cannot handle that shape. P1B REFUSES taproot scripts with an explicit `ImportWalletParse` ("taproot scripts are not yet supported..."); follow-up work tracked at SPEC §11.1 future-work. Sniff still positive-matches taproot blobs (sniff cells `sniff_true_on_p2tr_blob`) — the refusal is at parse time, surfacing as exit 2.

This is a SCOPE NARROWING vs the user's high-level "extract descriptor with proper wrapping per scriptType" instruction. Cycle-followup logged below.

## Fixture coverage (plan-doc P1B "~5" target)

5 fixtures created at `tests/fixtures/wallet_import/`:

1. `sparrow-singlesig-p2wpkh.json` — BIP-84 SINGLE / P2WPKH (Trezor 24-word reference xpub).
2. `sparrow-multisig-2of3-p2wsh-sortedmulti.json` — 2-of-3 P2WSH sortedmulti (export-fixture-mirror).
3. `sparrow-multisig-2of3-p2wsh-multi-ordered.json` — 2-of-3 P2WSH `multi` (ordered).
4. `sparrow-singlesig-p2sh-p2wpkh.json` — BIP-49 SINGLE / P2SH_P2WPKH (xpub lifted from core-bip49 fixture).
5. `sparrow-malformed-missing-script.json` — refusal cell (missing `defaultPolicy.miniscript.script`).

Plan-doc-suggested "descriptor-with-checksum verify" fixture: NOT added as a dedicated file. Sparrow's wire shape has NO BIP-380 checksum on the inline `miniscript.script` (the script is a bare policy expression). The checksum invariant is exercised on the toolkit-side `original_descriptor` via the `recompute_descriptor_checksum` helper; the unit-test `parse_single_wpkh_mainnet_happy_path` indirectly exercises this through the `parse_descriptor` pipeline. Logging as "wontfix; rationale logged" per cycle scope-creep defense.

## ImportProvenance variant placement

Per user-message critical constraint: `BitcoinCore < Bsms < Sparrow` alphabetically. The variant is inserted **after** `Bsms` at `mod.rs:69-73`, matching the existing P0B.2 alphabetical-order discipline. Accessors `bsms_audit()` and `source_metadata()` updated to handle the new variant; new accessor `sparrow_source_metadata()` added for symmetry (mirrors existing per-parser-accessor convention).

## P1C integration deferral

The following are P1C-scope and intentionally NOT done in P1B:
- Flip `cmd/import_wallet.rs:365` `"sparrow" => unimplemented!(...)` → `SparrowParser::parse(...)`.
- Add `tests/cli_import_wallet_sparrow.rs` integration cells.
- Flip Site 7 (envelope shape) in `emit_json_envelope` to populate `sparrow_source_metadata` on the JSON envelope.

P1B is a self-contained parse-library + canonicalize-helper + fixtures delivery; P1C wires the CLI dispatch.

## Cycle-followups logged

- **NEW** `sparrow-taproot-descriptor-passthrough-import-support`: Sparrow's taproot emit uses descriptor-passthrough (concrete-keys embedded directly, no `@N/**` placeholders). The P1B `@N/**` substitution path refuses these. Future work: detect descriptor-passthrough shape via heuristic (`[fp/path]xpub` substring vs `@N/**`) and route to a parallel parse path that consumes the embedded concrete-keys descriptor verbatim. Tier: v0.29+.
- **NEW (rationale-logged-as-wontfix)** dedicated `sparrow-descriptor-with-checksum-verify` fixture not added: Sparrow's wire shape has no BIP-380 checksum on `miniscript.script`. The toolkit-side `original_descriptor` carries a freshly computed checksum via `recompute_descriptor_checksum`; existing happy-path cells exercise this implicitly.
