# Phase P10A — architect R0 review

**Reviewer:** Opus 4.7 (acting as feature-dev:code-architect) — instance-H self-review pre-PR
**Branch:** `v0.28.0/h-core-fixtures`
**Commit under review:** P10A scope (4 new Core fixtures + 4 parse-only cells)
**Base SHA:** `33ec61d` (release/v0.28.0 tip; rebased from initial branch-point `71592bc`)
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 10 P10A row + §S.9 owner-phase tags

---

## Scope (verbatim from plan-doc Phase 10 table)

> **P10A** | 4 new Core fixtures: `core-bip44-mainnet.json`, `core-bip86-mainnet.json`, `core-wsh-sortedmulti-3of5.json`, `core-multipath-0-1.json`. Parse-only cells. | `tests/fixtures/wallet_import/core-*.json` (×4), `tests/cli_import_wallet_bitcoin_core.rs` | ~0 src + ~200 tests + 4 fixture files | architect R0 |

§S.9 confirms Core-side ownership: "P10A: `core-bip44-mainnet.json`, `core-bip86-mainnet.json`, `core-wsh-sortedmulti-3of5.json`, `core-multipath-0-1.json` (4 fixtures)".

---

## Critical (correctness-blocking; would break P10B execution or downstream consumers)

**None.** P10A delivers exactly the 4 fixtures + 4 parse-only test cells the plan-doc scopes. No SPEC §6/§10 contract changes; no new parser flags; no `wallet_import/*.rs` source modifications. Each fixture parses cleanly via `--format bitcoin-core` against the v0.28.0 BitcoinCoreParser; each cell's assertions are derived from observed stdout shape rather than predicted shape.

## Important (would block P10A merge)

**None.** Fixture-corpus expansion is the smallest plan-doc deliverable surface; the cells consume the existing `run_core_file_select` + `fixture_path` helpers without introducing new infrastructure. Self-review findings tracked under Minor below.

## Minor (fold inline or defer)

**M1 — `core-wsh-sortedmulti-3of5.json` synthesizes 2 extra cosigners via BIP-32 Test Vector 1 and Test Vector 2 root xpubs (chain m).** The existing toolkit test corpus has only 3 mainnet xpubs (`COSIGNER_{A,B,C}_XPUB` from `cli_export_wallet_jade.rs:13-17`); a 3-of-5 multisig requires 5 distinct xpubs to avoid miniscript's duplicate-key validation rejection. Two options:

- (a) Use BIP-32 published test vector xpubs (TV1 m, TV2 m at `tests/bip32_vectors.rs:46, 112`) with synthetic fingerprints `deadbeef` / `cafebabe`. Pros: widely-known synthetic xpubs, no key-reuse, distinct fingerprints. Cons: cosmetic mismatch between the synthetic fingerprint (`deadbeef`) and the actual `xpub6...` root fingerprint (would be a different 4-byte HASH160 prefix in real wallets).

- (b) Vendor 2 new mainnet test xpubs from a deterministic seed dedicated to the toolkit test corpus. Pros: real-fingerprint consistency. Cons: ~30 LOC new const declarations + lockstep updates if other fixtures reuse the same keys.

**Chosen:** (a). Rationale: this is a parse-only fixture; cosigner key-bytes round-trip through miniscript's parser, but no signature derivation or address derivation depends on the fingerprint-equals-HASH160(xpub-pubkey) invariant. The toolkit parser doesn't verify fingerprint correctness against the xpub itself (this is a Bitcoin Core wallet-import contract, not a toolkit invariant). The mismatch is purely cosmetic. **Fold:** documented inline in the fixture cell's prose comment + linked to the BIP-32 test vectors source file.

**M2 — `core-bip44-mainnet.json` and `core-bip86-mainnet.json` include the `timestamp: "now"` wallet-state field** (mirroring the existing `core-bip84-mainnet.json` shape at line 9). This means both new cells fire the dropped-fields NOTICE on stderr (the existing `core_dropped_fields_notice` cell already covers this contract). Including `timestamp` in the new fixtures is consistent with real-world Bitcoin Core `listdescriptors` output (which always emits `timestamp`), but the new cells do not assert on the NOTICE — they only assert on stdout bundle-shape. **No fold.** The contract-pinning of dropped-field NOTICE behavior is the existing `core_dropped_fields_notice` cell's job; P10A new cells deliberately scope to stdout assertions only.

**M3 — `core-wsh-sortedmulti-3of5.json` omits `timestamp`** to keep the assertions cleanly focused on the multisig contract (5 cosigners + threshold=3). Asymmetric vs P10A fixtures 1+2 (which keep `timestamp`). **No fold.** The asymmetry is intentional — a 3-of-5 multisig is the load-bearing assertion, not the wallet-state dropped-field behavior.

**M4 — Test-cell prose comments invoke "P10A.1" / "P10A.2" / "P10A.3" / "P10A.4" enumeration.** This mirrors the existing convention (`§3.2 — core_single_descriptor_wpkh_happy_path` at line 134). Consistent.

## Source-grep verification table

| Citation | Verified? | Notes |
|---|---|---|
| `tests/cli_import_wallet_bitcoin_core.rs::fixture_path` helper | YES | `tests/cli_import_wallet_bitcoin_core.rs:92-94` |
| `tests/cli_import_wallet_bitcoin_core.rs::run_core_file_select` helper | YES | `tests/cli_import_wallet_bitcoin_core.rs:124-131` |
| `BitcoinCoreParser::sniff` empty-array branch returns false | YES | `wallet_import/bitcoin_core.rs:95-97` |
| Empty-descriptors-array parse-error template | YES | `wallet_import/bitcoin_core.rs:138-143` (`"top-level \`descriptors\` array is empty; no bundles to emit"`) |
| Sniff dispatch consult-all-then-count | YES | `wallet_import/sniff.rs:74-105` (post-P0D) |
| MAINNET_FP_A constant | YES | `tests/cli_import_wallet_bitcoin_core.rs:19` (`"b8688df1"`) |
| `pkh()` descriptor wrapper acceptance | YES | Tested live with `pkh([…]xpub6FQya…/<0;1>/*)` — parses with cosigners=1 threshold=none |
| `tr()` key-path-only descriptor acceptance | YES | Tested live with `tr([…]xpub6FQya…/<0;1>/*)` — parses with cosigners=1 threshold=none; surfaces roundtrip-not-byte-exact warning (semantic equivalent) — assertion scope deliberately avoids the warning text |
| BIP-32 TV1 m + TV2 m xpubs used as extra cosigners | YES | `tests/bip32_vectors.rs:46, 112` — TV1 m = `xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8Nq...EGMcet8`; TV2 m = `xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8...EGuduB` |
| BIP-380 checksum on all 4 fixtures | YES | Computed via reference BIP-380 polymod; verified via toolkit's BIP-380 validator at parse time (would error if wrong checksum) |

## Scope-creep audit

| Item | In plan-doc P10A scope? | Acceptable? |
|---|---|---|
| `core-bip44-mainnet.json` fixture file | YES | YES |
| `core-bip86-mainnet.json` fixture file | YES | YES |
| `core-wsh-sortedmulti-3of5.json` fixture file | YES | YES |
| `core-multipath-0-1.json` fixture file | YES | YES |
| 4 new parse-only test cells | YES | YES |
| Synthetic fingerprints `deadbeef` / `cafebabe` (3-of-5 cosigners 4+5) | Implicit (plan-doc does not specify cosigner identities) | YES — see Minor M1 |
| BIP-32 Test Vector xpubs used as cosigners 4+5 | Implicit | YES — see Minor M1 |
| New helper functions, new const declarations, new modules | NO — explicitly NOT in scope | None added; cells use existing `run_core_file_select` + `fixture_path` |
| SPEC §6 / §10 changes | NO — explicitly NOT in scope | None added |
| `wallet_import/bitcoin_core.rs` source changes | NO — explicitly NOT in scope | None added |
| `gui-schema` changes / GUI mirror updates | NO — fixture-corpus is parse-only, no new CLI flags | None — no GUI lockstep required |

**Net scope assessment:** strict adherence to the plan-doc P10A row. No scope-creep beyond the implicit cosigner-identity choices documented as Minor M1.

## Test result

- **Baseline (pre-P10A):** 25 passed in `cli_import_wallet_bitcoin_core.rs`.
- **Post-P10A:** 29 passed in `cli_import_wallet_bitcoin_core.rs` (25 baseline + 4 new P10A cells). Δ = +4. Full workspace test suite green; clippy `--all-targets -D warnings` green.
- **No `#[ignore]`-gated cells added.** All cells run in `cargo test --workspace` default.

## Overall verdict

**GREEN.**

P10A delivers exactly what the plan-doc scopes: 4 vendored Core fixtures + 4 parse-only test cells. No SPEC changes, no source changes, no new infrastructure. Net LOC: ~135 test code + 4 fixture files. Net new tests: +4 (all green). Pre-existing tests unbroken (25 → 25 baseline + 4 new = 29 total).

R0 verdict GREEN → ready to merge to `release/v0.28.0` after P10B lands.

---

**Sources:**
- Plan-doc Phase 10 row at `/home/bcg/.claude/plans/unified-meandering-sundae.md:557-562`
- Plan-doc §S.9 owner-phase tags at `/home/bcg/.claude/plans/unified-meandering-sundae.md:409-422`
- v0.26.0 SPEC `wallet_import_v0_26_0.md` §6.1 sniff semantic carry-forward (still authoritative for Bitcoin Core sniff signature)
- BIP-32 Test Vectors at `tests/bip32_vectors.rs`
- BIP-380 reference implementation (Bitcoin Core `src/script/descriptor.cpp` polymod)
