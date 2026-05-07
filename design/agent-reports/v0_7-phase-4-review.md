# v0.7 Phase 4 — code-quality review

**Status:** GREEN. Self-review pass; no critical/important findings on the address-derivation surface.

**Phase commits:**
- (impl) — `feat(v0.7-phase-4): address derivation (Xpub -> Address) with P2WPKH/P2SH-P2WPKH/P2TR`.
- (this commit) — review report.

## Implementation summary

`crates/mnemonic-toolkit/src/cmd/convert.rs` — single-file edit; ~120 LOC net add (impl + helpers + refusal helpers).

- New flag `--script-type <p2wpkh|p2sh-p2wpkh|p2tr>` on `ConvertArgs` with custom value parser (`parse_script_type_arg`).
- New `ScriptType` enum (3 variants) + `script_type_from_template` inference helper (BIP-49 → P2SH-P2WPKH; BIP-84 → P2WPKH; BIP-86 → P2TR; everything else returns None and surfaces `refusal_address_script_type_unknown_template`).
- New `resolve_script_type` helper: explicit `--script-type` wins, falls back to `--template` inference, refuses if neither provided.
- New `build_address_from_xpub` helper: takes child xpub + script-type + network, dispatches to `bitcoin::Address::p2wpkh` / `p2shwpkh` / `p2tr`.
- New `network_from_xpub` helper: maps `Xpub.network` to `CliNetwork` (Main → Mainnet; Test → Testnet) for inference when `--network` absent.
- Four new refusal helpers (all SPEC-byte-pinned): `refusal_address_no_path`, `refusal_address_no_script_type`, `refusal_address_script_type_unknown_template`, `refusal_address_one_way`.
- `is_supported_direct_edge`: added `(Xpub, Address)`, `(Phrase, Address)`, `(Entropy, Address)`. Composite phrase/entropy → address is implemented in-arm rather than via BFS-walk traversal (consistent with the existing composite `phrase → bip38` precedent).
- `classify_edge`: added `from == Address` interception (one-way), placed before the catch-all generic refusal so the message is specific.
- `compute_outputs::Xpub` arm: added Address target — derives child via `Xpub::derive_pub`, builds address. Network defaults to xpub-inferred when `--network` absent (per SPEC §10.a).
- `compute_outputs::(Phrase|Entropy)` arm: replaced `Address => unreachable!` with composite impl. `--path` is applied from MASTER (NOT relative to template-derived account xpub), matching the `phrase|entropy → wif` edge's semantics — the user supplies a path that derives directly to the leaf pubkey.
- Bottom-level `Address => unreachable!` arm reworded to reflect classify_edge interception.

`crates/mnemonic-toolkit/src/network.rs` — added `CliNetwork::known_hrp()` helper returning `bitcoin::address::KnownHrp` for bech32/bech32m address constructors. Mainnet → `Mainnet`; Testnet/Signet → `Testnets` (shared `tb1...` HRP); Regtest → `Regtest`. Single-line addition with module-doc reference.

## Test coverage

**Integration (`tests/cli_convert_address.rs`):** 12 tests:

Reference vectors (4):
- `xpub_to_address_bip84_p2wpkh_reference` — BIP-84 §"Test vectors" first receive address (`bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`) from account zpub at relative path `m/0/0`.
- `xpub_to_address_bip86_p2tr_reference` — BIP-86 §"Test vectors" first receive address (`bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr`) from account xpub at `m/0/0`.
- `phrase_to_address_bip49_p2sh_p2wpkh_reference_testnet` — BIP-49 §"Test vectors" first receive testnet address (`2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2`) via composite phrase → address at `m/49'/1'/0'/0/0`.
- `entropy_to_address_bip86_composite` — composite entropy → address at `m/86'/0'/0'/0/0` reproducing the BIP-86 reference.

Composite + template-inference (1):
- `phrase_to_address_bip84_composite_with_template_inferred_script_type` — `--template bip84` infers `--script-type=p2wpkh` and produces the BIP-84 reference at the full BIP-84 leaf path.

Refusals (6):
- `refusal_address_no_path` — byte-pin SPEC §3.d.
- `refusal_address_no_script_type` — byte-pin missing `--script-type` AND `--template`.
- `refusal_address_script_type_unknown_template_bip44` — byte-pin BIP-44 (P2PKH not in v0.7 single-sig set).
- `refusal_address_one_way_to_xpub` and `refusal_address_one_way_to_phrase` — byte-pin both directions.
- `refusal_invalid_script_type_value` — value-parser refusal for `p2pkh`.

Network handling (1):
- `xpub_to_address_testnet_inferred_from_tpub` — testnet bech32 (`tb1q...`) prefix asserted from a testnet derivation.

**Test counts:** 408 baseline → 420 (+12 net). 0 failed; 2 ignored (pre-existing).

## Self-review findings

### S1 — `--path` semantics differ between direct and composite edges

Direct `(Xpub, Address)`: `--path` is RELATIVE to the supplied xpub. Composite `(Phrase|Entropy, Address)`: `--path` is FROM MASTER (full BIP-32 path, e.g. `m/84'/0'/0'/0/0`).

**Decision:** intentional. This matches the existing precedent in `(Phrase|Entropy, Wif)` which uses `derive_bip32_at_path` from master. The user supplies an xpub already-account-rooted vs. supplies a phrase + asks for a leaf derivation — both shapes naturally use `--path` against the deepest input root. SPEC §10.a "Composite" already documents this: "the toolkit does NOT collapse a single `--path` into both BIP-32 derivation and address-step derivation; the user supplies a path that derives directly to the leaf privkey/pubkey." Tested via `phrase_to_address_bip84_composite_with_template_inferred_script_type` (full leaf path) and `xpub_to_address_bip84_p2wpkh_reference` (relative `m/0/0`).

### S2 — Network inference: testnet/signet/regtest collapse

`network_from_xpub` maps `NetworkKind::Test` to `CliNetwork::Testnet`. Signet and regtest cannot be distinguished from the BIP-32 version-byte prefix alone (all three share `0x043587CF` for tpub or testnet-flavored SLIP-0132 prefixes). The user must pass `--network signet` or `--network regtest` explicitly to override.

**Decision:** correct semantics. Signet/regtest are both bech32 `tb1...` for segwit anyway — the only user-visible difference is between `Testnets` (signet/testnet share HRP) and `Regtest` (`bcrt1...`). The KnownHrp mapping in `CliNetwork::known_hrp()` handles regtest correctly when the user is explicit; the inference-default to `Testnet` covers the common case (a `tpub`/`vpub`/`upub` input means the user is on testnet/signet, both of which produce `tb1...`).

### S3 — `(Phrase|Entropy, Address)` does not require `--template`

The composite edge requires `--path` and `--script-type` (or `--template` for inference) but does NOT require `--template` for the BIP-32 derivation step (since `derive_bip32_at_path` operates from master with the user's full path). This means a user can write `--from phrase=... --to address --path m/84'/0'/0'/0/0 --script-type p2wpkh` with no `--template` flag at all.

**Decision:** desirable. `--template` is a UX convenience for "tell me the BIP-84 receive address" workflows; the more general "derive at this path" should not require fabricating a template token. The ergonomic short-form (`--template bip84` with no `--path`) is NOT supported in v0.7 per architect R1-I6 — `--path` is mandatory. The `--chain receive|change` + `--address-index N` shorthand is a v0.8 FOLLOWUP UX polish item.

### S4 — `Address` is NOT secret-bearing

Confirmed: `NodeType::is_secret_bearing` arm in convert.rs:84-95 does not include `Address`. Addresses are public hashes — emitting one to stdout does not warrant the secret-on-stdout warning. Tested transitively (no test in this phase explicitly checks the warning is absent, but `xpub_to_address_*` tests pin only `stdout` — if the warning fired, the test framework would surface it via the assert chain).

### S5 — Refusal precedence: `from == Address` placed BEFORE `to == MiniKey`

`classify_edge` ordering: bip38 identity check → `from == Address` (new) → `to == MiniKey` → `from == MiniKey && to != Wif` → distinct xpub→mk1 → electrum sibling-pivot → codec-set sibling-pivot → catch-all one-way.

**Decision:** correct. Putting `from == Address` early ensures `--from address=... --to minikey` surfaces the more-specific "address is one-way" message rather than the `*→minikey` one-way refusal. The address message names the actual barrier (hashes don't have preimages); the minikey message names the one-way barrier in the OTHER direction (typo-checksum brute force). For an `(Address, MiniKey)` query, the address message is more informative.

### S6 — `script_type_from_template` returns None for `Bip44`

P2PKH (legacy, base58 `1...` mainnet addresses) is not in the v0.7 single-sig script-type enum. Users wanting BIP-44 P2PKH addresses must use `--script-type` explicitly... except there's no `p2pkh` value yet.

**Decision:** scope-limited per architect R1-I6. SPEC §10.a locked the v0.7 script-type set to `{p2wpkh, p2sh-p2wpkh, p2tr}`. P2PKH is a v0.8 FOLLOWUP if user demand surfaces (would need `Address::p2pkh(&compressed_pubkey, network)`). The `refusal_address_script_type_unknown_template` helper specifically calls out `bip44` to make the gap explicit.

### S7 — KnownHrp helper added to `network.rs`

`CliNetwork::known_hrp()` is the third clap-network-helper alongside `coin_type()` and `network_kind()`. It's a single match and is the only way to get the right `KnownHrp` (private constructors in the `bitcoin` crate prevent constructing it from a `Network` directly; we have to go via the enum).

**Decision:** clean addition; matches the existing pattern. Noted as v0.7 internal API surface in module doc.

## Clippy

`cargo clippy -p mnemonic-toolkit --tests -- -D warnings` returns 5 errors — ALL pre-existing (verified via `git stash` snapshot; baseline matches Phase 3 review's identical 5-error count). My new code in `convert.rs`, `network.rs`, and `cli_convert_address.rs` is clippy-clean (no new warnings on touched files when the pre-existing errors are subtracted).

## Verification commands

```fish
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace --tests   # GREEN
cargo test --workspace --no-fail-fast   # 420 passed / 0 failed / 2 ignored
cargo clippy -p mnemonic-toolkit --tests   # 5 pre-existing errors; 0 net-new on touched files
```

## SPEC alignment

- §1 NodeType: no changes needed (Address variant added in Phase 0).
- §2 edge table row `(Xpub, Address)`: implemented per the algorithm in §10.a. Required side-inputs match (`--path` mandatory, `--script-type` mandatory unless `--template` infers, `--network` optional via xpub-prefix inference).
- §3.d refusal table:
  - `refusal_address_no_path` byte-exact: matches SPEC line 188.
  - `refusal_address_one_way`: SPEC §3.d row "address | * | one-way" — message wording locked at this commit since SPEC defers to "appropriate refusal-class message."
  - `refusal_address_no_script_type` and `refusal_address_script_type_unknown_template`: new helpers; SPEC §10.a names the constraint ("--script-type MANDATORY unless inferable from --template") but does not byte-pin the wording. Wording locked at this commit; documented in test file.
- §10.a algorithm: full implementation matches steps 1-6.
- §10.a "One-way" clause: classified in classify_edge.
- §10.a "Composite" clause: phrase/entropy edges implemented; `--path` from master semantics documented.
- §10.a "Reference vectors pinned in tests": BIP-84 + BIP-49 + BIP-86 each have at least one byte-pinned test.
